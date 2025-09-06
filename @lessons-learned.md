# @lessons-learned.md - Knowledge Retention

## Pagefind Search Implementation Insights

### Key Technical Learnings

#### 1. Pagefind Architecture
- **Static-first Design**: Pagefind generates static search bundles that work without servers
- **Chunked Indexes**: Only loads necessary parts of search index on demand
- **Rust Library**: v1.3.0 provides stable Rust API with feature parity to Node/Python
- **Performance**: ~100x improvement in v1.0+ for indexing operations

#### 2. WASM Integration Patterns
- **Conditional Compilation**: Use `#[cfg(target_arch = "wasm32")]` for WASM-specific code
- **Feature Flags**: Separate native and WASM builds with cargo features
- **CSP Considerations**: WebAssembly requires `wasm-unsafe-eval` directive
- **Time Handling**: `jiff` crate provides better WASM compatibility than `chrono`

#### 3. Async Rust Best Practices
- **Tokio Integration**: Use `tokio::test` for async testing
- **Error Propagation**: Custom error types with `thiserror` or `anyhow`
- **Channel Patterns**: `tokio::sync::broadcast` for live reload notifications
- **Structured Concurrency**: Prefer scoped tasks over fire-and-forget spawning

#### 4. Testing Strategy Without Mocks
- **Integration Testing**: Test real file system operations with temporary directories
- **Browser Testing**: Use real Pagefind bundles in frontend tests
- **Performance Testing**: Criterion for benchmarking with statistical analysis
- **WASM Testing**: `wasm-bindgen-test` with browser or Node.js runners

#### 5. Frontend Integration Patterns
- **Progressive Enhancement**: Search works with and without JavaScript
- **Keyboard UX**: Standard `/` key shortcut for search activation
- **URL Parameters**: `?q=search` support for shareable searches  
- **Debounced Search**: Prevent excessive API calls during typing
- **Local Storage**: Cache search history and preferences

### Configuration Management
- **Layered Config**: Environment vars → CLI args → config files → defaults
- **Multiple Formats**: TOML, JSON, YAML support with `twelf` crate
- **Shell Expansion**: Path variables expanded in config files
- **Validation**: Early validation prevents runtime failures

### Performance Considerations
- **Indexing Speed**: Target <2s for 1000 pages
- **Memory Usage**: Monitor with Rust allocator profiling
- **Bundle Size**: Pagefind generates optimized WASM bundles
- **Search Latency**: Client-side search with <100ms response time

### Development Workflow
- **Documentation Files**: `@memory.md`, `@scratchpad.md`, `@lessons-learned.md`
- **Test-First**: Write tests before implementation
- **Benchmarking**: Measure performance throughout development
- **Feature Parity**: Ensure WASM and native builds have identical functionality

### Common Pitfalls to Avoid
- **Directory Changes**: Pagefind 1.x uses `pagefind/` not `_pagefind/`
- **WASM Compatibility**: Not all Rust crates work in WASM
- **CSP Violations**: WebAssembly needs proper security policy setup
- **Async Blocking**: Don't block async runtimes with sync operations
- **Test Isolation**: Clean up temporary files and state between tests