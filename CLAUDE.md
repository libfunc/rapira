# CLAUDE.md - Rapira Codebase Guide

## Build & Test Commands
- Build: `cargo build`
- Test: `cargo test`
- Test specific: `cargo test test_name`
- Test a specific package: `cargo test -p rapira`
- Lint/Format: `cargo fmt`
- Check: `cargo check`

## Code Style Guidelines
- Edition: Rust 2024
- Import style: `imports_granularity = "Crate"` with `group_imports = "StdExternalCrate"` 
- Error handling: Uses thiserror for error types, with Result type aliases
- Naming: PascalCase for types/traits, snake_case for functions/variables
- Documentation: Document public APIs with doc comments, especially safety requirements
- Attributes: Use `#[rapira(...)]` for derive customization
- Safety: Document unsafe operations with safety comments
- Features: Conditional compilation via feature flags (`std`, `alloc`, etc.)
- Testing: Write tests in dedicated test modules/files

## Special Notes
- Serialization library similar to borsh and bincode
- Supports no_std environments with feature flags