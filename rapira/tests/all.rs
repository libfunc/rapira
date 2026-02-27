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

// --- Versioned deserialization tests ---

#[derive(Debug, PartialEq, Rapira)]
#[rapira(version = 2)]
struct UserV2 {
    name: String,
    age: u32,
    #[rapira(since = 2)]
    email: Option<String>,
}

#[test]
fn test_versioned_roundtrip() {
    let user = UserV2 {
        name: "Alice".into(),
        age: 30,
        email: Some("a@b.com".into()),
    };
    let bytes = rapira::serialize(&user);
    // from_slice reads all fields (current version)
    let deser: UserV2 = rapira::deserialize(&bytes).unwrap();
    assert_eq!(user, deser);
    // deserialize_versioned with current version also reads all fields
    let deser: UserV2 = rapira::deserialize_versioned(&bytes, 2).unwrap();
    assert_eq!(user, deser);
}

#[test]
fn test_versioned_backward_compat() {
    // Simulate v1 data (name + age, no email)
    let mut v1_bytes = Vec::new();
    rapira::extend_vec(&"Alice".to_string(), &mut v1_bytes);
    rapira::extend_vec(&30u32, &mut v1_bytes);

    let user: UserV2 = rapira::deserialize_versioned(&v1_bytes, 1).unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(user.email, None); // Default
}

#[test]
fn test_versioned_in_vec() {
    // Vec<UserV2> with v1 data
    let mut v1_bytes = Vec::new();
    rapira::extend_vec(&2u32, &mut v1_bytes); // count = 2
    rapira::extend_vec(&"Alice".to_string(), &mut v1_bytes);
    rapira::extend_vec(&30u32, &mut v1_bytes);
    rapira::extend_vec(&"Bob".to_string(), &mut v1_bytes);
    rapira::extend_vec(&25u32, &mut v1_bytes);

    let users: Vec<UserV2> = rapira::deserialize_versioned(&v1_bytes, 1).unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].name, "Alice");
    assert_eq!(users[0].age, 30);
    assert_eq!(users[0].email, None);
    assert_eq!(users[1].name, "Bob");
    assert_eq!(users[1].age, 25);
    assert_eq!(users[1].email, None);
}

#[derive(Debug, PartialEq, Rapira)]
#[rapira(version = 3)]
struct UserV3 {
    name: String,
    age: u32,
    #[rapira(since = 2)]
    email: Option<String>,
    #[rapira(since = 3)]
    score: u64,
}

#[test]
fn test_versioned_multi_versions() {
    // v1 data: only name + age
    let mut v1_bytes = Vec::new();
    rapira::extend_vec(&"Alice".to_string(), &mut v1_bytes);
    rapira::extend_vec(&30u32, &mut v1_bytes);

    let user: UserV3 = rapira::deserialize_versioned(&v1_bytes, 1).unwrap();
    assert_eq!(user.name, "Alice");
    assert_eq!(user.age, 30);
    assert_eq!(user.email, None);
    assert_eq!(user.score, 0);

    // v2 data: name + age + email
    let mut v2_bytes = Vec::new();
    rapira::extend_vec(&"Bob".to_string(), &mut v2_bytes);
    rapira::extend_vec(&25u32, &mut v2_bytes);
    rapira::extend_vec(&Some("bob@test.com".to_string()), &mut v2_bytes);

    let user: UserV3 = rapira::deserialize_versioned(&v2_bytes, 2).unwrap();
    assert_eq!(user.name, "Bob");
    assert_eq!(user.age, 25);
    assert_eq!(user.email, Some("bob@test.com".into()));
    assert_eq!(user.score, 0);

    // v3 data (full): roundtrip
    let user = UserV3 {
        name: "Charlie".into(),
        age: 35,
        email: Some("charlie@test.com".into()),
        score: 100,
    };
    let bytes = rapira::serialize(&user);
    let deser: UserV3 = rapira::deserialize_versioned(&bytes, 3).unwrap();
    assert_eq!(user, deser);
}

#[derive(Debug, PartialEq, Rapira)]
#[rapira(version = 1)]
struct NoSinceFields {
    name: String,
    age: u32,
}

#[test]
fn test_versioned_no_since_fields() {
    // Struct with version but no since fields - behaves like normal
    let item = NoSinceFields {
        name: "Test".into(),
        age: 42,
    };
    let bytes = rapira::serialize(&item);
    let deser: NoSinceFields = rapira::deserialize(&bytes).unwrap();
    assert_eq!(item, deser);

    let deser: NoSinceFields = rapira::deserialize_versioned(&bytes, 1).unwrap();
    assert_eq!(item, deser);
}
