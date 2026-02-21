# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p rapira
cargo test -p rapira-derive

# Run a single test
cargo test test_bool
cargo test -p rapira test_vec_fields

# Format (uses rustfmt.toml: imports_granularity=Crate, group_imports=StdExternalCrate)
cargo fmt

# Check without building
cargo check
```

## Architecture

This is a Cargo workspace with two crates:

- **`rapira/`** — core runtime library with the `Rapira` trait and all implementations
- **`rapira-derive/`** — proc-macro crate providing `#[derive(Rapira)]`, `#[derive(FromU8)]`, `#[derive(PrimitiveFromEnum)]`

### Core Trait (`rapira/src/lib.rs`)

The `Rapira` trait has four core methods:
- `size(&self) -> usize` — serialized byte size
- `check_bytes(slice: &mut &[u8]) -> Result<()>` — validates bytes before deserialization (UTF-8, collection bounds, NonZero, float finiteness). Call this for untrusted external data.
- `from_slice(slice: &mut &[u8]) -> Result<Self>` — safe deserialization (checks capacity limits)
- `convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize)` — serialization into pre-allocated buffer

And `STATIC_SIZE: Option<usize>` — `Some(n)` for fixed-size types, `None` for dynamic. Used for compile-time optimization.

**Deserialization safety tiers:**
1. `from_slice` — safe, checks collection capacity limits (use for external data)
2. `from_slice_unchecked` — skips UTF-8/float/NonZero checks but still checks cursor bounds
3. `from_slice_unsafe` — fully unsafe, no bounds checks (only use after `check_bytes`)

**Top-level API functions** (`rapira/src/funcs.rs`):
- `serialize(item)` → `Vec<u8>`
- `deserialize(bytes)` → `Result<T>`
- `check_bytes::<T>(bytes)` → `Result<()>`
- `extend_vec(item, vec)` — append serialized bytes to existing Vec

### Wire Format

- Little-endian for all integers
- Lengths for collections/strings serialized as `u32` (4 bytes = `LEN_SIZE`)
- `usize` serialized as `u32`, `isize` as `i64`
- `f32`/`f64` must be finite (NaN/Inf returns error)
- `Option<T>`: 1 byte discriminant (0=None, 1=Some) + payload
- Enums: 1 byte discriminant + variant payload
- `Duration`: stored as seconds only (nanoseconds discarded)

### Derive Macro Logic (`rapira-derive/src/lib.rs`)

The `#[derive(Rapira)]` macro dispatches to:
- `struct_serializer` for structs
- `simple_enum_serializer` for unit-only enums
- `enum_with_primitive_serializer` for enums with `#[primitive(PrimType)]`
- `enum_serializer` for complex enums

**Available derive attributes:**
- `#[rapira(with = path)]` — use module-level functions (like `rapira::byte_rapira`) instead of `Rapira` impl
- `#[rapira(skip)]` — skip field (uses `Default::default()` on deserialization)
- `#[rapira(static_size = expr)]` / `#[rapira(min_size = expr)]` — override computed sizes
- `#[idx = N]` — manually set enum variant discriminant
- `#[primitive(PrimitiveName)]` — link complex enum to a unit enum for discriminant lookup
- `#[rapira(debug)]` — print generated code during compilation

### Module Layout (`rapira/src/`)

- `primitive.rs` — impls for primitives, arrays, tuples, Option, Result, `str_rapira`/`bytes_rapira`/`byte_rapira` modules
- `allocated.rs` — impls for `String`, `Vec`, `BTreeMap`, `Cow`, `IpAddr`, etc.
- `implements.rs` — impls for optional deps: `arrayvec`, `smallvec`, `bytes`, `fjall`, `zerocopy`, `postcard`, `serde_json`, `rust_decimal`, compact strings, Solana types
- `max_cap.rs` — capacity limits for DoS protection: `VEC_MAX_CAP=512k`, `VEC_MAX_SIZE_OF=2GB`
- `error.rs` — `RapiraError` enum and `Result<T>` type alias
- `from_u8.rs` — `FromU8` trait for enum-from-byte conversion
- `macros.rs` — helper macros (std-only)

### Feature Flags

Default features: `std`, `zerocopy`, `serde`/`serde_json`, `arrayvec`, `bytes`, `time`

Optional: `smallvec`, `indexmap`, `rust_decimal`, `compact_str`, `smol_str`, `ecow`, `uuid`, `bytemuck`, `inline-array`, `fjall`, `byteview`, `postcard`, `solana`, `rmp`

`no_std` support: disable default features, enable `alloc` if needed.
