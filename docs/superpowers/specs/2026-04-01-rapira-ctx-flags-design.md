# Rapira Context-Aware Serialization (RapiraFlags)

## Problem

armour-core defines ID types (`Fuid<H>`, `Id64<H>`) that need different serialization behavior depending on context:
- **DB storage**: raw bytes (current behavior)
- **Wire protocol to mobile client**: Blowfish-encrypted bytes

Currently rapira's `Rapira` trait has no mechanism for context-dependent serialization. The encryption keys are compile-time constants in armour-core (`IdHasher` trait), so they can't be shipped to mobile clients. The mobile client receives encrypted bytes and uses them as-is (opaque u64_be).

The mechanism must be **extensible from outside rapira** — external crates define their own flag constants and override `_ctx` methods for their types.

## Design

### Approach: Bitflags on methods

Add `RapiraFlags` (a `u64` bitfield) and new `_ctx` methods to the `Rapira` trait with default implementations that delegate to the existing methods. External crates define flag constants and override `_ctx` methods only for types that need context-dependent behavior.

**Why bitflags over alternatives:**
- **Associated type on Rapira**: doesn't work — different fields in a struct have different config needs, but a struct can only have one associated type
- **Generic `C: RapiraCtx` trait**: requires specialization (unstable) to have both `impl<H> Rapira for Fuid<H>` (generic) and `impl<H: IdHasher> Rapira for Fuid<H>` (specialized)
- **`&dyn Any` downcast**: safe and extensible but vtable lookup overhead on every field
- **Bitflags**: simple, zero-cost (compiler eliminates branches on constant flags), externally extensible (any crate defines new constants)

**Bit allocation convention:**
- Bits 0-7: reserved for rapira
- Bits 8+: available for external crates

### Changes

#### 1. `rapira/src/lib.rs` — RapiraFlags type + trait methods

```rust
/// Bitflags for context-aware serialization.
/// rapira reserves bits 0-7. External crates use bits 8+.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RapiraFlags(pub u64);

impl RapiraFlags {
    pub const NONE: Self = Self(0);

    #[inline]
    pub const fn new(flags: u64) -> Self { Self(flags) }

    #[inline]
    pub const fn has(self, flag: u64) -> bool { self.0 & flag != 0 }

    #[inline]
    pub const fn with(self, flag: u64) -> Self { Self(self.0 | flag) }
}
```

Three new methods on `Rapira` trait with default impls:

```rust
pub trait Rapira {
    // ... existing methods unchanged ...

    #[inline]
    fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, _flags: RapiraFlags) {
        self.convert_to_bytes(slice, cursor)
    }

    #[inline]
    fn from_slice_ctx(slice: &mut &[u8], _flags: RapiraFlags) -> Result<Self>
    where Self: Sized
    {
        Self::from_slice(slice)
    }

    #[inline]
    fn size_ctx(&self, _flags: RapiraFlags) -> usize {
        self.size()
    }
}
```

All existing types (`u32`, `String`, `Vec<T>`, `Option<T>`, etc.) use default impls — zero changes, zero cost.

#### 2. `rapira/src/funcs.rs` — helper functions

```rust
#[inline]
pub fn size_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> usize {
    match T::STATIC_SIZE {
        Some(s) => s,
        None => item.size_ctx(flags),
    }
}

#[cfg(feature = "alloc")]
pub fn serialize_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> Vec<u8> {
    let value_size = size_ctx(item, flags);
    let mut bytes = vec![0u8; value_size];
    item.convert_to_bytes_ctx(&mut bytes, &mut 0, flags);
    bytes
}

pub fn deserialize_ctx<T: Rapira>(bytes: &[u8], flags: RapiraFlags) -> Result<T> {
    let mut bytes = bytes;
    T::from_slice_ctx(&mut bytes, flags)
}
```

Re-export `RapiraFlags` from `lib.rs`.

#### 3. `rapira-derive/src/structs.rs` (and enums) — derive macro

For each field, generate `_ctx` method calls that pass `flags` through:

```rust
// Generated for struct Message { id: Fuid<H>, name: String }
fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: rapira::RapiraFlags) {
    self.id.convert_to_bytes_ctx(slice, cursor, flags);
    self.name.convert_to_bytes_ctx(slice, cursor, flags);
}

fn from_slice_ctx(slice: &mut &[u8], flags: rapira::RapiraFlags) -> rapira::Result<Self> {
    let id = <Fuid<H> as rapira::Rapira>::from_slice_ctx(slice, flags)?;
    let name = <String as rapira::Rapira>::from_slice_ctx(slice, flags)?;
    Ok(Message { id, name })
}

fn size_ctx(&self, flags: rapira::RapiraFlags) -> usize {
    0 + (match <Fuid<H> as rapira::Rapira>::STATIC_SIZE {
        Some(s) => s,
        None => self.id.size_ctx(flags)
    }) + (match <String as rapira::Rapira>::STATIC_SIZE {
        Some(s) => s,
        None => self.name.size_ctx(flags)
    })
}
```

Fields with `#[rapira(with = path)]` generate `path::convert_to_bytes_ctx(...)` etc. Enum variants use the same pattern — `flags` is passed into each variant's fields.

**Derive code changes:**
- New `Vec<TokenStream>` for `convert_to_bytes_ctx`, `from_slice_ctx`, `size_ctx`
- Each field appends the `_ctx` call (mirroring existing pattern)
- Final `quote!` block includes three new methods

#### 4. Tests

**Existing tests**: must pass unchanged. Default `_ctx` methods delegate to existing methods.

**New tests:**

```rust
#[test]
fn ctx_default_same_as_plain() {
    let val: u32 = 42;
    let plain = rapira::serialize(&val);
    let ctx = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    assert_eq!(plain, ctx);
}

#[test]
fn ctx_default_struct() {
    #[derive(Rapira, Debug, PartialEq)]
    struct Msg { x: u32, name: String }

    let msg = Msg { x: 1, name: "hello".into() };
    let plain = rapira::serialize(&msg);
    let ctx = rapira::serialize_ctx(&msg, RapiraFlags::NONE);
    assert_eq!(plain, ctx);

    let decoded: Msg = rapira::deserialize_ctx(&ctx, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, msg);
}

#[test]
fn ctx_flags_operations() {
    let flags = RapiraFlags::NONE.with(1).with(4);
    assert!(flags.has(1));
    assert!(flags.has(4));
    assert!(!flags.has(2));
}

#[test]
fn ctx_roundtrip_enum() {
    #[derive(Rapira, Debug, PartialEq)]
    enum Action { Ping, Send { data: Vec<u8> } }

    let val = Action::Send { data: vec![1, 2, 3] };
    let bytes = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    let decoded: Action = rapira::deserialize_ctx(&bytes, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, val);
}
```

**Flag behavior test** — custom type that reacts to a REVERSE flag:

```rust
const REVERSE: u64 = 1;

struct RevU64(u64);

impl Rapira for RevU64 {
    const STATIC_SIZE: Option<usize> = Some(8);
    const MIN_SIZE: usize = 8;
    fn size(&self) -> usize { 8 }
    fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()> {
        <[u8; 8] as Rapira>::check_bytes(slice)
    }
    fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self> {
        let bytes = <[u8; 8]>::from_slice(slice)?;
        Ok(RevU64(u64::from_le_bytes(bytes)))
    }
    fn convert_to_bytes(&self, slice: &mut [u8], cursor: &mut usize) {
        self.0.to_le_bytes().convert_to_bytes(slice, cursor)
    }
    fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: RapiraFlags) {
        if flags.has(REVERSE) {
            self.0.to_be_bytes().convert_to_bytes(slice, cursor)
        } else {
            self.convert_to_bytes(slice, cursor)
        }
    }
    fn from_slice_ctx(slice: &mut &[u8], flags: RapiraFlags) -> rapira::Result<Self> {
        let bytes = <[u8; 8]>::from_slice(slice)?;
        if flags.has(REVERSE) {
            Ok(RevU64(u64::from_be_bytes(bytes)))
        } else {
            Ok(RevU64(u64::from_le_bytes(bytes)))
        }
    }
}

#[test]
fn ctx_reverse_flag() {
    let val = RevU64(0x0102030405060708);

    // With REVERSE flag: writes BE
    let be_bytes = rapira::serialize_ctx(&val, RapiraFlags::new(REVERSE));
    assert_eq!(be_bytes, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    // Without flag: writes LE
    let le_bytes = rapira::serialize(&val);
    assert_eq!(le_bytes, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);

    // Deserialize BE bytes WITHOUT flag: raw BE bytes read as LE
    let raw: RevU64 = rapira::deserialize(&be_bytes).unwrap();
    assert_eq!(raw.0, 0x0807060504030201);

    // Deserialize BE bytes WITH flag: correct value
    let decoded: RevU64 = rapira::deserialize_ctx(&be_bytes, RapiraFlags::new(REVERSE)).unwrap();
    assert_eq!(decoded.0, 0x0102030405060708);
}
```

## Scope

This spec covers **rapira and rapira-derive only**. Integration into armour-core (overriding `_ctx` for `Fuid`/`Id64`, defining `RAPIRA_ENCRYPT` flag, adding `cipher` feature flag) is a separate task.

## Future: armour-core integration (out of scope)

For context on how this will be used:

```rust
// armour-core defines:
pub const RAPIRA_ENCRYPT: u64 = 1 << 8; // bit 8, outside rapira's reserved range

// Fuid overrides _ctx methods:
impl<H: IdHasher> Rapira for Fuid<H> {
    fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: RapiraFlags) {
        if flags.has(RAPIRA_ENCRYPT) {
            let encrypted = H::HASHER.0.encrypt_u64(self.get());
            encrypted.to_le_bytes().convert_to_bytes(slice, cursor);
        } else {
            self.convert_to_bytes(slice, cursor)
        }
    }
    // ...
}
```

The mobile client compiles armour-core without `cipher` feature — `Fuid<H>` uses default `_ctx` impl (pass-through). Server compiles with `cipher` — `Fuid<H: IdHasher>` encrypts/decrypts based on flags.
