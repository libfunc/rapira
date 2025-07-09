use rapira::*;
use std::fmt::Debug;

#[derive(Rapira, Debug, PartialEq)]
#[rapira(debug)]
struct StructWithGeneric<T: PartialEq + Rapira + Debug, const N: usize> {
    a: String,
    b: u32,
    c: T,
    d: bool,
    e: [u16; N],
}

#[derive(Rapira, Debug, PartialEq)]
#[rapira(debug)]
enum EnumWithGeneric<T>
where
    T: PartialEq + Rapira + Debug,
{
    A(T),
    B(u32),
    C,
}

#[test]
fn test_generics() -> Result<()> {
    let with_generics = StructWithGeneric::<Vec<u16>, 4> {
        a: String::from("asdasd"),
        b: 234234,
        c: vec![1, 2, 3, 4],
        d: true,
        e: [312; 4],
    };
    let vec = serialize(&with_generics);
    assert!(with_generics == deserialize::<StructWithGeneric<Vec<u16>, 4>>(&vec)?);

    let a = EnumWithGeneric::<Vec<u16>>::A(vec![1, 2, 3, 4]);
    let vec = serialize(&a);
    assert!(a == deserialize::<EnumWithGeneric<Vec<u16>>>(&vec)?);

    let b = EnumWithGeneric::<Vec<u16>>::B(12);
    let vec = serialize(&b);
    assert!(b == deserialize::<EnumWithGeneric<Vec<u16>>>(&vec)?);

    let c = EnumWithGeneric::<Vec<u16>>::C;
    let vec = serialize(&c);
    assert!(c == deserialize::<EnumWithGeneric<Vec<u16>>>(&vec)?);

    Ok(())
}
