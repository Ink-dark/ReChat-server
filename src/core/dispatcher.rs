use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

use crate::core::adapter::AdapterManager;
use crate::core::message::MessageRepository;
use crate::models::message::MessageStatus;

pub struct MessageDispatcher {
    adapter_manager: Arc<AdapterManager>,
    db_path: String,
    max_retries: u32,
    retry_interval: u64,
    batch_size: usize,
    concurrency: usize,
    shutdown: Arc<AtomicBool>,
}

impl MessageDispatcher {
    pub fn new(
        adapter_manager: Arc<AdapterManager>,
        db_path: String,
        max_retries: u32,
        retry_interval: u64,
        batch_size: usize,
        concurrency: usize,
    ) -> Self {
        Self {
            adapter_manager,
            db_path,
            max_retries,
            retry_interval,
            batch_size,
            concurrency,
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn shutdown_handle(&self) -> Arc<AtomicBool> {
        self.shutdown.clone()
    }

    pub fn start(self) {
        let retry_interval_secs = self.retry_interval;
        let batch_size = self.batch_size;
        let max_retries = self.max_retries;
        let concurrency = self.concurrency;
        let adapter_manager = self.adapter_manager;
        let db_path = self.db_path;
        let shutdown = self.shutdown;

        tokio::spawn(async move {
            let sem = Arc::new(tokio::sync::Semaphore::new(concurrency));

            loop {
                if shutdown.load(Ordering::Relaxed) {
                    tracing::info!("Message dispatcher shutting down gracefully");
                    sem.close();
                    break;
                }

                let pending = match MessageRepository::new(&db_path) {
                    Ok(repo) => repo.get_pending_messages(batch_size).unwrap_or_default(),
                    Err(e) => {
                        tracing::error!(error = %e, "Dispatcher failed to read pending messages");
                        tokio::time::sleep(tokio::time::Duration::from_secs(retry_interval_secs))
                            .await;
                        continue;
                    }
                };

                if pending.is_empty() {
                    tokio::time::sleep(tokio::time::Duration::from_secs(retry_interval_secs)).await;
                    continue;
                }

                for message in pending {
                    let sem = sem.clone();
                    let am = adapter_manager.clone();
                    let db_path = db_path.clone();
                    let shutdown = shutdown.clone();

                    tokio::spawn(async move {
                        let _permit = sem.acquire().await;

                        let repo = match MessageRepository::new(&db_path) {
                            Ok(r) => r,
                            Err(e) => {
                                tracing::error!(
                                    error = %e,
                                    "Failed to open DB in dispatcher worker"
                                );
                                return;
                            }
                        };

                        let mut sent = false;

                        for attempt in 0..=max_retries {
                            if shutdown.load(Ordering::Relaxed) {
                                return;
                            }

                            if attempt > 0 {
                                tracing::info!(
                                    message_id = %message.id,
                                    attempt = attempt,
                                    max_retries = max_retries,
                                    "Retrying message send"
                                );
                                tokio::time::sleep(tokio::time::Duration::from_secs(
                                    retry_interval_secs,
                                ))
                                .await;
                            }

                            match am.send_to_adapter(&message.recipient, &message) {
                                Ok(()) => {
                                    tracing::info!(
                                        message_id = %message.id,
                                        "Message sent successfully"
                                    );
                                    if let Err(e) = repo
                                        .update_message_status(&message.id, &MessageStatus::Sent)
                                    {
                                        tracing::error!(
                                            error = %e,
                                            message_id = %message.id,
                                            "Failed to update message status to Sent"
                                        );
                                    }
                                    sent = true;
                                    break;
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        message_id = %message.id,
                                        attempt = attempt,
                                        error = %e,
                                        "Failed to send message"
                                    );
                                    if let Err(e) = repo.increment_retry(&message.id) {
                                        tracing::error!(
                                            error = %e,
                                            "Failed to increment retry count"
                                        );
                                    }
                                }
                            }
                        }

                        if !sent {
                            tracing::warn!(
                                message_id = %message.id,
                                "Message failed after all retries, marking as Failed"
                            );
                            if let Err(e) =
                                repo.update_message_status(&message.id, &MessageStatus::Failed)
                            {
                                tracing::error!(
                                    error = %e,
                                    message_id = %message.id,
                                    "Failed to update message status to Failed"
                                );
                            }
                        }
                    });
                }
            }
        });
    }
}
