use rapira::*;

#[derive(Rapira, PartialEq)]
struct StructWithGeneric<T: PartialEq + Rapira, const N: usize> {
    a: String,
    b: u32,
    c: T,
    d: bool,
    e: [u16; N],
}

#[derive(Rapira, PartialEq)]
enum EnumWithGeneric<T>
where
    T: PartialEq + Rapira,
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
    let vec = with_generics.serialize();
    assert!(with_generics == StructWithGeneric::<Vec<u16>, 4>::deserialize(&vec)?);

    let a = EnumWithGeneric::<Vec<u16>>::A(vec![1, 2, 3, 4]);
    let vec = a.serialize();
    assert!(a == EnumWithGeneric::<Vec<u16>>::deserialize(&vec)?);

    let b = EnumWithGeneric::<Vec<u16>>::B(12);
    let vec = b.serialize();
    assert!(b == EnumWithGeneric::<Vec<u16>>::deserialize(&vec)?);

    let c = EnumWithGeneric::<Vec<u16>>::C;
    let vec = c.serialize();
    assert!(c == EnumWithGeneric::<Vec<u16>>::deserialize(&vec)?);

    Ok(())
}
