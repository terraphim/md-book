# src/pagefind_service.rs Summary

## Purpose
Integration with Pagefind search engine for full-text documentation search.

## Key Types
- `PagefindBuilder`: Builder for configuring and running Pagefind indexing
- `PagefindError`: Error types for configuration, indexing, I/O, and WASM errors

## Implementation
When `search` feature is enabled:
- `new(source_path)`: Validates source path exists
- `build()`: Runs pagefind CLI as subprocess (`pagefind --site <path>`)
- Reports indexing time using `jiff` for timing

When `search` feature is disabled:
- All methods return `PagefindError::IndexingFailed`

## Error Types
- `Config`: Configuration errors
- `MultipleConfigs`: Multiple config files found
- `InvalidConfigFormat`: Bad config file format
- `SourcePathNotFound`: Source directory missing
- `IndexingFailed`: Pagefind command failure
- `Io`: File system errors
- `WasmError`: WASM-specific failures

## Prerequisites
Requires `pagefind` CLI tool to be installed and available in PATH.
