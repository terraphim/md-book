pub mod config;
pub mod pagefind_service;
pub mod core;

// Optional server module for native builds only
#[cfg(feature = "server")]
pub mod server;

pub use config::BookConfig;
pub use pagefind_service::{PagefindBuilder, PagefindError};
pub use core::{build, Args, PageInfo};

// Re-export server functionality when available
#[cfg(feature = "server")]
pub use server::serve_book;

// WASM-specific exports
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_process_markdown(content: &str) -> String {
    // Basic markdown processing for WASM
    markdown::to_html(content)
}