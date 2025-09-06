# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MD-book is a modern mdbook replacement written in Rust that generates beautiful HTML documentation from Markdown files. It supports multiple markdown formats (Markdown, GFM, MDX) with server-side syntax highlighting, live development server, and integrated search functionality via Pagefind.

## Common Development Commands

### Build and Run
```bash
# Basic build - converts markdown to HTML
cargo run -- -i input_dir -o output_dir

# Development mode with file watching
cargo run -- -i input_dir -o output_dir --watch

# Development with built-in server
cargo run -- -i input_dir -o output_dir --serve

# Full development mode (watch + serve on custom port)
cargo run -- -i input_dir -o output_dir --watch --serve --port 8080
```

### Testing and Quality
```bash
# Run tests
cargo test

# Check code
cargo check

# Format code
cargo fmt

# Run clippy
cargo clippy
```

## Architecture Overview

### Core Components

- **main.rs**: Entry point handling CLI arguments, coordinating build process, and managing file watching/serving modes
- **config.rs**: Configuration system using `twelf` crate supporting TOML, JSON, YAML with environment variable overrides
- **server.rs**: Development server using `warp` with WebSocket-based live reload functionality
- **pagefind_service.rs**: Search functionality integration using the Pagefind search engine
- **templates/**: Tera template system for HTML generation with CSS/JS assets

### Key Dependencies

- **markdown**: Core markdown parsing with support for multiple formats (markdown, GFM, MDX)
- **tera**: Template engine for HTML generation
- **syntect**: Server-side syntax highlighting for code blocks
- **pagefind**: Full-text search engine integration
- **warp**: Web server for development mode
- **notify**: File system watching for auto-rebuild
- **twelf**: Configuration management with multiple source support

### Build Process Flow

1. **Configuration Loading**: Loads book.toml with environment variable and CLI overrides
2. **Template Setup**: Initializes Tera templates (uses embedded defaults or custom from src/templates/)
3. **Asset Copying**: Copies CSS, JS, images, and components to output directory
4. **Markdown Processing**: 
   - Walks input directory for .md files
   - Extracts titles and builds navigation structure
   - Processes each file with syntax highlighting
   - Generates HTML using Tera templates
5. **Search Indexing**: Runs Pagefind to create search index
6. **Development Features**: Optional file watching and live server with WebSocket reload

### Configuration System

The project uses a layered configuration approach (priority order):
1. Command line arguments (highest)
2. Environment variables (MDBOOK_ prefix)
3. Custom config file (if provided)
4. Default book.toml
5. Default values (lowest)

Supports TOML, JSON, and YAML configuration files with shell expansion for paths.

## Development Guidelines

### Rust & Async Programming Expert Guidelines

You are an expert in Rust, async programming, concurrent systems, and WASM. Follow these principles:

#### Key Principles
- Write clear, concise, and idiomatic Rust code with accurate examples
- Use async programming paradigms effectively, leveraging `tokio` for concurrency
- Prioritize modularity, clean code organization, and efficient resource management
- Use expressive variable names that convey intent (e.g., `is_ready`, `has_data`)
- Adhere to Rust naming conventions: snake_case for variables/functions, PascalCase for types/structs
- Avoid code duplication; use functions and modules to encapsulate reusable logic
- Write code with safety, concurrency, and performance in mind, embracing Rust's ownership and type system

#### Time Handling
- **ALWAYS use `jiff` instead of `chrono`** for all date/time operations
- Current project already uses `jiff` (see dependencies in Cargo.toml)

#### Async Programming
- Use `tokio` as the async runtime for handling asynchronous tasks and I/O
- Implement async functions using `async fn` syntax
- Leverage `tokio::spawn` for task spawning and concurrency
- Use `tokio::select!` for managing multiple async tasks and cancellations
- Favor structured concurrency: prefer scoped tasks and clean cancellation paths
- Implement timeouts, retries, and backoff strategies for robust async operations

#### Channels and Concurrency
- Use `tokio::sync::mpsc` for asynchronous, multi-producer, single-consumer channels
- Use `tokio::sync::broadcast` for broadcasting messages to multiple consumers
- Implement `tokio::sync::oneshot` for one-time communication between tasks
- Prefer bounded channels for backpressure; handle capacity limits gracefully
- Use `tokio::sync::Mutex` and `tokio::sync::RwLock` for shared state across tasks, avoiding deadlocks

#### Error Handling and Safety
- Embrace Rust's Result and Option types for error handling
- Use `?` operator to propagate errors in async functions
- Implement custom error types using `thiserror` or `anyhow` for more descriptive errors
- Handle errors and edge cases early, returning errors where appropriate
- Use `.await` responsibly, ensuring safe points for context switching

#### Testing
- Write unit tests with `tokio::test` for async tests
- Use `tokio::time::pause` for testing time-dependent code without real delays
- Implement integration tests to validate async behavior and concurrency
- Use mocks and fakes for external dependencies in tests
- **Never use mocks in tests** (as per user's global preferences)

#### Performance Optimization
- Minimize async overhead; use sync code where async is not needed
- Use non-blocking operations and atomic data types for concurrency
- Avoid blocking operations inside async functions; offload to dedicated blocking threads if necessary
- Use `tokio::task::yield_now` to yield control in cooperative multitasking scenarios
- Optimize data structures and algorithms for async use, reducing contention and lock duration
- Use `tokio::time::sleep` and `tokio::time::interval` for efficient time-based operations
- **Ensure high performance of each component with benchmarks**

#### WASM Compatibility
- **Maintain feature parity between native and WASM targets**
- Structure code to be compatible with both native and WASM environments
- Use conditional compilation (`#[cfg(target_arch = "wasm32")]`) when needed
- Test both native and WASM builds regularly

#### Web Frameworks
- Use [salvo](https://salvo.rs/book/) for async web server applications
- Use `axum` as an alternative async web framework
- Leverage `hyper` or `reqwest` for async HTTP requests
- Use `tonic` for gRPC with async support

#### Key Conventions
1. Structure the application into modules: separate concerns like networking, database, and business logic
2. Use environment variables for configuration management
3. Ensure code is well-documented with inline comments and Rustdoc
4. Use `serde` for serialization/deserialization
5. Use `sqlx` or `tokio-postgres` for async database interactions

## Project Management

### Required Documentation Files
Maintain these files throughout all interactions:
- **@memory.md**: Interaction history and context
- **@scratchpad.md**: Active task management and progress tracking
- **@lessons-learned.md**: Knowledge retention and insights

These files must be kept up to date with progress and should be created if they don't exist.