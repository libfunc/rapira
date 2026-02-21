# Rapira

**Rapira** is a high-performance, safe serialization library for Rust, designed as an alternative to `borsh`, `bincode`, and `speedy`. It prioritizes compactness and speed while offering robust safety checks.

## Project Structure

The project is a Cargo workspace containing:

*   **`rapira/`**: The core runtime library. Defines the `Rapira` trait and implements serialization primitives.
*   **`rapira-derive/`**: Procedural macros for deriving the `Rapira` trait and other helper traits (`FromU8`, `PrimitiveFromEnum`).

## Key Concepts

### The `Rapira` Trait

The core of the library is the `Rapira` trait, which defines:

*   `size(&self) -> usize`: Returns the serialized size in bytes.
*   `check_bytes(slice: &mut &[u8]) -> Result<()>`: Validates the byte slice before deserialization (e.g., checking UTF-8 validity, collection lengths).
*   `from_slice(slice: &mut &[u8]) -> Result<Self>`: Deserializes the object from the byte slice.
*   `convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize)`: Serializes the object into the provided buffer.

### Features

*   **`no_std` Support**: core functionality is available without the standard library (disable default features).
*   **Safety**: Explicit `check_bytes` method to validate data before unsafe operations.
*   **Integrations**:
    *   `serde`: Optional support for `serde` serialization.
    *   `zerocopy`: efficient zero-copy deserialization.
    *   `solana`: Support for `solana-pubkey` and `solana-signature`.

## Building and Running

### Build

```bash
cargo build
```

### Test

Run the test suite:

```bash
cargo test
```

### Run Examples

Run the provided examples to see `Rapira` in action:

```bash
cargo run --example debug_struct
cargo run --example debug_enum
cargo run --example debug_combined
```

## Development Conventions

*   **Derive Macros**: Use `#[derive(Rapira)]` for structs and enums to automatically implement serialization logic.
*   **Attributes**:
    *   `#[rapira(static_size = ...)]`: optimize for fixed-size types.
    *   `#[rapira(min_size = ...)]`: assert minimum size.
    *   `#[rapira(skip)]`: skip fields during serialization.
    *   `#[idx = ...]`: manually specify enum variant indices.
    *   `#[primitive(Type)]`: for complex enums that map to a primitive type.
*   **Formatting**: ensure code is formatted with `cargo fmt`.

## Key Files

*   `rapira/src/lib.rs`: Defines the `Rapira` trait and core primitives.
*   `rapira-derive/src/lib.rs`: Implementation of the procedural macros.
*   `rapira/Cargo.toml`: Library dependencies and feature flags.
