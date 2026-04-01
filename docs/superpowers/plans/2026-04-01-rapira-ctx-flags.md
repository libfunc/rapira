# RapiraFlags Context-Aware Serialization — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `RapiraFlags` bitfield and `_ctx` methods to the `Rapira` trait so external crates can define context-dependent serialization behavior (e.g., encryption, compression) via flag constants.

**Architecture:** `RapiraFlags` is a `u64` bitfield added to `rapira/src/lib.rs`. Three new default methods (`convert_to_bytes_ctx`, `from_slice_ctx`, `size_ctx`) are added to the `Rapira` trait — they delegate to existing methods when not overridden. The derive macro (`rapira-derive`) generates these methods for structs and enums, passing `flags` through to all fields. Helper functions in `funcs.rs` provide `serialize_ctx`/`deserialize_ctx` entry points.

**Tech Stack:** Rust, proc-macro2/quote/syn (derive macro)

**Spec:** `docs/superpowers/specs/2026-04-01-rapira-ctx-flags-design.md`

---

### Task 1: Add `RapiraFlags` type and trait methods to rapira

**Files:**
- Modify: `rapira/src/lib.rs`

- [ ] **Step 1: Add `RapiraFlags` struct to `lib.rs`**

Add after the existing `pub use` block (before `pub trait Rapira`), at line 32:

```rust
/// Bitflags for context-aware serialization.
/// Bits 0–7 are reserved for rapira. External crates use bits 8+.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct RapiraFlags(pub u64);

impl RapiraFlags {
    pub const NONE: Self = Self(0);

    #[inline]
    pub const fn new(flags: u64) -> Self {
        Self(flags)
    }

    #[inline]
    pub const fn has(self, flag: u64) -> bool {
        self.0 & flag != 0
    }

    #[inline]
    pub const fn with(self, flag: u64) -> Self {
        Self(self.0 | flag)
    }
}
```

- [ ] **Step 2: Add three default methods to the `Rapira` trait**

Add these methods inside `pub trait Rapira { ... }`, after the existing `convert_to_bytes` method (line 147):

```rust
    /// Context-aware serialization. Default: delegates to `convert_to_bytes`.
    #[inline]
    fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, _flags: RapiraFlags) {
        self.convert_to_bytes(slice, cursor)
    }

    /// Context-aware deserialization. Default: delegates to `from_slice`.
    #[inline]
    fn from_slice_ctx(slice: &mut &[u8], _flags: RapiraFlags) -> Result<Self>
    where
        Self: Sized,
    {
        Self::from_slice(slice)
    }

    /// Context-aware size calculation. Default: delegates to `size`.
    #[inline]
    fn size_ctx(&self, _flags: RapiraFlags) -> usize {
        self.size()
    }
```

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/railka/libfunc/rapira && cargo check -p rapira`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira/src/lib.rs
git commit -m "feat(rapira): add RapiraFlags type and _ctx methods to Rapira trait"
```

---

### Task 2: Add helper functions to `funcs.rs`

**Files:**
- Modify: `rapira/src/funcs.rs`
- Modify: `rapira/src/lib.rs` (re-exports)

- [ ] **Step 1: Add `_ctx` helper functions to `funcs.rs`**

Add at the end of `rapira/src/funcs.rs`:

```rust
use crate::RapiraFlags;

#[inline]
pub fn size_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> usize {
    match T::STATIC_SIZE {
        Some(s) => s,
        None => item.size_ctx(flags),
    }
}

/// Serialize with context flags.
#[cfg(feature = "alloc")]
pub fn serialize_ctx<T: Rapira>(item: &T, flags: RapiraFlags) -> Vec<u8> {
    let value_size = size_ctx(item, flags);
    let mut bytes: Vec<u8> = vec![0u8; value_size];
    item.convert_to_bytes_ctx(&mut bytes, &mut 0, flags);
    bytes
}

/// Deserialize with context flags.
pub fn deserialize_ctx<T: Rapira + Sized>(mut bytes: &[u8], flags: RapiraFlags) -> Result<T> {
    T::from_slice_ctx(&mut bytes, flags)
}
```

- [ ] **Step 2: Add re-exports in `lib.rs`**

In `rapira/src/lib.rs`, update the existing `pub use funcs::` block. Change:

```rust
pub use funcs::{
    check_bytes, deser_unchecked, deser_unsafe, deserialize, deserialize_versioned, size,
};
#[cfg(feature = "alloc")]
pub use funcs::{extend_vec, serialize};
```

to:

```rust
pub use funcs::{
    check_bytes, deser_unchecked, deser_unsafe, deserialize, deserialize_ctx,
    deserialize_versioned, size, size_ctx,
};
#[cfg(feature = "alloc")]
pub use funcs::{extend_vec, serialize, serialize_ctx};
```

- [ ] **Step 3: Verify it compiles**

Run: `cd /Users/railka/libfunc/rapira && cargo check -p rapira`
Expected: compiles with no errors

- [ ] **Step 4: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira/src/funcs.rs rapira/src/lib.rs
git commit -m "feat(rapira): add serialize_ctx/deserialize_ctx/size_ctx helper functions"
```

---

### Task 3: Generate `_ctx` methods in derive macro for named structs

**Files:**
- Modify: `rapira-derive/src/structs.rs`

The pattern mirrors existing code generation. For each field, we add `_ctx` variants that pass `flags` through. Two code paths: fields with `#[rapira(with = path)]` and fields without.

- [ ] **Step 1: Add new Vec declarations in named struct branch**

In `struct_serializer`, inside the `Fields::Named` match arm (around line 82-91), add three new Vec declarations alongside the existing ones:

```rust
            let mut convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(named_len);
            let mut size_ctx: Vec<TokenStream> = Vec::with_capacity(named_len);
```

- [ ] **Step 2: Populate the new Vecs in the per-field loop (with_attr branch)**

Inside the `for (field, _, with_attr, since) in fields_insert.iter()` loop, inside `match with_attr { Some(with_attr) => { ... } }` (the `with` attribute branch), add after the existing `convert_to_bytes` push:

```rust
                        convert_to_bytes_ctx.push(quote! {
                            #with_attr::convert_to_bytes_ctx(&self.#ident, slice, cursor, flags);
                        });
                        from_slice_ctx.push(quote! {
                            let #ident: #typ = #with_attr::from_slice_ctx(slice, flags)?;
                        });
                        size_ctx.push(quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                            Some(s) => s,
                            None => #with_attr::size_ctx(&self.#ident, flags)
                        }) });
```

- [ ] **Step 3: Populate the new Vecs in the per-field loop (no with_attr branch)**

Inside `match with_attr { None => { ... } }`, add after the existing `convert_to_bytes` push:

```rust
                        convert_to_bytes_ctx.push(quote! {
                            self.#ident.convert_to_bytes_ctx(slice, cursor, flags);
                        });
                        from_slice_ctx.push(quote! {
                            let #ident = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                        });
                        size_ctx.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                            Some(s) => s,
                            None => self.#ident.size_ctx(flags)
                        }) });
```

- [ ] **Step 4: Add generated methods to the final `quote!` block**

In the `let res = quote! { #name_with_generics { ... } };` block (around line 267), add three new methods after the existing `convert_to_bytes` method:

```rust
                    #[inline]
                    fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: rapira::RapiraFlags) {
                        #(#convert_to_bytes_ctx)*
                    }

                    #[inline]
                    fn from_slice_ctx(slice: &mut &[u8], flags: rapira::RapiraFlags) -> rapira::Result<Self>
                    where
                        Self: Sized,
                    {
                        #(#from_slice_ctx)*
                        Ok(#name {
                            #(#field_names)*
                        })
                    }

                    #[inline]
                    fn size_ctx(&self, flags: rapira::RapiraFlags) -> usize {
                        0 #(#size_ctx)*
                    }
```

- [ ] **Step 5: Add the same for unnamed struct branch**

In the `Fields::Unnamed` match arm (starts around line 337), apply the same pattern:

1. Add three Vec declarations:
```rust
            let mut convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
            let mut size_ctx: Vec<TokenStream> = Vec::with_capacity(unnamed_len);
```

2. In the per-field loop, for `Some(with_attr)` branch, add:
```rust
                        convert_to_bytes_ctx.push(quote! {
                            #with_attr::convert_to_bytes_ctx(&self.#id, slice, cursor, flags);
                        });
                        from_slice_ctx.push(quote! {
                            let #field_name: #typ = #with_attr::from_slice_ctx(slice, flags)?;
                        });
                        size_ctx.push(quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                            Some(s) => s,
                            None => #with_attr::size_ctx(&self.#id, flags)
                        }) });
```

3. In the per-field loop, for `None` branch, add:
```rust
                        convert_to_bytes_ctx.push(quote! {
                            self.#id.convert_to_bytes_ctx(slice, cursor, flags);
                        });
                        from_slice_ctx.push(quote! {
                            let #field_name = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                        });
                        size_ctx.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                            Some(s) => s,
                            None => self.#id.size_ctx(flags)
                        }) });
```

4. In the final `quote!` block for unnamed structs, add the three methods (same signatures, with `#name(#(#field_names)*)` for the constructor).

- [ ] **Step 6: Verify it compiles**

Run: `cd /Users/railka/libfunc/rapira && cargo check -p rapira-derive`
Expected: compiles with no errors

- [ ] **Step 7: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira-derive/src/structs.rs
git commit -m "feat(rapira-derive): generate _ctx methods for structs"
```

---

### Task 4: Generate `_ctx` methods in derive macro for enums

**Files:**
- Modify: `rapira-derive/src/enums.rs`
- Modify: `rapira-derive/src/simple_enum.rs`

The enum serializer has three field types: Unit, Unnamed, Named. For each variant with fields, we pass `flags` through like structs. Simple enums (no fields) use the default trait methods (no override needed, but we can skip generating for them).

- [ ] **Step 1: Add top-level Vec declarations in `enum_serializer`**

In `enum_serializer` function (line 22-34 area), add alongside existing Vecs:

```rust
    let mut convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut size_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
```

- [ ] **Step 2: Handle Unit variants**

Inside the `Fields::Unit` match arm, add:

```rust
                convert_to_bytes_ctx.push(quote! {
                    #name::#variant_name => {
                        rapira::push(slice, cursor, #variant_id);
                    }
                });
                from_slice_ctx.push(quote! {
                    #variant_id => {
                        Ok(#name::#variant_name)
                    }
                });
                size_ctx.push(quote! {
                    #name::#variant_name => 0,
                });
```

- [ ] **Step 3: Handle Unnamed variants**

Inside the `Fields::Unnamed` match arm, add per-field Vecs:

```rust
                let mut fields_convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_size_ctx: Vec<TokenStream> = Vec::with_capacity(len);
```

In the per-field loop, for `Some(with_attr)`:
```rust
                            fields_convert_to_bytes_ctx.push(quote! {
                                #with_attr::convert_to_bytes_ctx(#field_name, slice, cursor, flags);
                            });
                            fields_from_slice_ctx.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_ctx(slice, flags)?;
                            });
                            fields_size_ctx.push(quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                Some(s) => s,
                                None => #with_attr::size_ctx(#field_name, flags)
                            }) });
```

For `None`:
```rust
                            fields_convert_to_bytes_ctx.push(quote! {
                                #field_name.convert_to_bytes_ctx(slice, cursor, flags);
                            });
                            fields_from_slice_ctx.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                            });
                            fields_size_ctx.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                                Some(s) => s,
                                None => #field_name.size_ctx(flags)
                            }) });
```

After the per-field loop, push variant-level entries:
```rust
                convert_to_bytes_ctx.push(quote! {
                    #name::#variant_name(#(#field_names)*) => {
                        rapira::push(slice, cursor, #variant_id);
                        #(#fields_convert_to_bytes_ctx)*
                    }
                });
                from_slice_ctx.push(quote! {
                    #variant_id => {
                        #(#fields_from_slice_ctx)*
                        Ok(#name::#variant_name(#(#field_names)*))
                    }
                });
                size_ctx.push(quote! {
                    #name::#variant_name(#(#field_names)*) => {
                        0 #(#fields_size_ctx)*
                    },
                });
```

- [ ] **Step 4: Handle Named variants**

Same pattern as Unnamed but with `{ #field_names }` syntax. Inside `Fields::Named`, add per-field Vecs and populate them identically to step 3 but using the named field pattern.

Per-field Vecs:
```rust
                let mut fields_convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut fields_size_ctx: Vec<TokenStream> = Vec::with_capacity(len);
```

For `Some(with_attr)`:
```rust
                            fields_convert_to_bytes_ctx.push(quote! {
                                #with_attr::convert_to_bytes_ctx(#field_name, slice, cursor, flags);
                            });
                            fields_from_slice_ctx.push(quote! {
                                let #field_name: #typ = #with_attr::from_slice_ctx(slice, flags)?;
                            });
                            fields_size_ctx.push(quote! { + (match #with_attr::static_size(core::marker::PhantomData::<#typ>) {
                                Some(s) => s,
                                None => #with_attr::size_ctx(#field_name, flags)
                            }) });
```

For `None`:
```rust
                            fields_convert_to_bytes_ctx.push(quote! {
                                #field_name.convert_to_bytes_ctx(slice, cursor, flags);
                            });
                            fields_from_slice_ctx.push(quote! {
                                let #field_name = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                            });
                            fields_size_ctx.push(quote! { + (match <#typ as rapira::Rapira>::STATIC_SIZE {
                                Some(s) => s,
                                None => #field_name.size_ctx(flags)
                            }) });
```

After the per-field loop, push variant-level entries (with `{ }` syntax):
```rust
                convert_to_bytes_ctx.push(quote! {
                    #name::#variant_name{#(#field_names)*} => {
                        rapira::push(slice, cursor, #variant_id);
                        #(#fields_convert_to_bytes_ctx)*
                    }
                });
                from_slice_ctx.push(quote! {
                    #variant_id => {
                        #(#fields_from_slice_ctx)*
                        Ok(#name::#variant_name{#(#field_names)*})
                    }
                });
                size_ctx.push(quote! {
                    #name::#variant_name{#(#field_names)*} => {
                        0 #(#fields_size_ctx)*
                    },
                });
```

- [ ] **Step 5: Add methods to the final `quote!` block in `enum_serializer`**

In the `let res = quote! { ... }` block (around line 495), add after the `convert_to_bytes` method:

```rust
            #[inline]
            fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: rapira::RapiraFlags) {
                match self {
                    #(#convert_to_bytes_ctx)*
                }
            }

            #[inline]
            fn from_slice_ctx(slice: &mut &[u8], flags: rapira::RapiraFlags) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                match val {
                    #(#from_slice_ctx)*
                    _ => Err(rapira::RapiraError::EnumVariant),
                }
            }

            #[inline]
            fn size_ctx(&self, flags: rapira::RapiraFlags) -> usize {
                1 + match self {
                    #(#size_ctx)*
                }
            }
```

- [ ] **Step 6: Verify it compiles**

Run: `cd /Users/railka/libfunc/rapira && cargo check -p rapira-derive`
Expected: compiles with no errors

Note: `simple_enum.rs` does NOT need changes — simple enums have no fields (just a u8 tag), so the default trait `_ctx` methods (which delegate to `from_slice`/`convert_to_bytes`) are correct.

- [ ] **Step 7: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira-derive/src/enums.rs
git commit -m "feat(rapira-derive): generate _ctx methods for enums"
```

---

### Task 4b: Generate `_ctx` methods in derive macro for enum_with_primitive

**Files:**
- Modify: `rapira-derive/src/enum_with_primitive.rs`

The `enum_with_primitive` serializer handles enums with a `#[primitive(PrimName)]` attribute. It generates the same Rapira methods as `enums.rs`, but uses a primitive enum for the tag dispatch. Fields need `flags` propagation.

- [ ] **Step 1: Add top-level Vec declarations**

In `enum_with_primitive_serializer`, add alongside existing Vecs (after line 25):

```rust
    let mut convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
    let mut size_ctx: Vec<TokenStream> = Vec::with_capacity(variants_len);
```

- [ ] **Step 2: Handle Unit variants**

Inside `Fields::Unit`, add:

```rust
                convert_to_bytes_ctx.push(quote! {
                    #name::#variant_name => {}
                });
                from_slice_ctx.push(quote! {
                    #primitive_name::#variant_name => {
                        Ok(#name::#variant_name)
                    }
                });
                size_ctx.push(quote! {
                    #name::#variant_name => 0,
                });
```

- [ ] **Step 3: Handle single-field Unnamed variants**

Inside `Fields::Unnamed` where `len == 1`, add:

```rust
                    convert_to_bytes_ctx.push(quote! {
                        #name::#variant_name(v) => {
                            v.convert_to_bytes_ctx(slice, cursor, flags);
                        }
                    });
                    from_slice_ctx.push(quote! {
                        #primitive_name::#variant_name => {
                            let v = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                            Ok(#name::#variant_name(v))
                        }
                    });
                    size_ctx.push(quote! {
                        #name::#variant_name(v) => {
                            match <#typ>::STATIC_SIZE {
                                Some(s) => s,
                                None => v.size_ctx(flags),
                            }
                        },
                    });
```

- [ ] **Step 4: Handle multi-field Unnamed variants**

Inside the `else` branch (multi-field unnamed), add per-field Vecs and populate them:

```rust
                    let mut unnamed_convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                    let mut unnamed_size_ctx: Vec<TokenStream> = Vec::with_capacity(len);
```

In the per-field loop, add:
```rust
                        unnamed_convert_to_bytes_ctx.push(quote! {
                            #field_name.convert_to_bytes_ctx(slice, cursor, flags);
                        });
                        unnamed_from_slice_ctx.push(quote! {
                            let #field_name = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                        });
                        unnamed_size_ctx.push(quote! { + (match <#typ>::STATIC_SIZE {
                            Some(s) => s,
                            None => #field_name.size_ctx(flags)
                        }) });
```

After the per-field loop, push variant-level entries:
```rust
                    convert_to_bytes_ctx.push(quote! {
                        #name::#variant_name(#(#field_names)*) => {
                            #(#unnamed_convert_to_bytes_ctx)*
                        }
                    });
                    from_slice_ctx.push(quote! {
                        #primitive_name::#variant_name => {
                            #(#unnamed_from_slice_ctx)*
                            Ok(#name::#variant_name(#(#field_names)*))
                        }
                    });
                    size_ctx.push(quote! {
                        #name::#variant_name(#(#field_names)*) => {
                            0 #(#unnamed_size_ctx)*
                        },
                    });
```

- [ ] **Step 5: Handle Named variants**

Same pattern. Add per-field Vecs:
```rust
                let mut named_convert_to_bytes_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_from_slice_ctx: Vec<TokenStream> = Vec::with_capacity(len);
                let mut named_size_ctx: Vec<TokenStream> = Vec::with_capacity(len);
```

In the per-field loop:
```rust
                    named_convert_to_bytes_ctx.push(quote! {
                        #field_name.convert_to_bytes_ctx(slice, cursor, flags);
                    });
                    named_from_slice_ctx.push(quote! {
                        let #field_name = <#typ as rapira::Rapira>::from_slice_ctx(slice, flags)?;
                    });
                    named_size_ctx.push(quote! { + (match <#typ>::STATIC_SIZE {
                        Some(s) => s,
                        None => #field_name.size_ctx(flags)
                    }) });
```

After the per-field loop:
```rust
                convert_to_bytes_ctx.push(quote! {
                    #name::#variant_name{#(#field_names)*} => {
                        #(#named_convert_to_bytes_ctx)*
                    }
                });
                from_slice_ctx.push(quote! {
                    #primitive_name::#variant_name => {
                        #(#named_from_slice_ctx)*
                        Ok(#name::#variant_name{#(#field_names)*})
                    }
                });
                size_ctx.push(quote! {
                    #name::#variant_name{#(#field_names)*} => {
                        0 #(#named_size_ctx)*
                    },
                });
```

- [ ] **Step 6: Add methods to the final `quote!` block**

In the `let res = quote! { impl rapira::Rapira for #name { ... } }` block, add after `convert_to_bytes`:

```rust
            #[inline]
            fn convert_to_bytes_ctx(&self, slice: &mut [u8], cursor: &mut usize, flags: rapira::RapiraFlags) {
                let t = #primitive_name::from(self) as u8;
                rapira::push(slice, cursor, t);
                match self {
                    #(#convert_to_bytes_ctx)*
                }
            }

            #[inline]
            fn from_slice_ctx(slice: &mut &[u8], flags: rapira::RapiraFlags) -> rapira::Result<Self>
            where
                Self: Sized,
            {
                let val: u8 = rapira::byte_rapira::from_slice(slice)?;
                let t = <#primitive_name as TryFrom<u8>>::try_from(val).map_err(|_| rapira::RapiraError::EnumVariant)?;
                match t {
                    #(#from_slice_ctx)*
                }
            }

            #[inline]
            fn size_ctx(&self, flags: rapira::RapiraFlags) -> usize {
                1 + match self {
                    #(#size_ctx)*
                }
            }
```

- [ ] **Step 7: Verify it compiles**

Run: `cd /Users/railka/libfunc/rapira && cargo check -p rapira-derive`
Expected: compiles with no errors

- [ ] **Step 8: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira-derive/src/enum_with_primitive.rs
git commit -m "feat(rapira-derive): generate _ctx methods for enum_with_primitive"
```

---

### Task 5: Run existing tests (regression check)

**Files:** none modified

- [ ] **Step 1: Run full test suite**

Run: `cd /Users/railka/libfunc/rapira && cargo test`
Expected: all existing tests pass with no failures

- [ ] **Step 2: Run clippy**

Run: `cd /Users/railka/libfunc/rapira && cargo clippy`
Expected: no new warnings

---

### Task 6: Write new tests

**Files:**
- Create: `rapira/tests/ctx_flags.rs`

- [ ] **Step 1: Create the test file with all tests**

Create `rapira/tests/ctx_flags.rs`:

```rust
use rapira::{Rapira, RapiraFlags};

// --- RapiraFlags unit tests ---

#[test]
fn ctx_flags_none_is_zero() {
    assert_eq!(RapiraFlags::NONE.0, 0);
    assert_eq!(RapiraFlags::default(), RapiraFlags::NONE);
}

#[test]
fn ctx_flags_operations() {
    let flags = RapiraFlags::NONE.with(1).with(4);
    assert!(flags.has(1));
    assert!(flags.has(4));
    assert!(!flags.has(2));
    assert!(!flags.has(8));
}

#[test]
fn ctx_flags_with_is_or() {
    let a = RapiraFlags::new(0b0101);
    let b = a.with(0b1010);
    assert_eq!(b.0, 0b1111);
}

// --- Default _ctx delegates to plain methods ---

#[test]
fn ctx_default_same_as_plain_u32() {
    let val: u32 = 42;
    let plain = rapira::serialize(&val);
    let ctx = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    assert_eq!(plain, ctx);

    let decoded: u32 = rapira::deserialize_ctx(&ctx, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, val);
}

#[test]
fn ctx_default_same_as_plain_string() {
    let val = String::from("hello rapira");
    let plain = rapira::serialize(&val);
    let ctx = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    assert_eq!(plain, ctx);
}

// --- Derived struct tests ---

#[derive(rapira::Rapira, Debug, PartialEq)]
struct Msg {
    x: u32,
    name: String,
}

#[test]
fn ctx_default_struct() {
    let msg = Msg {
        x: 1,
        name: "hello".into(),
    };
    let plain = rapira::serialize(&msg);
    let ctx = rapira::serialize_ctx(&msg, RapiraFlags::NONE);
    assert_eq!(plain, ctx);

    let decoded: Msg = rapira::deserialize_ctx(&ctx, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, msg);
}

#[derive(rapira::Rapira, Debug, PartialEq)]
struct Pair(u32, u64);

#[test]
fn ctx_default_unnamed_struct() {
    let pair = Pair(10, 20);
    let plain = rapira::serialize(&pair);
    let ctx = rapira::serialize_ctx(&pair, RapiraFlags::NONE);
    assert_eq!(plain, ctx);

    let decoded: Pair = rapira::deserialize_ctx(&ctx, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, pair);
}

// --- Derived enum tests ---

#[derive(rapira::Rapira, Debug, PartialEq)]
enum Action {
    Ping,
    Send { data: Vec<u8> },
    Echo(String),
}

#[test]
fn ctx_roundtrip_enum_unit() {
    let val = Action::Ping;
    let bytes = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    let decoded: Action = rapira::deserialize_ctx(&bytes, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, val);
}

#[test]
fn ctx_roundtrip_enum_named() {
    let val = Action::Send {
        data: vec![1, 2, 3],
    };
    let plain = rapira::serialize(&val);
    let ctx = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    assert_eq!(plain, ctx);

    let decoded: Action = rapira::deserialize_ctx(&ctx, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, val);
}

#[test]
fn ctx_roundtrip_enum_unnamed() {
    let val = Action::Echo("test".into());
    let bytes = rapira::serialize_ctx(&val, RapiraFlags::NONE);
    let decoded: Action = rapira::deserialize_ctx(&bytes, RapiraFlags::NONE).unwrap();
    assert_eq!(decoded, val);
}

// --- Custom flag behavior test (REVERSE) ---

const REVERSE: u64 = 1;

struct RevU64(u64);

impl Rapira for RevU64 {
    const STATIC_SIZE: Option<usize> = Some(8);
    const MIN_SIZE: usize = 8;

    fn size(&self) -> usize {
        8
    }

    fn check_bytes(slice: &mut &[u8]) -> rapira::Result<()> {
        <[u8; 8] as Rapira>::check_bytes(slice)
    }

    fn from_slice(slice: &mut &[u8]) -> rapira::Result<Self> {
        let bytes = <[u8; 8] as Rapira>::from_slice(slice)?;
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
        let bytes = <[u8; 8] as Rapira>::from_slice(slice)?;
        if flags.has(REVERSE) {
            Ok(RevU64(u64::from_be_bytes(bytes)))
        } else {
            Ok(RevU64(u64::from_le_bytes(bytes)))
        }
    }
}

#[test]
fn ctx_reverse_flag_writes_be() {
    let val = RevU64(0x0102030405060708);

    // With REVERSE flag: writes BE bytes
    let be_bytes = rapira::serialize_ctx(&val, RapiraFlags::new(REVERSE));
    assert_eq!(be_bytes, [0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);

    // Without flag: writes LE bytes (default)
    let le_bytes = rapira::serialize(&val);
    assert_eq!(le_bytes, [0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
}

#[test]
fn ctx_reverse_flag_deserialize_without_flag_gives_raw() {
    let val = RevU64(0x0102030405060708);
    let be_bytes = rapira::serialize_ctx(&val, RapiraFlags::new(REVERSE));

    // Deserialize BE bytes WITHOUT flag: raw BE bytes read as LE
    let raw: RevU64 = rapira::deserialize(&be_bytes).unwrap();
    assert_eq!(raw.0, 0x0807060504030201);
}

#[test]
fn ctx_reverse_flag_roundtrip() {
    let val = RevU64(0x0102030405060708);
    let be_bytes = rapira::serialize_ctx(&val, RapiraFlags::new(REVERSE));

    // Deserialize BE bytes WITH flag: correct value
    let decoded: RevU64 = rapira::deserialize_ctx(&be_bytes, RapiraFlags::new(REVERSE)).unwrap();
    assert_eq!(decoded.0, 0x0102030405060708);
}

// --- size_ctx delegates correctly ---

#[test]
fn ctx_size_matches_plain_size() {
    let msg = Msg {
        x: 42,
        name: "test".into(),
    };
    assert_eq!(rapira::size(&msg), rapira::size_ctx(&msg, RapiraFlags::NONE));
}
```

- [ ] **Step 2: Run the new tests**

Run: `cd /Users/railka/libfunc/rapira && cargo test --test ctx_flags`
Expected: all tests pass

- [ ] **Step 3: Run full test suite (final regression check)**

Run: `cd /Users/railka/libfunc/rapira && cargo test`
Expected: all tests pass (old + new)

- [ ] **Step 4: Run clippy**

Run: `cd /Users/railka/libfunc/rapira && cargo clippy`
Expected: no warnings

- [ ] **Step 5: Commit**

```bash
cd /Users/railka/libfunc/rapira
git add rapira/tests/ctx_flags.rs
git commit -m "test: add ctx_flags tests for RapiraFlags and _ctx methods"
```
