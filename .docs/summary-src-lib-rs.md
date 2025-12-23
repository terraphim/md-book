# src/lib.rs Summary

## Purpose
Library root that exports public API and manages feature-gated modules.

## Modules
- `config`: Configuration system
- `core`: Core build functionality
- `pagefind_service`: Search indexing
- `server`: Development server (feature-gated)

## Public Exports
- `BookConfig`: Configuration struct
- `build`, `Args`, `PageInfo`: Core build types
- `PagefindBuilder`, `PagefindError`: Search types
- `serve_book`: Server function (when server feature enabled)

## WASM Support
Provides `wasm_process_markdown()` function for basic markdown processing in WebAssembly environments.
