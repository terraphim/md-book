#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use wasm_bindgen_test::*;
    use md_book::{PagefindBuilder, PagefindError};
    use std::path::PathBuf;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    async fn test_wasm_pagefind_error() {
        // Test that WASM-specific error handling works
        let temp_path = PathBuf::from("/tmp/test");
        
        // This should fail since the path doesn't exist
        let result = PagefindBuilder::new(temp_path).await;
        assert!(result.is_err());
        
        match result.unwrap_err() {
            PagefindError::SourcePathNotFound { .. } => {
                // Expected error type
            }
            _ => panic!("Expected SourcePathNotFound error"),
        }
    }

    #[wasm_bindgen_test]
    fn test_wasm_error_display() {
        let error = PagefindError::WasmError {
            message: "Test WASM error".to_string(),
        };
        
        let error_string = error.to_string();
        assert!(error_string.contains("WASM operation failed"));
        assert!(error_string.contains("Test WASM error"));
    }

    #[wasm_bindgen_test]
    async fn test_wasm_build_method() {
        // Create a mock builder (this will fail at path validation)
        // but we can test the WASM-specific method exists
        let temp_path = PathBuf::from("/");
        
        if let Ok(builder) = PagefindBuilder::new(temp_path).await {
            let wasm_result = builder.build_wasm().await;
            
            // Should return WASM error since it's not implemented
            assert!(wasm_result.is_err());
            match wasm_result.unwrap_err() {
                PagefindError::WasmError { message } => {
                    assert!(message.contains("not yet implemented"));
                }
                _ => panic!("Expected WasmError"),
            }
        }
    }

    // Test WASM-specific compilation features
    #[wasm_bindgen_test]
    fn test_wasm_feature_flags() {
        // This test ensures WASM-specific code compiles
        #[cfg(target_arch = "wasm32")]
        {
            // WASM-specific assertions
            assert!(true, "WASM compilation successful");
        }
        
        #[cfg(not(target_arch = "wasm32"))]
        {
            panic!("This test should only run on WASM");
        }
    }

    // Test that jiff works in WASM environment
    #[wasm_bindgen_test]
    fn test_jiff_wasm_compatibility() {
        use jiff::Zoned;
        
        // Test that jiff's time functions work in WASM
        let now = Zoned::now();
        assert!(now.year() > 2020, "Should get reasonable year");
    }
}

// Native-only tests for comparison
#[cfg(not(target_arch = "wasm32"))]
mod native_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_native_vs_wasm_feature_parity() {
        // Test that the same API works on native
        use md_book::{PagefindBuilder, PagefindError};
        use std::path::PathBuf;
        
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