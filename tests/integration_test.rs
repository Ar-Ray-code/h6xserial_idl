use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_generate_c_header_from_example_json() {
    // Use the example JSON file
    let input_path = PathBuf::from("example/example_json/intermediate_example.json");
    assert!(input_path.exists(), "Example JSON file should exist");

    // Create temporary output directory
    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("test_output.h");

    // Read and parse the input JSON
    let raw = fs::read_to_string(&input_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let obj = json.as_object().unwrap();

    let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
    assert!(!messages.is_empty(), "Should parse at least one message");
    messages.sort_by_key(|m| m.packet_id);

    // Generate C code
    let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();

    // Verify the generated code contains expected elements
    assert!(source.contains("#ifndef"), "Should have header guard");
    assert!(source.contains("#include <stdint.h>"), "Should include stdint.h");
    assert!(source.contains("typedef struct"), "Should have typedef struct");
    assert!(source.contains("static inline"), "Should have inline functions");

    // Write and verify file can be written
    fs::write(&output_path, source).unwrap();
    assert!(output_path.exists(), "Output file should be created");

    let written_content = fs::read_to_string(&output_path).unwrap();
    assert!(!written_content.is_empty(), "Generated file should not be empty");
}

#[test]
fn test_generate_c_header_for_all_message_types() {
    // Create a JSON with all message types
    let json_content = r#"{
        "version": "1.0.0",
        "max_address": 255,
        "scalar_uint8": {
            "packet_id": 1,
            "msg_type": "uint8",
            "array": false,
            "msg_desc": "Scalar uint8 message"
        },
        "scalar_float32_be": {
            "packet_id": 2,
            "msg_type": "float32",
            "array": false,
            "endianess": "big",
            "msg_desc": "Scalar float32 big-endian"
        },
        "array_char": {
            "packet_id": 3,
            "msg_type": "char",
            "array": true,
            "max_length": 32,
            "msg_desc": "Character array"
        },
        "array_int16_le": {
            "packet_id": 4,
            "msg_type": "int16",
            "array": true,
            "endianess": "little",
            "max_length": 8,
            "msg_desc": "Int16 array little-endian"
        },
        "struct_mixed": {
            "packet_id": 5,
            "msg_type": "struct",
            "fields": {
                "field_uint8": {
                    "type": "uint8"
                },
                "field_float32_be": {
                    "type": "float32",
                    "endianess": "big"
                },
                "field_uint32_le": {
                    "type": "uint32",
                    "endianess": "little"
                }
            },
            "msg_desc": "Struct with mixed types"
        }
    }"#;

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test_input.json");
    let output_path = temp_dir.path().join("test_output.h");

    fs::write(&input_path, json_content).unwrap();

    let raw = fs::read_to_string(&input_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let obj = json.as_object().unwrap();

    let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
    assert_eq!(messages.len(), 5, "Should parse 5 messages");
    messages.sort_by_key(|m| m.packet_id);

    assert_eq!(metadata.version, Some("1.0.0".to_string()));
    assert_eq!(metadata.max_address, Some(255));

    let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();

    // Verify all message types are present
    assert!(source.contains("SERIDL_MSG_SCALAR_UINT8_PACKET_ID 1"));
    assert!(source.contains("SERIDL_MSG_SCALAR_FLOAT32_BE_PACKET_ID 2"));
    assert!(source.contains("SERIDL_MSG_ARRAY_CHAR_PACKET_ID 3"));
    assert!(source.contains("SERIDL_MSG_ARRAY_INT16_LE_PACKET_ID 4"));
    assert!(source.contains("SERIDL_MSG_STRUCT_MIXED_PACKET_ID 5"));

    // Verify encode/decode functions
    assert!(source.contains("seridl_msg_scalar_uint8_encode"));
    assert!(source.contains("seridl_msg_scalar_uint8_decode"));
    assert!(source.contains("seridl_msg_array_char_encode"));
    assert!(source.contains("seridl_msg_struct_mixed_decode"));

    // Verify struct definitions
    assert!(source.contains("seridl_msg_scalar_uint8_t"));
    assert!(source.contains("seridl_msg_array_char_t"));
    assert!(source.contains("seridl_msg_struct_mixed_t"));

    // Verify endian helper functions are included
    assert!(source.contains("seridl_write_u16_le"));
    assert!(source.contains("seridl_read_u16_be"));
    assert!(source.contains("seridl_write_f32_be"));

    fs::write(&output_path, source).unwrap();
    assert!(output_path.exists());
}

#[test]
fn test_consistent_generation() {
    // Test that generating the same input multiple times produces identical output
    let json_content = r#"{
        "test_msg": {
            "packet_id": 42,
            "msg_type": "uint32",
            "array": false,
            "endianess": "big"
        }
    }"#;

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("test.json");
    let output_path = temp_dir.path().join("output.h");

    fs::write(&input_path, json_content).unwrap();

    // Generate twice with the same output path
    let mut source1 = String::new();
    let mut source2 = String::new();

    for (i, source_ref) in [&mut source1, &mut source2].iter_mut().enumerate() {
        let raw = fs::read_to_string(&input_path).unwrap();
        let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let obj = json.as_object().unwrap();
        let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
        messages.sort_by_key(|m| m.packet_id);
        let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();
        **source_ref = source;
        if i == 0 {
            fs::write(&output_path, &**source_ref).unwrap();
        }
    }

    // Compare outputs
    assert_eq!(source1, source2, "Multiple generations should produce identical output");
}

#[test]
fn test_header_guard_generation() {
    let json_content = r#"{
        "test": {
            "packet_id": 1,
            "msg_type": "uint8",
            "array": false
        }
    }"#;

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("my_messages.h");

    fs::write(&input_path, json_content).unwrap();

    let raw = fs::read_to_string(&input_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let obj = json.as_object().unwrap();
    let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
    messages.sort_by_key(|m| m.packet_id);
    let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();

    // Verify header guard matches filename
    assert!(source.contains("#ifndef MY_MESSAGES_H"));
    assert!(source.contains("#define MY_MESSAGES_H"));
    assert!(source.contains("#endif /* MY_MESSAGES_H */"));
}

#[test]
fn test_metadata_in_generated_header() {
    let json_content = r#"{
        "version": "2.3.4",
        "max_address": 128,
        "test": {
            "packet_id": 1,
            "msg_type": "uint8",
            "array": false
        }
    }"#;

    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.json");
    let output_path = temp_dir.path().join("output.h");

    fs::write(&input_path, json_content).unwrap();

    let raw = fs::read_to_string(&input_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let obj = json.as_object().unwrap();
    let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
    messages.sort_by_key(|m| m.packet_id);
    let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();

    // Verify metadata appears in comments
    assert!(source.contains("Protocol version: 2.3.4"));
    assert!(source.contains("Max address: 128"));
    assert!(source.contains("Auto-generated by h6xserial_idl"));
}

#[test]
fn test_sensor_example_generation() {
    // Test the actual sensor_messages.json from the example
    let input_path = PathBuf::from("example/c_usage/sensor_messages.json");

    // Skip test if file doesn't exist (for CI environments)
    if !input_path.exists() {
        eprintln!("Skipping test: sensor_messages.json not found");
        return;
    }

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.path().join("sensor_test.h");

    let raw = fs::read_to_string(&input_path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&raw).unwrap();
    let obj = json.as_object().unwrap();

    let (metadata, mut messages) = h6xserial_idl::parse_messages(obj).unwrap();
    messages.sort_by_key(|m| m.packet_id);

    assert!(messages.len() >= 5, "Sensor example should have multiple messages");

    let source = h6xserial_idl::emit_c::generate(&metadata, &messages, &input_path, &output_path).unwrap();

    // Verify some expected message types from sensor example
    assert!(source.contains("SERIDL_MSG_PING_PACKET_ID"));
    assert!(source.contains("SERIDL_MSG_TEMPERATURE_PACKET_ID"));
    assert!(source.contains("seridl_msg_sensor_data_t") || source.contains("sensor_data"));

    fs::write(&output_path, source).unwrap();
}
