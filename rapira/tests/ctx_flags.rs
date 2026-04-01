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
    assert_eq!(
        rapira::size(&msg),
        rapira::size_ctx(&msg, RapiraFlags::NONE)
    );
}
