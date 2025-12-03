//! h6xserial_idl - Code generator for serial communication message definitions
//!
//! This library reads JSON intermediate representations and generates
//! language-specific serializer/deserializer code for structured messages.

pub mod emit_c;
pub mod emit_markdown;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

/// Maximum supported array length for safety
const MAX_ARRAY_LENGTH: usize = 1024;

/// Runs the code generator with command-line arguments.
///
/// # Returns
/// * `Ok(())` - Generation succeeded
/// * `Err(...)` - Error with context about what failed
pub fn run() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();

    // Check for --export_docs flag
    let export_docs = parse_export_docs(&mut args);

    let language = parse_language(&mut args)?;

    let input_path = if !args.is_empty() {
        PathBuf::from(args.remove(0))
    } else {
        resolve_default_path(
            "msgs/intermediate_msg.json",
            "../msgs/intermediate_msg.json",
        )
    };

    let (primary_output, fallback_output) = if export_docs {
        ("docs/COMMANDS.md", "../docs/COMMANDS.md")
    } else {
        language.default_output_paths()
    };

    let output_path = if !args.is_empty() {
        PathBuf::from(args.remove(0))
    } else {
        resolve_default_path(primary_output, fallback_output)
    };

    let raw = fs::read_to_string(&input_path)
        .with_context(|| format!("failed to read input JSON: {}", input_path.display()))?;
    let json: Value =
        serde_json::from_str(&raw).context("failed to parse intermediate representation JSON")?;
    let obj = json
        .as_object()
        .context("top-level JSON must be an object")?;

    let (metadata, mut messages) = parse_messages(obj)?;
    if messages.is_empty() {
        bail!("no message definitions found in {}", input_path.display());
    }
    messages.sort_by_key(|m| m.packet_id);

    let source = if export_docs {
        emit_markdown::generate(&metadata, &messages, &input_path)?
    } else {
        match language {
            TargetLanguage::C => emit_c::generate(&metadata, &messages, &input_path, &output_path)?,
        }
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }
    fs::write(&output_path, source)
        .with_context(|| format!("failed to write output to {}", output_path.display()))?;

    if export_docs {
        println!(
            "Generated documentation at {} for {} command(s).",
            output_path.display(),
            messages.len()
        );
    } else {
        println!(
            "Generated {} output at {} for {} message definition(s).",
            language.display_name(),
            output_path.display(),
            messages.len()
        );
    }

    Ok(())
}

fn parse_export_docs(args: &mut Vec<String>) -> bool {
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--export_docs" {
            args.remove(index);
            return true;
        }
        index += 1;
    }
    false
}

fn parse_language(args: &mut Vec<String>) -> Result<TargetLanguage> {
    if let Some(first) = args.first().cloned()
        && let Some(lang) = TargetLanguage::try_from_str(&first)
    {
        args.remove(0);
        return Ok(lang);
    }

    let mut index = 0;
    while index < args.len() {
        if args[index] == "--lang" || args[index] == "-l" {
            if index + 1 >= args.len() {
                bail!("--lang requires a value (c)");
            }
            let value = args.remove(index + 1);
            args.remove(index);
            return TargetLanguage::parse(&value);
        }
        if let Some(value) = args[index].strip_prefix("--lang=") {
            let value = value.to_string();
            args.remove(index);
            return TargetLanguage::parse(&value);
        }
        index += 1;
    }

    Ok(TargetLanguage::C)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TargetLanguage {
    C,
}

impl TargetLanguage {
    fn try_from_str(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "c" | "c99" => Some(Self::C),
            _ => None,
        }
    }

    fn parse(value: &str) -> Result<Self> {
        Self::try_from_str(value)
            .ok_or_else(|| anyhow::anyhow!("unsupported language '{}', expected 'c'", value))
    }

    fn display_name(self) -> &'static str {
        match self {
            TargetLanguage::C => "C99",
        }
    }

    fn default_output_paths(self) -> (&'static str, &'static str) {
        match self {
            TargetLanguage::C => (
                "generated_c/h6xserial_generated_messages.h",
                "../generated_c/h6xserial_generated_messages.h",
            ),
        }
    }

    fn template_subdir(self) -> &'static str {
        match self {
            TargetLanguage::C => "c",
        }
    }
}

#[derive(Default, Debug)]
pub struct Metadata {
    pub version: Option<String>,
    pub max_address: Option<u32>,
}

#[derive(Debug)]
pub struct MessageDefinition {
    pub name: String,
    pub packet_id: u32,
    pub description: Option<String>,
    pub body: MessageBody,
}

#[derive(Debug)]
pub enum MessageBody {
    Scalar(ScalarSpec),
    Array(ArraySpec),
    Struct(StructSpec),
}

#[derive(Debug)]
pub struct ScalarSpec {
    pub primitive: PrimitiveType,
    pub endian: Endian,
}

#[derive(Debug)]
pub struct ArraySpec {
    pub primitive: PrimitiveType,
    pub endian: Endian,
    pub max_length: usize,
    pub sector_bytes: Option<usize>,
}

#[derive(Debug)]
pub struct StructSpec {
    pub fields: Vec<StructField>,
}

#[derive(Debug)]
pub struct StructField {
    pub name: String,
    pub field_type: StructFieldType,
    pub endian: Endian,
}

#[derive(Debug)]
pub struct StructFieldArraySpec {
    pub primitive: PrimitiveType,
    pub max_length: usize,
}

#[derive(Debug)]
pub enum StructFieldType {
    Primitive(PrimitiveType),
    Array(StructFieldArraySpec),
    Nested(StructSpec),
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Endian {
    #[default]
    Little,
    Big,
}

impl Endian {
    pub(crate) fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "little" | "le" => Ok(Endian::Little),
            "big" | "be" => Ok(Endian::Big),
            other => bail!("unsupported endian value '{}'", other),
        }
    }

    pub(crate) fn suffix(self) -> &'static str {
        match self {
            Endian::Little => "le",
            Endian::Big => "be",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PrimitiveType {
    Char,
    Int8,
    Uint8,
    Int16,
    Uint16,
    Int32,
    Uint32,
    Int64,
    Uint64,
    Float32,
    Float64,
}

impl PrimitiveType {
    pub(crate) fn from_str(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "char" => Ok(PrimitiveType::Char),
            "int8" | "i8" => Ok(PrimitiveType::Int8),
            "uint8" | "u8" => Ok(PrimitiveType::Uint8),
            "int16" | "i16" => Ok(PrimitiveType::Int16),
            "uint16" | "u16" => Ok(PrimitiveType::Uint16),
            "int32" | "i32" => Ok(PrimitiveType::Int32),
            "uint32" | "u32" => Ok(PrimitiveType::Uint32),
            "int64" | "i64" => Ok(PrimitiveType::Int64),
            "uint64" | "u64" => Ok(PrimitiveType::Uint64),
            "float32" | "f32" => Ok(PrimitiveType::Float32),
            "float64" | "f64" | "double" => Ok(PrimitiveType::Float64),
            other => bail!("unsupported primitive type '{}'", other),
        }
    }

    pub(crate) fn c_type(self) -> &'static str {
        match self {
            PrimitiveType::Char => "char",
            PrimitiveType::Int8 => "int8_t",
            PrimitiveType::Uint8 => "uint8_t",
            PrimitiveType::Int16 => "int16_t",
            PrimitiveType::Uint16 => "uint16_t",
            PrimitiveType::Int32 => "int32_t",
            PrimitiveType::Uint32 => "uint32_t",
            PrimitiveType::Int64 => "int64_t",
            PrimitiveType::Uint64 => "uint64_t",
            PrimitiveType::Float32 => "float",
            PrimitiveType::Float64 => "double",
        }
    }

    pub(crate) fn byte_len(self) -> usize {
        match self {
            PrimitiveType::Char | PrimitiveType::Int8 | PrimitiveType::Uint8 => 1,
            PrimitiveType::Int16 | PrimitiveType::Uint16 => 2,
            PrimitiveType::Int32 | PrimitiveType::Uint32 | PrimitiveType::Float32 => 4,
            PrimitiveType::Int64 | PrimitiveType::Uint64 | PrimitiveType::Float64 => 8,
        }
    }
}

/// Parses JSON message definitions into internal structures.
///
/// # Arguments
/// * `map` - JSON object containing metadata and message definitions
///
/// # Returns
/// * `Ok((metadata, messages))` - Parsed metadata and list of message definitions
/// * `Err(...)` - Parse error with detailed context
///
/// # Example
/// ```
/// use serde_json::json;
/// use h6xserial_idl::parse_messages;
///
/// let json = json!({
///     "version": "1.0.0",
///     "ping": {
///         "packet_id": 0,
///         "msg_type": "uint8",
///         "array": false
///     }
/// });
/// let obj = json.as_object().unwrap();
/// let (metadata, messages) = parse_messages(obj).unwrap();
/// assert_eq!(messages.len(), 1);
/// ```
pub fn parse_messages(map: &Map<String, Value>) -> Result<(Metadata, Vec<MessageDefinition>)> {
    let mut metadata = Metadata::default();
    let mut messages = Vec::new();

    for (key, value) in map {
        match key.as_str() {
            "version" => {
                metadata.version = value.as_str().map(|s| s.to_string());
            }
            "max_address" => {
                metadata.max_address = value.as_u64().map(|v| v as u32);
            }
            _ => {
                let msg_map = value
                    .as_object()
                    .with_context(|| format!("message '{}' must be an object", key))?;
                let definition = parse_message_definition(key, msg_map)?;
                messages.push(definition);
            }
        }
    }

    Ok((metadata, messages))
}

/// Parses a single message definition from JSON.
///
/// # Arguments
/// * `name` - Message name from JSON key
/// * `map` - JSON object for this message
///
/// # Returns
/// * `Ok(MessageDefinition)` - Parsed message
/// * `Err(...)` - Parse error with context
fn parse_message_definition(name: &str, map: &Map<String, Value>) -> Result<MessageDefinition> {
    let packet_id = map
        .get("packet_id")
        .and_then(|v| v.as_u64())
        .with_context(|| {
            format!(
                "message '{}' is missing required field 'packet_id' (must be 0-255)",
                name
            )
        })? as u32;

    if packet_id > 255 {
        bail!(
            "message '{}' has packet_id {} which exceeds maximum of 255",
            name,
            packet_id
        );
    }

    let description = map
        .get("msg_desc")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let msg_type = map
        .get("msg_type")
        .and_then(|v| v.as_str())
        .with_context(|| {
            format!(
                "message '{}' is missing required field 'msg_type' (e.g., 'uint8', 'float32', 'struct')",
                name
            )
        })?;

    if msg_type.eq_ignore_ascii_case("struct") {
        let fields_obj = map
            .get("fields")
            .and_then(|v| v.as_object())
            .with_context(|| {
                format!(
                    "struct message '{}' requires a 'fields' object containing field definitions",
                    name
                )
            })?;

        if fields_obj.is_empty() {
            bail!(
                "struct message '{}' must define at least one field in 'fields' object",
                name
            );
        }
        let fields = parse_struct_fields(fields_obj, name)?;
        Ok(MessageDefinition {
            name: name.to_string(),
            packet_id,
            description,
            body: MessageBody::Struct(StructSpec { fields }),
        })
    } else {
        let primitive = PrimitiveType::from_str(msg_type).with_context(|| {
            format!(
                "unsupported 'msg_type' '{}' for message '{}'",
                msg_type, name
            )
        })?;
        let endian = get_optional_endian(map)?.unwrap_or_default();
        let is_array = map.get("array").and_then(|v| v.as_bool()).unwrap_or(false);
        if is_array {
            let max_length = map
                .get("max_length")
                .and_then(|v| v.as_u64())
                .with_context(|| {
                    format!(
                        "array message '{}' requires 'max_length' field (1-{})",
                        name, MAX_ARRAY_LENGTH
                    )
                })? as usize;

            if max_length == 0 {
                bail!(
                    "array message '{}' has max_length of 0, must be at least 1",
                    name
                );
            }

            if max_length > MAX_ARRAY_LENGTH {
                bail!(
                    "array message '{}' has max_length {} which exceeds maximum of {}",
                    name,
                    max_length,
                    MAX_ARRAY_LENGTH
                );
            }
            let sector_bytes = map
                .get("sector_bytes")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);
            Ok(MessageDefinition {
                name: name.to_string(),
                packet_id,
                description,
                body: MessageBody::Array(ArraySpec {
                    primitive,
                    endian,
                    max_length,
                    sector_bytes,
                }),
            })
        } else {
            Ok(MessageDefinition {
                name: name.to_string(),
                packet_id,
                description,
                body: MessageBody::Scalar(ScalarSpec { primitive, endian }),
            })
        }
    }
}

/// Parses struct fields recursively, supporting nested structs.
fn parse_struct_fields(fields_obj: &Map<String, Value>, parent_name: &str) -> Result<Vec<StructField>> {
    let mut fields = Vec::new();
    for (field_name, field_value) in fields_obj {
        let field_map = field_value.as_object().with_context(|| {
            format!(
                "field '{}' in '{}' must be an object",
                field_name, parent_name
            )
        })?;

        // Support both "type" and "msg_type" for field type specification
        let type_str = field_map
            .get("type")
            .or_else(|| field_map.get("msg_type"))
            .and_then(|v| v.as_str())
            .with_context(|| {
                format!(
                    "field '{}' in '{}' is missing 'type' or 'msg_type'",
                    field_name, parent_name
                )
            })?;

        let endian = get_optional_endian(field_map)?.unwrap_or_default();

        // Check if this is a nested struct
        if type_str.eq_ignore_ascii_case("struct") {
            let nested_fields_obj = field_map
                .get("fields")
                .and_then(|v| v.as_object())
                .with_context(|| {
                    format!(
                        "nested struct field '{}' in '{}' requires a 'fields' object",
                        field_name, parent_name
                    )
                })?;

            if nested_fields_obj.is_empty() {
                bail!(
                    "nested struct field '{}' in '{}' must define at least one field",
                    field_name, parent_name
                );
            }

            let nested_path = format!("{}.{}", parent_name, field_name);
            let nested_fields = parse_struct_fields(nested_fields_obj, &nested_path)?;
            fields.push(StructField {
                name: field_name.clone(),
                field_type: StructFieldType::Nested(StructSpec { fields: nested_fields }),
                endian,
            });
        } else {
            let primitive = PrimitiveType::from_str(type_str).with_context(|| {
                format!(
                    "unsupported type '{}' for field '{}' in '{}'",
                    type_str, field_name, parent_name
                )
            })?;

            // Check if this field is an array
            let is_array = field_map.get("array").and_then(|v| v.as_bool()).unwrap_or(false);
            if is_array {
                let max_length = field_map
                    .get("max_length")
                    .and_then(|v| v.as_u64())
                    .with_context(|| {
                        format!(
                            "array field '{}' in '{}' requires 'max_length' field (1-{})",
                            field_name, parent_name, MAX_ARRAY_LENGTH
                        )
                    })? as usize;

                if max_length == 0 {
                    bail!(
                        "array field '{}' in '{}' has max_length of 0, must be at least 1",
                        field_name, parent_name
                    );
                }

                if max_length > MAX_ARRAY_LENGTH {
                    bail!(
                        "array field '{}' in '{}' has max_length {} which exceeds maximum of {}",
                        field_name, parent_name, max_length, MAX_ARRAY_LENGTH
                    );
                }

                fields.push(StructField {
                    name: field_name.clone(),
                    field_type: StructFieldType::Array(StructFieldArraySpec {
                        primitive,
                        max_length,
                    }),
                    endian,
                });
            } else {
                fields.push(StructField {
                    name: field_name.clone(),
                    field_type: StructFieldType::Primitive(primitive),
                    endian,
                });
            }
        }
    }
    Ok(fields)
}

fn get_optional_endian(map: &Map<String, Value>) -> Result<Option<Endian>> {
    for key in ["endianess", "endianness"] {
        if let Some(value) = map.get(key) {
            let text = value
                .as_str()
                .with_context(|| format!("'{}' must be a string", key))?;
            return Ok(Some(Endian::from_str(text)?));
        }
    }
    Ok(None)
}

pub(crate) fn load_templates(language: TargetLanguage, files: &[&str]) -> Result<String> {
    let template_dir = resolve_template_dir(language)?;
    let mut combined = String::new();

    for file_name in files {
        let path = template_dir.join(file_name);
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read template {}", path.display()))?;
        combined.push_str(&content);
        if !content.ends_with('\n') {
            combined.push('\n');
        }
        combined.push('\n');
    }

    Ok(combined)
}

fn resolve_template_dir(language: TargetLanguage) -> Result<PathBuf> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let subdir = language.template_subdir();
    let relative_candidates = [
        format!("src/msg_template/{}", subdir),
        format!("msg_template/{}", subdir),
        format!("../src/msg_template/{}", subdir),
        format!("../msg_template/{}", subdir),
    ];

    let mut candidates: Vec<PathBuf> = Vec::new();
    for rel in &relative_candidates {
        candidates.push(PathBuf::from(rel));
    }
    for rel in &relative_candidates {
        candidates.push(manifest_dir.join(rel));
    }

    for candidate in candidates {
        if candidate.is_dir() {
            return Ok(candidate);
        }
    }

    bail!(
        "could not locate 'msg_template/{}' directory for language {}",
        subdir,
        language.display_name()
    )
}

fn resolve_default_path(primary: &str, fallback: &str) -> PathBuf {
    let primary_path = PathBuf::from(primary);
    if primary_path.exists() {
        primary_path
    } else {
        PathBuf::from(fallback)
    }
}

pub(crate) fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            let lower = ch.to_ascii_lowercase();
            if result.is_empty() && lower.is_ascii_digit() {
                result.push('_');
            }
            result.push(lower);
            last_was_underscore = false;
        } else if !last_was_underscore {
            result.push('_');
            last_was_underscore = true;
        }
    }
    if result.ends_with('_') {
        result.pop();
    }
    if result.is_empty() {
        result.push_str("msg");
    }
    result
}

pub(crate) fn to_macro_ident(name: &str) -> String {
    let mut result = String::new();
    let mut last_was_underscore = false;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            let upper = ch.to_ascii_uppercase();
            if result.is_empty() && upper.is_ascii_digit() {
                result.push('_');
            }
            result.push(upper);
            last_was_underscore = false;
        } else if !last_was_underscore {
            result.push('_');
            last_was_underscore = true;
        }
    }
    if result.ends_with('_') {
        result.pop();
    }
    if result.is_empty() {
        result.push_str("MSG");
    }
    result
}

#[allow(dead_code)]
pub(crate) fn to_pascal_case(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if result.is_empty() && ch.is_ascii_digit() {
                result.push('M');
            }
            if capitalize {
                result.push(ch.to_ascii_uppercase());
            } else {
                result.push(ch.to_ascii_lowercase());
            }
            capitalize = false;
        } else {
            capitalize = true;
        }
    }
    if result.is_empty() {
        result.push_str("Msg");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_to_snake_case() {
        // Note: to_snake_case converts to lowercase but doesn't detect camelCase boundaries
        assert_eq!(to_snake_case("HelloWorld"), "helloworld");
        assert_eq!(to_snake_case("get_temperatures"), "get_temperatures");
        assert_eq!(to_snake_case("LED Control"), "led_control");
        assert_eq!(to_snake_case("CO2Level"), "co2level");
        assert_eq!(to_snake_case("firmware_version"), "firmware_version");
        assert_eq!(to_snake_case("123test"), "_123test");
        assert_eq!(to_snake_case(""), "msg");
    }

    #[test]
    fn test_to_macro_ident() {
        // Note: to_macro_ident converts to uppercase but doesn't detect camelCase boundaries
        assert_eq!(to_macro_ident("HelloWorld"), "HELLOWORLD");
        assert_eq!(to_macro_ident("get_temperatures"), "GET_TEMPERATURES");
        assert_eq!(to_macro_ident("LED Control"), "LED_CONTROL");
        assert_eq!(to_macro_ident("CO2Level"), "CO2LEVEL");
        assert_eq!(to_macro_ident("firmware_version"), "FIRMWARE_VERSION");
        assert_eq!(to_macro_ident("123test"), "_123TEST");
        assert_eq!(to_macro_ident(""), "MSG");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("get_temperatures"), "GetTemperatures");
        assert_eq!(to_pascal_case("LED Control"), "LedControl");
        assert_eq!(to_pascal_case("CO2Level"), "Co2level");
        assert_eq!(to_pascal_case("firmware_version"), "FirmwareVersion");
        assert_eq!(to_pascal_case("123test"), "M123test");
        assert_eq!(to_pascal_case(""), "Msg");
    }

    #[test]
    fn test_primitive_type_from_str() {
        assert_eq!(
            PrimitiveType::from_str("char").unwrap(),
            PrimitiveType::Char
        );
        assert_eq!(
            PrimitiveType::from_str("uint8").unwrap(),
            PrimitiveType::Uint8
        );
        assert_eq!(
            PrimitiveType::from_str("int16").unwrap(),
            PrimitiveType::Int16
        );
        assert_eq!(
            PrimitiveType::from_str("float32").unwrap(),
            PrimitiveType::Float32
        );
        assert_eq!(
            PrimitiveType::from_str("f64").unwrap(),
            PrimitiveType::Float64
        );
        assert!(PrimitiveType::from_str("invalid").is_err());
    }

    #[test]
    fn test_primitive_type_c_type() {
        assert_eq!(PrimitiveType::Char.c_type(), "char");
        assert_eq!(PrimitiveType::Uint8.c_type(), "uint8_t");
        assert_eq!(PrimitiveType::Int16.c_type(), "int16_t");
        assert_eq!(PrimitiveType::Float32.c_type(), "float");
        assert_eq!(PrimitiveType::Float64.c_type(), "double");
    }

    #[test]
    fn test_primitive_type_byte_len() {
        assert_eq!(PrimitiveType::Char.byte_len(), 1);
        assert_eq!(PrimitiveType::Uint8.byte_len(), 1);
        assert_eq!(PrimitiveType::Int16.byte_len(), 2);
        assert_eq!(PrimitiveType::Uint32.byte_len(), 4);
        assert_eq!(PrimitiveType::Float32.byte_len(), 4);
        assert_eq!(PrimitiveType::Float64.byte_len(), 8);
    }

    #[test]
    fn test_endian_from_str() {
        assert_eq!(Endian::from_str("little").unwrap(), Endian::Little);
        assert_eq!(Endian::from_str("big").unwrap(), Endian::Big);
        assert_eq!(Endian::from_str("le").unwrap(), Endian::Little);
        assert_eq!(Endian::from_str("be").unwrap(), Endian::Big);
        assert!(Endian::from_str("invalid").is_err());
    }

    #[test]
    fn test_endian_suffix() {
        assert_eq!(Endian::Little.suffix(), "le");
        assert_eq!(Endian::Big.suffix(), "be");
    }

    #[test]
    fn test_target_language_parse() {
        assert_eq!(TargetLanguage::parse("c").unwrap(), TargetLanguage::C);
        assert_eq!(TargetLanguage::parse("C99").unwrap(), TargetLanguage::C);
        assert!(TargetLanguage::parse("python").is_err());
    }

    #[test]
    fn test_parse_scalar_message() {
        let json = json!({
            "version": "1.0.0",
            "ping": {
                "packet_id": 0,
                "msg_type": "uint8",
                "array": false,
                "msg_desc": "Ping command"
            }
        });

        let obj = json.as_object().unwrap();
        let (metadata, messages) = parse_messages(obj).unwrap();

        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].name, "ping");
        assert_eq!(messages[0].packet_id, 0);
        assert_eq!(messages[0].description, Some("Ping command".to_string()));

        match &messages[0].body {
            MessageBody::Scalar(spec) => {
                assert_eq!(spec.primitive, PrimitiveType::Uint8);
                assert_eq!(spec.endian, Endian::Little);
            }
            _ => panic!("Expected scalar message"),
        }
    }

    #[test]
    fn test_parse_array_message() {
        let json = json!({
            "temperatures": {
                "packet_id": 20,
                "msg_type": "float32",
                "array": true,
                "endianess": "big",
                "max_length": 8,
                "msg_desc": "Temperature array"
            }
        });

        let obj = json.as_object().unwrap();
        let (_, messages) = parse_messages(obj).unwrap();

        assert_eq!(messages.len(), 1);
        match &messages[0].body {
            MessageBody::Array(spec) => {
                assert_eq!(spec.primitive, PrimitiveType::Float32);
                assert_eq!(spec.endian, Endian::Big);
                assert_eq!(spec.max_length, 8);
            }
            _ => panic!("Expected array message"),
        }
    }

    #[test]
    fn test_parse_struct_message() {
        let json = json!({
            "sensor_data": {
                "packet_id": 30,
                "msg_type": "struct",
                "fields": {
                    "temperature": {
                        "type": "float32",
                        "endianess": "big"
                    },
                    "humidity": {
                        "type": "uint8"
                    }
                },
                "msg_desc": "Sensor readings"
            }
        });

        let obj = json.as_object().unwrap();
        let (_, messages) = parse_messages(obj).unwrap();

        assert_eq!(messages.len(), 1);
        match &messages[0].body {
            MessageBody::Struct(spec) => {
                assert_eq!(spec.fields.len(), 2);
                // Note: JSON object field order is not guaranteed, so check both fields exist
                let temp_field = spec.fields.iter().find(|f| f.name == "temperature");
                let hum_field = spec.fields.iter().find(|f| f.name == "humidity");

                assert!(temp_field.is_some(), "temperature field should exist");
                let temp_field = temp_field.unwrap();
                match &temp_field.field_type {
                    StructFieldType::Primitive(prim) => {
                        assert_eq!(*prim, PrimitiveType::Float32);
                    }
                    _ => panic!("Expected primitive field"),
                }
                assert_eq!(temp_field.endian, Endian::Big);

                assert!(hum_field.is_some(), "humidity field should exist");
                let hum_field = hum_field.unwrap();
                match &hum_field.field_type {
                    StructFieldType::Primitive(prim) => {
                        assert_eq!(*prim, PrimitiveType::Uint8);
                    }
                    _ => panic!("Expected primitive field"),
                }
            }
            _ => panic!("Expected struct message"),
        }
    }

    #[test]
    fn test_parse_messages_sorted_by_packet_id() {
        let json = json!({
            "version": "1.0.0",
            "max_address": 255,
            "msg_c": {
                "packet_id": 30,
                "msg_type": "uint8",
                "array": false
            },
            "msg_a": {
                "packet_id": 10,
                "msg_type": "uint8",
                "array": false
            },
            "msg_b": {
                "packet_id": 20,
                "msg_type": "uint8",
                "array": false
            }
        });

        let obj = json.as_object().unwrap();
        let (metadata, mut messages) = parse_messages(obj).unwrap();

        assert_eq!(metadata.version, Some("1.0.0".to_string()));
        assert_eq!(metadata.max_address, Some(255));
        assert_eq!(messages.len(), 3);

        messages.sort_by_key(|m| m.packet_id);
        assert_eq!(messages[0].name, "msg_a");
        assert_eq!(messages[0].packet_id, 10);
        assert_eq!(messages[1].name, "msg_b");
        assert_eq!(messages[1].packet_id, 20);
        assert_eq!(messages[2].name, "msg_c");
        assert_eq!(messages[2].packet_id, 30);
    }

    #[test]
    fn test_array_without_max_length_fails() {
        let json = json!({
            "temperatures": {
                "packet_id": 20,
                "msg_type": "float32",
                "array": true
            }
        });

        let obj = json.as_object().unwrap();
        let result = parse_messages(obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_without_fields_fails() {
        let json = json!({
            "sensor_data": {
                "packet_id": 30,
                "msg_type": "struct"
            }
        });

        let obj = json.as_object().unwrap();
        let result = parse_messages(obj);
        assert!(result.is_err());
    }

    #[test]
    fn test_struct_with_empty_fields_fails() {
        let json = json!({
            "sensor_data": {
                "packet_id": 30,
                "msg_type": "struct",
                "fields": {}
            }
        });

        let obj = json.as_object().unwrap();
        let result = parse_messages(obj);
        assert!(result.is_err());
    }
}
