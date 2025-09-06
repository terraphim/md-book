use anyhow::Result;
use std::path::PathBuf;
use thiserror::Error;
use jiff::Zoned;

#[derive(Error, Debug)]
pub enum PagefindError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),
    
    #[error("Multiple config files found: {files:?}. Please ensure only one exists")]
    MultipleConfigs { files: Vec<String> },
    
    #[error("Invalid config file format: {path}")]
    InvalidConfigFormat { path: String },
    
    #[error("Source path does not exist: {path}")]
    SourcePathNotFound { path: PathBuf },
    
    #[error("Indexing failed: {message}")]
    IndexingFailed { message: String },
    
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    
    #[cfg(target_arch = "wasm32")]
    #[error("WASM operation failed: {message}")]
    WasmError { message: String },
}


#[derive(Debug)]
pub struct PagefindBuilder {
    source_path: PathBuf,
}

impl PagefindBuilder {
    pub async fn new(source_path: PathBuf) -> Result<Self, PagefindError> {
        // Validate source path exists
        if !source_path.exists() {
            return Err(PagefindError::SourcePathNotFound { path: source_path });
        }

        Ok(Self { source_path })
    }

    pub async fn build(&self) -> Result<(), PagefindError> {
        let start_time = Zoned::now();
        
        // Simple implementation using tokio command to run pagefind CLI
        // This is a fallback approach when the Rust API is not stable
        let output = tokio::process::Command::new("pagefind")
            .arg("--site")
            .arg(&self.source_path)
            .output()
            .await
            .map_err(|e| PagefindError::IndexingFailed { 
                message: format!("Failed to run pagefind command: {}", e)
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(PagefindError::IndexingFailed {
                message: format!("Pagefind command failed: {}", stderr)
            });
        }

        let end_time = Zoned::now();
        let duration = end_time.since(&start_time).map_err(|e| PagefindError::IndexingFailed {
            message: format!("Time calculation failed: {}", e)
        })?;
        
        println!("Pagefind indexing completed in {}ms", duration.total(jiff::Unit::Millisecond).unwrap_or(0.0));
        
        Ok(())
    }
    
    /// Returns the configured source path
    pub fn source_path(&self) -> Option<&PathBuf> {
        Some(&self.source_path)
    }
    
    #[cfg(target_arch = "wasm32")]
    pub async fn build_wasm(&self) -> Result<(), PagefindError> {
        // WASM-specific implementation
        // This would use different APIs optimized for WebAssembly
        Err(PagefindError::WasmError { 
            message: "WASM build not yet implemented".to_string() 
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_pagefind_builder_new() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().to_path_buf();
        
        let builder = PagefindBuilder::new(source_path.clone()).await;
        assert!(builder.is_ok());
        
        let builder = builder.unwrap();
        assert_eq!(builder.source_path(), Some(&source_path));
    }
    
    #[tokio::test]
    async fn test_invalid_source_path() {
        let invalid_path = PathBuf::from("/nonexistent/path");
        let result = PagefindBuilder::new(invalid_path.clone()).await;
        
        assert!(result.is_err());
        match result.unwrap_err() {
            PagefindError::SourcePathNotFound { path } => {
                assert_eq!(path, invalid_path);
            }
            _ => panic!("Expected SourcePathNotFound error"),
        }
    }
} 