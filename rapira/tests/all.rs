use rapira::*;
use serde::{Deserialize, Serialize};
use zerocopy::{
    FromBytes, Immutable, IntoBytes, KnownLayout,
    byteorder::{LE, U64},
};

#[test]
fn test_bool() {
    let bytes = serialize(&true);
    assert_eq!(bytes.len(), 1);
    let val: bool = deserialize(&bytes).unwrap();
    assert!(val);
    let val = unsafe { deser_unsafe::<bool>(&bytes).unwrap() };
    assert!(val);
}

#[derive(Debug, Rapira, PartialEq)]
struct StructVecFields {
    vec: Vec<u8>,
    arr: [i32; 8],
    arr_bytes: [u8; 4],
    #[rapira(with = rapira::byte_rapira)]
    byte: u8,
}

impl StructVecFields {
    pub fn random() -> Self {
        Self {
            vec: vec![0, 4, 6, 7],
            arr: [-1, 0, 4, 7, -8, 9, -1, 2],
            arr_bytes: [0, 1, 2, 3],
            byte: 4,
        }
    }
}

#[test]
fn test_vec_fields() -> Result<()> {
    let item = StructVecFields::random();

    let vec = serialize(&item);
    assert!(item == deserialize::<StructVecFields>(&vec)?);
    Ok(())
}

#[derive(Debug, Rapira, PartialEq, Serialize, Deserialize)]
struct UnnamedFields(
    Vec<u8>,
    [i32; 8],
    [u8; 4],
    #[rapira(with = rapira::byte_rapira)] u8,
);

impl UnnamedFields {
    pub fn random() -> Self {
        Self(
            vec![0, 4, 6, 7],
            [-1, 0, 4, 7, -8, 9, -1, 2],
            [0, 1, 2, 3],
            4,
        )
    }
}

#[test]
fn test_unnamed_fields() -> Result<()> {
    let item = UnnamedFields::random();

    let vec = serialize(&item);
    assert!(item == deserialize::<UnnamedFields>(&vec)?);
    Ok(())
}

#[repr(C)]
#[derive(Copy, Clone, IntoBytes, FromBytes, PartialEq, Debug, Immutable, KnownLayout)]
struct Zero {
    a: [u8; 16],
    b: U64<LE>,
    c: u8,
}

#[derive(Debug, Rapira, PartialEq)]
struct ZeroOwned {
    #[rapira(with = rapira::zero)]
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
    let vec = serialize(&item);
    assert!(item == deserialize::<ZeroOwned>(&vec)?);
    Ok(())
}

#[derive(Debug, Rapira, PartialEq)]
enum FullEnum {
    A(String),
    B(u16, u64, #[rapira(with = rapira::zero)] Zero),
    C {
        c1: bool,
        c2: (usize, isize),
        #[rapira(with = rapira::byte_rapira)]
        c3: u8,
        #[rapira(with = rapira::zero)]
        c4: Zero,
    },
    D,
}

#[derive(Debug, Rapira, PartialEq, Serialize, Deserialize)]
#[rapira(static_size = None)]
#[rapira(min_size = 1)]
enum NonStaticSized {
    A(String),
    B(Box<NonStaticSized>),
    C,
}

impl NonStaticSized {
    pub fn random() -> Self {
        Self::A(String::from("asd"))
    }

    pub fn random_with_child() -> Self {
        Self::B(Box::new(Self::random()))
    }
}

#[test]
fn test_enum() -> Result<()> {
    let zero = Zero {
        a: [3u8; 16],
        b: U64::<LE>::new(123123123123),
        c: 5,
    };
    let a = FullEnum::A("adasd".to_owned());
    let vec = serialize(&a);
    assert!(a == deserialize::<FullEnum>(&vec)?);

    let b = FullEnum::B(12, 12312312321123, zero);
    let vec = serialize(&b);
    assert!(b == deserialize::<FullEnum>(&vec)?);

    let c = FullEnum::C {
        c1: true,
        c2: (4_034_435_354, -12312312312),
        c3: 7,
        c4: zero,
    };
    let vec = serialize(&c);
    println!("{c:?}");
    let new_c: FullEnum = deserialize(&vec)?;
    println!("{new_c:?}");
    assert!(c == new_c);

    let d = FullEnum::D;
    let vec = serialize(&d);
    assert!(d == deserialize::<FullEnum>(&vec)?);

    let e = NonStaticSized::random_with_child();
    let vec = serialize(&e);
    assert!(e == deserialize::<NonStaticSized>(&vec)?);

    Ok(())
}

#[cfg(feature = "postcard")]
#[test]
fn test_postcard_fields() -> Result<()> {
    #[derive(Debug, Rapira, PartialEq)]
    struct PostcardFields {
        #[rapira(with = rapira::postcard)]
        vec: Vec<u8>,
        arr: [i32; 8],
        arr_bytes: [u8; 4],
        #[rapira(with = rapira::byte_rapira)]
        byte: u8,
        name: String,
        #[rapira(with = rapira::postcard)]
        e1: NonStaticSized,
        #[rapira(with = rapira::postcard)]
        e2: NonStaticSized,
        #[rapira(with = rapira::postcard)]
        s1: UnnamedFields,
    }

    let fields = PostcardFields {
        vec: vec![1, 2, 3, 4],
        arr: [1, 2, 3, 4, 5, 6, 7, 8],
        arr_bytes: [1, 2, 3, 4],
        byte: 42,
        name: "John".to_owned(),
        e1: NonStaticSized::random_with_child(),
        e2: NonStaticSized::random(),
        s1: UnnamedFields::random(),
    };

    let vec = serialize(&fields);
    let new_fields: PostcardFields = deserialize(&vec)?;

    assert_eq!(fields, new_fields);

    Ok(())
}
