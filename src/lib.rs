pub mod api;
pub mod cli;
pub mod core;
pub mod models;
pub mod services;
pub mod web;

use std::cell::RefCell;

// 使用线程局部存储来为每个线程创建独立的MessageRepository实例
thread_local! {
    pub static REPO: RefCell<Option<core::message::MessageRepository>> = const { RefCell::new(None) };
}

// 导出线程局部存储的REPO
pub use crate::core::message::MessageRepository;
