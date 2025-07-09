use rapira::Rapira;
use std::fmt::Debug;

// Basic enum with different variant types
#[derive(Debug, Rapira, PartialEq)]
enum Status {
    Active,
    Inactive { reason: String },
    Suspended(u32, String),
}

// Struct containing an enum
#[derive(Debug, Rapira, PartialEq)]
struct User {
    id: u64,
    name: String,
    status: Status,
    permissions: Vec<Permission>,
}

// Another enum type
#[derive(Debug, Rapira, PartialEq)]
enum Permission {
    Read,
    Write,
    Admin { level: u16 },
}

// Enum containing structs
#[derive(Debug, Rapira, PartialEq)]
enum Message {
    Text(TextMessage),
    Image {
        url: String,
        metadata: ImageMetadata,
    },
    System,
}

#[derive(Debug, Rapira, PartialEq)]
struct TextMessage {
    content: String,
    timestamp: u64,
}

#[derive(Debug, Rapira, PartialEq)]
struct ImageMetadata {
    width: u32,
    height: u32,
    format: String,
}

// Complex nested structure
#[derive(Debug, Rapira, PartialEq)]
struct Application {
    version: (u16, u16, u16),
    users: Vec<User>,
    messages: Vec<Message>,
}

fn main() {
    println!("=== Testing debug_from_slice with combined structs and enums ===\n");

    // Test basic enum
    test_status_enum();

    // Test struct with enum field
    test_user_struct();

    // Test enum containing structs
    test_message_enum();

    // Test complex nested structure
    test_application();
}

fn test_status_enum() {
    println!("--- Testing Status enum ---");

    let statuses = vec![
        Status::Active,
        Status::Inactive {
            reason: "Maintenance".to_string(),
        },
        Status::Suspended(30, "Policy violation".to_string()),
    ];

    for (i, status) in statuses.into_iter().enumerate() {
        println!("\nTest case {}:", i + 1);

        let mut bytes = vec![0u8; 256];
        let mut cursor = 0;
        status.convert_to_bytes(&mut bytes, &mut cursor);
        bytes.truncate(cursor);

        println!("Serialized {} bytes", bytes.len());
        println!("\nDeserializing with debug_from_slice:");
        let mut slice = &bytes[..];
        let decoded = Status::debug_from_slice(&mut slice).unwrap();

        println!("\nOriginal: {status:?}");
        println!("Decoded:  {decoded:?}");
        assert_eq!(status, decoded);
    }
    println!("\n✓ Status enum tests passed\n");
}

fn test_user_struct() {
    println!("--- Testing User struct with enum fields ---");

    let user = User {
        id: 42,
        name: "Alice".to_string(),
        status: Status::Active,
        permissions: vec![
            Permission::Read,
            Permission::Write,
            Permission::Admin { level: 5 },
        ],
    };

    let mut bytes = vec![0u8; 512];
    let mut cursor = 0;
    user.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    let mut slice = &bytes[..];
    let decoded = User::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {user:?}");
    println!("Decoded:  {decoded:?}");
    assert_eq!(user, decoded);
    println!("✓ User struct test passed\n");
}

fn test_message_enum() {
    println!("--- Testing Message enum containing structs ---");

    let messages = vec![
        Message::Text(TextMessage {
            content: "Hello, World!".to_string(),
            timestamp: 1234567890,
        }),
        Message::Image {
            url: "https://example.com/image.png".to_string(),
            metadata: ImageMetadata {
                width: 1920,
                height: 1080,
                format: "PNG".to_string(),
            },
        },
        Message::System,
    ];

    for (i, message) in messages.into_iter().enumerate() {
        println!(
            "\nTest case {} - {:?}:",
            i + 1,
            match &message {
                Message::Text(_) => "Text",
                Message::Image { .. } => "Image",
                Message::System => "System",
            }
        );

        let mut bytes = vec![0u8; 512];
        let mut cursor = 0;
        message.convert_to_bytes(&mut bytes, &mut cursor);
        bytes.truncate(cursor);

        println!("Serialized {} bytes", bytes.len());
        println!("\nDeserializing with debug_from_slice:");
        let mut slice = &bytes[..];
        let decoded = Message::debug_from_slice(&mut slice).unwrap();

        println!("\nOriginal: {message:?}");
        println!("Decoded:  {decoded:?}");
        assert_eq!(message, decoded);
    }
    println!("\n✓ Message enum tests passed\n");
}

fn test_application() {
    println!("--- Testing complex Application struct ---");

    let app = Application {
        version: (1, 2, 3),
        users: vec![
            User {
                id: 1,
                name: "Admin".to_string(),
                status: Status::Active,
                permissions: vec![Permission::Admin { level: 10 }],
            },
            User {
                id: 2,
                name: "Guest".to_string(),
                status: Status::Inactive {
                    reason: "Trial expired".to_string(),
                },
                permissions: vec![Permission::Read],
            },
        ],
        messages: vec![
            Message::System,
            Message::Text(TextMessage {
                content: "Welcome!".to_string(),
                timestamp: 1000000000,
            }),
        ],
    };

    let mut bytes = vec![0u8; 1024];
    let mut cursor = 0;
    app.convert_to_bytes(&mut bytes, &mut cursor);
    bytes.truncate(cursor);

    println!("\nDeserializing with debug_from_slice:");
    println!("Total size: {} bytes", bytes.len());
    let mut slice = &bytes[..];
    let decoded = Application::debug_from_slice(&mut slice).unwrap();

    println!("\nOriginal: {app:#?}");
    println!("\nDecoded:  {decoded:#?}");
    assert_eq!(app, decoded);
    println!("✓ Application struct test passed\n");

    println!("=== All tests passed! ===");
}
