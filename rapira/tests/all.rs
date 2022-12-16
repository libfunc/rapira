use byteorder::LE;
use rapira::*;
use zerocopy::{byteorder::U64, AsBytes, FromBytes};

#[test]
fn test_bool() {
    let bytes = true.serialize();
    assert_eq!(bytes.len(), 1);
    let val = bool::deserialize(&bytes).unwrap();
    assert!(val);
    let val = unsafe { bool::deser_unsafe(&bytes).unwrap() };
    assert!(val);
}

#[derive(Debug, Rapira, PartialEq)]
struct StructVecFields {
    vec: Vec<u8>,
    arr: [i32; 8],
    arr_bytes: [u8; 4],
    #[rapira(with = "rapira::byte_rapira")]
    byte: u8,
}

#[test]
fn test_vec_fields() -> Result<()> {
    let item = StructVecFields {
        vec: vec![0, 4, 6, 7],
        arr: [-1, 0, 4, 7, -8, 9, -1, 2],
        arr_bytes: [0, 1, 2, 3],
        byte: 4,
    };

    let vec = item.serialize();
    assert!(item == StructVecFields::deserialize(&vec)?);
    Ok(())
}

#[derive(Debug, Rapira, PartialEq)]
struct UnnamedFields(
    Vec<u8>,
    [i32; 8],
    [u8; 4],
    #[rapira(with = "rapira::byte_rapira")] u8,
);

#[test]
fn test_unnamed_fields() -> Result<()> {
    let item = UnnamedFields(
        vec![0, 4, 6, 7],
        [-1, 0, 4, 7, -8, 9, -1, 2],
        [0, 1, 2, 3],
        4,
    );

    let vec = item.serialize();
    assert!(item == UnnamedFields::deserialize(&vec)?);
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, AsBytes, FromBytes, PartialEq, Debug)]
struct Zero {
    a: [u8; 16],
    b: U64<LE>,
    c: u8,
}

#[derive(Debug, Rapira, PartialEq)]
struct ZeroOwned {
    #[rapira(with = "rapira::zero")]
    zero: Zero,
    other: String,
}

#[test]
fn test_zero() -> Result<()> {
    let zero = Zero {
        a: [3u8; 16],
        b: U64::<LE>::new(123123123123),
        c: 5,
    };
    let item = ZeroOwned {
        zero,
        other: String::from("asdasdas"),
    };
    let vec = item.serialize();
    assert!(item == ZeroOwned::deserialize(&vec)?);
    Ok(())
}

#[derive(Debug, Rapira, PartialEq)]
enum FullEnum {
    A(String),
    B(u16, u64, #[rapira(with = "rapira::zero")] Zero),
    C {
        c1: bool,
        c2: (usize, isize),
        #[rapira(with = "rapira::byte_rapira")]
        c3: u8,
        #[rapira(with = "rapira::zero")]
        c4: Zero,
    },
    D,
}

#[derive(Debug, Rapira, PartialEq)]
#[rapira(static_size = "None")]
enum NonStaticSized {
    A(String),
    B(Box<NonStaticSized>),
    C,
}

#[test]
fn test_enum() -> Result<()> {
    let zero = Zero {
        a: [3u8; 16],
        b: U64::<LE>::new(123123123123),
        c: 5,
    };
    let a = FullEnum::A("adasd".to_owned());
    let vec = a.serialize();
    assert!(a == FullEnum::deserialize(&vec)?);

    let b = FullEnum::B(12, 12312312321123, zero);
    let vec = b.serialize();
    assert!(b == FullEnum::deserialize(&vec)?);

    let c = FullEnum::C {
        c1: true,
        c2: (34534435354, -12312312312),
        c3: 7,
        c4: zero,
    };
    let vec = c.serialize();
    assert!(c == FullEnum::deserialize(&vec)?);

    let d = FullEnum::D;
    let vec = d.serialize();
    assert!(d == FullEnum::deserialize(&vec)?);

    let e = NonStaticSized::B(Box::new(NonStaticSized::A(String::from("asd"))));
    let vec = e.serialize();
    assert!(e == NonStaticSized::deserialize(&vec)?);

    Ok(())
}
