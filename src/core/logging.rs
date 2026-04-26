use std::fs;
use std::fs::File;
use std::sync::Mutex;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub fn init() {
    let _ = fs::create_dir_all("logs");

    let log_file = match File::create("logs/rechat.log") {
        Ok(f) => Some(Mutex::new(f)),
        Err(e) => {
            eprintln!(
                "Warning: failed to create log file: {}. Logging to stderr only.",
                e
            );
            None
        }
    };

    match log_file {
        Some(file) => init_with_file(file),
        None => init_stderr_only(),
    }
}

#[cfg(not(feature = "windows-gui"))]
fn init_with_file(file: Mutex<File>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_layer = fmt::layer().with_ansi(false).with_target(false);
    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_target(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer.with_writer(file))
        .with(stderr_layer)
        .init();
}

#[cfg(not(feature = "windows-gui"))]
fn init_stderr_only() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let stderr_layer = fmt::layer().with_writer(std::io::stderr).with_target(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .init();
}

#[cfg(feature = "windows-gui")]
fn init_with_file(file: Mutex<File>) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let file_layer = fmt::layer().with_ansi(false).with_target(false);
    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer.with_writer(file))
        .init();
}

#[cfg(feature = "windows-gui")]
fn init_stderr_only() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry().with(filter).init();
}
