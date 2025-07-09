use rapira::Rapira;
use std::fmt::Debug;

#[derive(Debug, Rapira, PartialEq)]
struct NamedStruct {
    id: u64,
    name: String,
    active: bool,
    score: f32,
}

#[derive(Debug, Rapira, PartialEq)]
struct TupleStruct(u32, String, Vec<i32>);

#[derive(Debug, Rapira, PartialEq)]
struct UnitStruct;

#[derive(Debug, Rapira, PartialEq)]
struct ComplexStruct {
    user_id: u64,
    username: String,
    emails: Vec<String>,
    settings: NestedStruct,
}

#[derive(Debug, Rapira, PartialEq)]
struct NestedStruct {
    notifications: bool,
    theme: String,
    volume: u16,
}

fn main() {
    println!("=== Testing debug_from_slice for structs ===\n");

    // Test Named struct
    test_named_struct();

    // Test Tuple struct
    test_tuple_struct();

    // Test Unit struct
    test_unit_struct();

    // Test Complex struct with nested fields
    test_complex_struct();
}

fn test_named_struct() {
    println!("--- Testing NamedStruct ---");

    let original = NamedStruct {
        id: 12345,
        name: "Alice".to_string(),
        active: true,
        score: 98.5,
    };

    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    let mut slice = &bytes[..];
    let decoded = NamedStruct::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
    println!("✓ NamedStruct test passed\n");
}

fn test_tuple_struct() {
    println!("--- Testing TupleStruct ---");

    let original = TupleStruct(42, "Hello World".to_string(), vec![1, -2, 3, -4, 5]);

    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    let mut slice = &bytes[..];
    let decoded = TupleStruct::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
    println!("✓ TupleStruct test passed\n");
}

fn test_unit_struct() {
    println!("--- Testing UnitStruct ---");

    let original = UnitStruct;

    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    let mut slice = &bytes[..];
    let decoded = UnitStruct::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
    println!("✓ UnitStruct test passed\n");
}

fn test_complex_struct() {
    println!("--- Testing ComplexStruct ---");

    let original = ComplexStruct {
        user_id: 999888777,
        username: "power_user".to_string(),
        emails: vec![
            "user@example.com".to_string(),
            "backup@example.com".to_string(),
        ],
        settings: NestedStruct {
            notifications: false,
            theme: "dark".to_string(),
            volume: 75,
        },
    };

    let mut bytes = vec![0u8; 512];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    let mut slice = &bytes[..];
    let decoded = ComplexStruct::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
    println!("✓ ComplexStruct test passed\n");
}
