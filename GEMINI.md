# Rust Scanner Workspace

## Project Overview
This project is a high-performance, multi-threaded file scanning tool written in Rust. It leverages a Cargo Workspace architecture to separate the core scanning engine from the user interface. The tool allows users to scan directories (or the entire filesystem) using regular expressions, with built-in patterns for Taiwanese ID numbers and top 10 surnames.

The workspace consists of two main crates:
1.  **`scanner-core` (Library)**: The core scanning engine. It utilizes the `ignore` crate for fast, multi-threaded directory traversal (respecting `.gitignore` and automatically skipping non-UTF-8 files) and the `regex` crate for pattern matching. It uses a single-buffer read strategy to minimize memory allocations.
2.  **`rust-scanner-cli` (Binary)**: A Terminal User Interface (TUI) built with `ratatui` and `crossterm`. It provides an interactive way for users to select built-in regex patterns, input custom patterns with real-time error validation, and specify target directories for scanning.

## Building and Running

Ensure you have Rust and Cargo installed (edition 2024).

*   **To run the TUI application:**
    Navigate to the workspace directory and run the CLI project:
    ```bash
    cd rust-scanner-workspace
    cargo run --bin rust-scanner-cli
    ```
*   **To build the workspace (Debug):**
    ```bash
    cd rust-scanner-workspace
    cargo build
    ```
*   **To build for Release (optimized performance):**
    ```bash
    cd rust-scanner-workspace
    cargo build --release
    ```
*   **To run code checks and linting:**
    ```bash
    cd rust-scanner-workspace
    cargo check
    cargo clippy
    cargo fmt
    ```

## Architecture & Integration

*   **Workspace Structure**: The root directory should ideally be kept clean of top-level `Cargo.toml` or source code files outside of the `rust-scanner-workspace` folder to maintain organizational clarity.
*   **Core Extraction**: The scanning logic (`scanner-core`) is completely decoupled from the UI. Other Rust applications (e.g., Web backends, GUI apps) can depend on it directly.
*   **Dependency Usage**:
    *   To use `scanner-core` in another local project, add the following to that project's `Cargo.toml`:
        ```toml
        scanner-core = { path = "../path/to/rust-scanner-workspace/scanner-core" }
        ```

## Development Conventions

*   **Rust Best Practices**: Follow standard Rust idioms. Prioritize Borrowing over Ownership where applicable, and handle errors gracefully using `Result<T, E>`.
*   **Error Handling in UI**: The CLI should not panic. TUI components must catch errors (like invalid Regex input) and display them interactively within the UI layout (e.g., in red text) rather than silently failing or crashing.
*   **Memory Efficiency**: The `scanner-core` should maintain its optimized single-buffer reading strategy (`line_buf.clear()`) inside loops instead of allocating new `String` instances per line to prevent GC overhead when scanning large codebases or logs.
*   **Formatting**: Always run `cargo fmt` before committing code. Code style is enforced by standard `rustfmt` rules.
