//! This module about structural warnings

mod commands;
mod dto;
mod handlers;
mod repository;

pub use handlers::setup_warnings_handler;
pub use repository::WarnsRepository;
