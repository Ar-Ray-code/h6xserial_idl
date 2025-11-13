mod emit_c;

use std::env;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use serde_json::{Map, Value};

pub fn run() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let language = parse_language(&mut args)?;

    let input_path = if !args.is_empty() {
        PathBuf::from(args.remove(0))
    } else {
        resolve_default_path(
            "msgs/intermediate_msg.json",
            "../msgs/intermediate_msg.json",
        )
    };

    let (primary_output, fallback_output) = language.default_output_paths();
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

    let source = match language {
        TargetLanguage::C => emit_c::generate(&metadata, &messages, &input_path, &output_path)?,
    };

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create output directory {}", parent.display()))?;
    }
    fs::write(&output_path, source)
        .with_context(|| format!("failed to write output to {}", output_path.display()))?;

    println!(
        "Generated {} output at {} for {} message definition(s).",
        language.display_name(),
        output_path.display(),
        messages.len()
    );

    Ok(())
}

fn parse_language(args: &mut Vec<String>) -> Result<TargetLanguage> {
    if let Some(first) = args.first().cloned() {
        if let Some(lang) = TargetLanguage::try_from_str(&first) {
            args.remove(0);
            return Ok(lang);
        }
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
        Self::try_from_str(value).ok_or_else(|| {
            anyhow::anyhow!("unsupported language '{}', expected 'c'", value)
        })
    }

    fn display_name(self) -> &'static str {
        match self {
            TargetLanguage::C => "C99"
        }
    }

    fn default_output_paths(self) -> (&'static str, &'static str) {
        match self {
            TargetLanguage::C => (
                "generated_c/seridl_generated_messages.h",
                "../generated_c/seridl_generated_messages.h",
            )
        }
    }

    fn template_subdir(self) -> &'static str {
        match self {
            TargetLanguage::C => "c"
        }
    }
}

#[derive(Default, Debug)]
pub(crate) struct Metadata {
    pub(crate) version: Option<String>,
    pub(crate) max_address: Option<u32>,
}

#[derive(Debug)]
pub(crate) struct MessageDefinition {
    pub(crate) name: String,
    pub(crate) packet_id: u32,
    pub(crate) description: Option<String>,
    pub(crate) body: MessageBody,
}

#[derive(Debug)]
pub(crate) enum MessageBody {
    Scalar(ScalarSpec),
    Array(ArraySpec),
    Struct(StructSpec),
}

#[derive(Debug)]
pub(crate) struct ScalarSpec {
    pub(crate) primitive: PrimitiveType,
    pub(crate) endian: Endian,
}

#[derive(Debug)]
pub(crate) struct ArraySpec {
    pub(crate) primitive: PrimitiveType,
    pub(crate) endian: Endian,
    pub(crate) max_length: usize,
    pub(crate) sector_bytes: Option<usize>,
}

#[derive(Debug)]
pub(crate) struct StructSpec {
    pub(crate) fields: Vec<StructField>,
}

#[derive(Debug)]
pub(crate) struct StructField {
    pub(crate) name: String,
    pub(crate) primitive: PrimitiveType,
    pub(crate) endian: Endian,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Endian {
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
pub(crate) enum PrimitiveType {
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

pub(crate) fn parse_messages(
    map: &Map<String, Value>,
) -> Result<(Metadata, Vec<MessageDefinition>)> {
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

fn parse_message_definition(name: &str, map: &Map<String, Value>) -> Result<MessageDefinition> {
    let packet_id =
        map.get("packet_id")
            .and_then(|v| v.as_u64())
            .with_context(|| format!("message '{}' is missing 'packet_id'", name))? as u32;

    let description = map
        .get("msg_desc")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let msg_type = map
        .get("msg_type")
        .and_then(|v| v.as_str())
        .with_context(|| format!("message '{}' is missing 'msg_type'", name))?;

    if msg_type.eq_ignore_ascii_case("struct") {
        let fields_obj = map
            .get("fields")
            .and_then(|v| v.as_object())
            .with_context(|| format!("struct message '{}' requires a 'fields' object", name))?;
        if fields_obj.is_empty() {
            bail!("struct message '{}' must define at least one field", name);
        }
        let mut fields = Vec::new();
        for (field_name, field_value) in fields_obj {
            let field_map = field_value.as_object().with_context(|| {
                format!(
                    "field '{}' in message '{}' must be an object",
                    field_name, name
                )
            })?;
            let type_str = field_map
                .get("type")
                .and_then(|v| v.as_str())
                .with_context(|| {
                    format!(
                        "field '{}' in message '{}' is missing 'type'",
                        field_name, name
                    )
                })?;
            let primitive = PrimitiveType::from_str(type_str).with_context(|| {
                format!(
                    "unsupported type '{}' for field '{}' in message '{}'",
                    type_str, field_name, name
                )
            })?;
            let endian = get_optional_endian(field_map)?.unwrap_or_default();
            fields.push(StructField {
                name: field_name.clone(),
                primitive,
                endian,
            });
        }
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
                .with_context(|| format!("array message '{}' requires 'max_length'", name))?
                as usize;
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
