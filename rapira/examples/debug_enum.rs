use rapira::Rapira;
use std::fmt::Debug;

#[derive(Debug, Rapira, PartialEq)]
enum SimpleEnum {
    Empty,
    Single(u32),
    Double(u16, u32),
}

#[derive(Debug, Rapira, PartialEq)]
enum ComplexEnum {
    Unnamed(String, Vec<u8>),
    Named { id: u64, name: String },
    Mixed,
}

fn main() {
    println!("=== Testing debug_from_slice for enums ===\n");

    // Test SimpleEnum variants
    test_simple_enum();

    // Test ComplexEnum variants
    test_complex_enum();
}

fn test_simple_enum() {
    println!("--- Testing SimpleEnum ---");

    // Test Empty variant
    println!("\nTesting Empty variant:");
    let original = SimpleEnum::Empty;
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = SimpleEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);

    // Test Single variant
    println!("\nTesting Single variant:");
    let original = SimpleEnum::Single(42);
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = SimpleEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);

    // Test Double variant
    println!("\nTesting Double variant:");
    let original = SimpleEnum::Double(255, 65535);
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = SimpleEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
}

fn test_complex_enum() {
    println!("\n--- Testing ComplexEnum ---");

    // Test Unnamed variant
    println!("\nTesting Unnamed variant:");
    let original = ComplexEnum::Unnamed("Hello".to_string(), vec![1, 2, 3, 4, 5]);
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = ComplexEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);

    // Test Named variant
    println!("\nTesting Named variant:");
    let original = ComplexEnum::Named {
        id: 12345,
        name: "Test Name".to_string(),
    };
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = ComplexEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);

    // Test Mixed variant
    println!("\nTesting Mixed variant:");
    let original = ComplexEnum::Mixed;
    let mut bytes = vec![0u8; 256];
    let mut cursor = 0;
    original.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    let mut slice = &bytes[..];
    let decoded = ComplexEnum::debug_from_slice(&mut slice).unwrap();
    println!("Original: {original:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(original, decoded);
}
