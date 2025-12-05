//! C99 code generator for message definitions.
//!
//! Generates header files with type definitions and encode/decode functions.

use std::collections::HashSet;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::Result;

use crate::{
    ArraySpec, Endian, MessageBody, MessageDefinition, Metadata, PrimitiveType, RequestType,
    ScalarSpec, StructField, StructFieldType, StructSpec, TargetLanguage, load_templates,
    to_macro_ident, to_snake_case,
};

/// Determines which functions to generate for a message.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionMode {
    /// Generate only encode function
    EncodeOnly,
    /// Generate only decode function
    DecodeOnly,
    /// Generate both encode and decode functions
    Both,
}

/// Output file specification for multi-file generation.
#[derive(Debug)]
pub struct OutputFile {
    pub filename: String,
    pub content: String,
}

/// Template files containing C helper functions for serialization.
const TEMPLATE_FILES: &[&str] = &[
    "helpers_u16.h",
    "helpers_u32.h",
    "helpers_u64.h",
    "helpers_f32.h",
    "helpers_f64.h",
];

/// Generates multiple C99 header files for server and clients.
///
/// This function creates:
/// - `<base_name>_types.h` - Common type definitions, macros, and helper functions
/// - `<base_name>_server.h` - Server header with pub->encode, sub->decode
/// - `<base_name>_client_common.h` - Common client functions (for target_client_id=-1)
/// - `<base_name>_client_<id>.h` - Client headers with pub->decode, sub->encode
///
/// # Arguments
/// * `metadata` - Protocol metadata (version, max_address)
/// * `messages` - List of message definitions to generate code for
/// * `input_path` - Path to input JSON file (for documentation)
/// * `base_name` - Base name for generated files (without extension)
///
/// # Returns
/// * `Ok(Vec<OutputFile>)` - List of generated files with their content
/// * `Err(...)` - Generation error with context
pub fn generate_multiple(
    metadata: &Metadata,
    messages: &[MessageDefinition],
    input_path: &Path,
    base_name: &str,
) -> Result<Vec<OutputFile>> {
    let helper_block = load_templates(TargetLanguage::C, TEMPLATE_FILES)?;
    let mut files = Vec::new();

    // Collect all unique client IDs
    let client_ids: HashSet<i32> = messages
        .iter()
        .filter(|m| m.target_client_id > 0)
        .map(|m| m.target_client_id)
        .collect();

    // Generate types header (common definitions)
    let types_filename = format!("{}_types.h", base_name);
    let types_content = generate_types_header(
        metadata,
        messages,
        input_path,
        &types_filename,
        &helper_block,
    );
    files.push(OutputFile {
        filename: types_filename.clone(),
        content: types_content,
    });

    // Generate server header
    let server_filename = format!("{}_server.h", base_name);
    let server_content = generate_header_for_role(
        metadata,
        messages,
        input_path,
        &server_filename,
        &types_filename,
        Role::Server,
        None,
    );
    files.push(OutputFile {
        filename: server_filename,
        content: server_content,
    });

    // Generate client common header (for target_client_id=-1 messages)
    let client_common_filename = format!("{}_client_common.h", base_name);
    let client_common_content = generate_header_for_role(
        metadata,
        messages,
        input_path,
        &client_common_filename,
        &types_filename,
        Role::ClientCommon,
        None,
    );
    files.push(OutputFile {
        filename: client_common_filename.clone(),
        content: client_common_content,
    });

    // Generate client headers for each unique client ID
    for client_id in &client_ids {
        let client_filename = format!("{}_client_{}.h", base_name, client_id);
        let client_content = generate_header_for_role(
            metadata,
            messages,
            input_path,
            &client_filename,
            &types_filename,
            Role::Client(*client_id),
            Some(&client_common_filename),
        );
        files.push(OutputFile {
            filename: client_filename,
            content: client_content,
        });
    }

    Ok(files)
}

/// Role for which to generate the header.
#[derive(Clone, Copy, Debug)]
enum Role {
    /// Server role: pub->encode, sub->decode
    Server,
    /// Client common role: only messages with target_client_id=-1
    ClientCommon,
    /// Client role with specific ID: pub->decode, sub->encode (only specific messages)
    Client(i32),
}

/// Generates the types header containing common definitions.
/// This includes:
/// - Helper functions for serialization
/// - Type definitions (structs)
/// - Packet ID macros
/// - Max length macros
fn generate_types_header(
    metadata: &Metadata,
    messages: &[MessageDefinition],
    input_path: &Path,
    filename: &str,
    helper_block: &str,
) -> String {
    let header_guard = header_guard_name_from_str(filename);

    let mut out = String::new();
    writeln!(&mut out, "/*").unwrap();
    writeln!(&mut out, " * Auto-generated by h6xserial_idl.").unwrap();
    writeln!(&mut out, " * Source: {}", input_path.display()).unwrap();
    writeln!(&mut out, " * Common type definitions and helper functions").unwrap();
    if let Some(version) = &metadata.version {
        writeln!(&mut out, " * Protocol version: {}", version).unwrap();
    }
    if let Some(max_address) = metadata.max_address {
        writeln!(&mut out, " * Max address: {}", max_address).unwrap();
    }
    writeln!(&mut out, " */\n").unwrap();

    writeln!(&mut out, "#ifndef {}", header_guard).unwrap();
    writeln!(&mut out, "#define {}\n", header_guard).unwrap();

    out.push_str(
        "#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n#include <string.h>\n\n",
    );

    out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
    out.push_str(helper_block);

    // Generate type definitions only (no functions)
    for msg in messages {
        out.push('\n');
        out.push_str(&generate_message_types_only(msg));
    }

    out.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n");
    writeln!(&mut out, "#endif /* {} */", header_guard).unwrap();

    out
}

/// Generates a header file for a specific role (server or client).
/// This header includes the types header and defines only the functions.
fn generate_header_for_role(
    metadata: &Metadata,
    messages: &[MessageDefinition],
    input_path: &Path,
    filename: &str,
    types_header: &str,
    role: Role,
    client_common_header: Option<&str>,
) -> String {
    let header_guard = header_guard_name_from_str(filename);

    let mut out = String::new();
    writeln!(&mut out, "/*").unwrap();
    writeln!(&mut out, " * Auto-generated by h6xserial_idl.").unwrap();
    writeln!(&mut out, " * Source: {}", input_path.display()).unwrap();
    match role {
        Role::Server => writeln!(&mut out, " * Role: Server").unwrap(),
        Role::ClientCommon => writeln!(&mut out, " * Role: Client (Common)").unwrap(),
        Role::Client(id) => writeln!(&mut out, " * Role: Client (ID: {})", id).unwrap(),
    }
    if let Some(version) = &metadata.version {
        writeln!(&mut out, " * Protocol version: {}", version).unwrap();
    }
    if let Some(max_address) = metadata.max_address {
        writeln!(&mut out, " * Max address: {}", max_address).unwrap();
    }
    writeln!(&mut out, " */\n").unwrap();

    writeln!(&mut out, "#ifndef {}", header_guard).unwrap();
    writeln!(&mut out, "#define {}\n", header_guard).unwrap();

    // Include the types header
    writeln!(&mut out, "#include \"{}\"", types_header).unwrap();

    // For specific client headers, include the common client header
    if let Some(common_header) = client_common_header {
        writeln!(&mut out, "#include \"{}\"", common_header).unwrap();
    }
    out.push('\n');

    out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");

    for msg in messages {
        // Determine if this message applies to the current role
        let (applies, mode) = match role {
            Role::Server => {
                // Server: pub->encode, sub->decode
                let mode = match msg.request_type {
                    RequestType::Pub => FunctionMode::EncodeOnly,
                    RequestType::Sub => FunctionMode::DecodeOnly,
                };
                (true, mode)
            }
            Role::ClientCommon => {
                // ClientCommon: only messages with target_client_id == -1
                let applies = msg.target_client_id == -1;
                // Client: pub->decode, sub->encode (opposite of server)
                let mode = match msg.request_type {
                    RequestType::Pub => FunctionMode::DecodeOnly,
                    RequestType::Sub => FunctionMode::EncodeOnly,
                };
                (applies, mode)
            }
            Role::Client(client_id) => {
                // Client: only messages with specific target_client_id (NOT -1, those are in common)
                let applies = msg.target_client_id == client_id;
                // Client: pub->decode, sub->encode (opposite of server)
                let mode = match msg.request_type {
                    RequestType::Pub => FunctionMode::DecodeOnly,
                    RequestType::Sub => FunctionMode::EncodeOnly,
                };
                (applies, mode)
            }
        };

        if applies {
            out.push('\n');
            out.push_str(&generate_message_functions_only(msg, mode));
        }
    }

    out.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n");
    writeln!(&mut out, "#endif /* {} */", header_guard).unwrap();

    out
}

/// Legacy generate function for backwards compatibility.
/// Generates a single header with all encode/decode functions.
pub fn generate(
    metadata: &Metadata,
    messages: &[MessageDefinition],
    input_path: &Path,
    output_path: &Path,
) -> Result<String> {
    let helper_block = load_templates(TargetLanguage::C, TEMPLATE_FILES)?;
    let header_guard = header_guard_name(output_path);

    let mut out = String::new();
    writeln!(&mut out, "/*").unwrap();
    writeln!(&mut out, " * Auto-generated by h6xserial_idl.").unwrap();
    writeln!(&mut out, " * Source: {}", input_path.display()).unwrap();
    if let Some(version) = &metadata.version {
        writeln!(&mut out, " * Protocol version: {}", version).unwrap();
    }
    if let Some(max_address) = metadata.max_address {
        writeln!(&mut out, " * Max address: {}", max_address).unwrap();
    }
    writeln!(&mut out, " */\n").unwrap();

    writeln!(&mut out, "#ifndef {}", header_guard).unwrap();
    writeln!(&mut out, "#define {}\n", header_guard).unwrap();

    out.push_str(
        "#include <stdbool.h>\n#include <stddef.h>\n#include <stdint.h>\n#include <string.h>\n\n",
    );

    out.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n\n");
    out.push_str(&helper_block);

    for msg in messages {
        out.push('\n');
        out.push_str(&generate_message_block_with_mode(msg, FunctionMode::Both));
    }

    out.push_str("\n#ifdef __cplusplus\n}\n#endif\n\n");
    writeln!(&mut out, "#endif /* {} */", header_guard).unwrap();

    Ok(out)
}

fn generate_message_block_with_mode(msg: &MessageDefinition, mode: FunctionMode) -> String {
    let mut out = String::new();
    if let Some(desc) = &msg.description {
        writeln!(&mut out, "/* {} */", desc).unwrap();
    }
    let macro_prefix = to_macro_ident(&msg.name);
    writeln!(
        &mut out,
        "#define H6XSERIAL_MSG_{}_PACKET_ID {}",
        macro_prefix, msg.packet_id
    )
    .unwrap();

    match &msg.body {
        MessageBody::Array(spec) => {
            writeln!(
                &mut out,
                "#define H6XSERIAL_MSG_{}_MAX_LENGTH {}",
                macro_prefix, spec.max_length
            )
            .unwrap();
            if let Some(sector) = spec.sector_bytes {
                writeln!(
                    &mut out,
                    "#define H6XSERIAL_MSG_{}_SECTOR_BYTES {}",
                    macro_prefix, sector
                )
                .unwrap();
            }
            out.push('\n');
            out.push_str(&generate_array_block(msg, spec, mode));
        }
        MessageBody::Scalar(spec) => {
            out.push('\n');
            out.push_str(&generate_scalar_block(msg, spec, mode));
        }
        MessageBody::Struct(spec) => {
            out.push('\n');
            out.push_str(&generate_struct_block(msg, spec, mode));
        }
    }

    out
}

/// Generates only type definitions and macros for a message (for _types.h)
fn generate_message_types_only(msg: &MessageDefinition) -> String {
    let mut out = String::new();
    if let Some(desc) = &msg.description {
        writeln!(&mut out, "/* {} */", desc).unwrap();
    }
    let macro_prefix = to_macro_ident(&msg.name);
    writeln!(
        &mut out,
        "#define H6XSERIAL_MSG_{}_PACKET_ID {}",
        macro_prefix, msg.packet_id
    )
    .unwrap();

    match &msg.body {
        MessageBody::Array(spec) => {
            writeln!(
                &mut out,
                "#define H6XSERIAL_MSG_{}_MAX_LENGTH {}",
                macro_prefix, spec.max_length
            )
            .unwrap();
            if let Some(sector) = spec.sector_bytes {
                writeln!(
                    &mut out,
                    "#define H6XSERIAL_MSG_{}_SECTOR_BYTES {}",
                    macro_prefix, sector
                )
                .unwrap();
            }
            out.push('\n');
            out.push_str(&generate_array_typedef(msg, spec));
        }
        MessageBody::Scalar(spec) => {
            out.push('\n');
            out.push_str(&generate_scalar_typedef(msg, spec));
        }
        MessageBody::Struct(spec) => {
            out.push('\n');
            out.push_str(&generate_struct_typedef_for_types(msg, spec));
        }
    }

    out
}

/// Generates only functions for a message (for _server.h and _client_<id>.h)
fn generate_message_functions_only(msg: &MessageDefinition, mode: FunctionMode) -> String {
    let mut out = String::new();
    if let Some(desc) = &msg.description {
        writeln!(&mut out, "/* {} */", desc).unwrap();
    }

    match &msg.body {
        MessageBody::Array(spec) => {
            out.push_str(&generate_array_functions(msg, spec, mode));
        }
        MessageBody::Scalar(spec) => {
            out.push_str(&generate_scalar_functions(msg, spec, mode));
        }
        MessageBody::Struct(spec) => {
            out.push_str(&generate_struct_functions(msg, spec, mode));
        }
    }

    out
}

/// Generate typedef only for scalar message
fn generate_scalar_typedef(msg: &MessageDefinition, spec: &ScalarSpec) -> String {
    let type_name = type_name(msg);
    format!(
        "typedef struct {{\n    {} value;\n}} {};\n\n",
        spec.primitive.c_type(),
        type_name
    )
}

/// Generate typedef only for array message
fn generate_array_typedef(msg: &MessageDefinition, spec: &ArraySpec) -> String {
    let type_name = type_name(msg);
    let macro_prefix = to_macro_ident(&msg.name);
    let max_macro = format!("H6XSERIAL_MSG_{}_MAX_LENGTH", macro_prefix);
    format!(
        "typedef struct {{\n    size_t length;\n    {} data[{}];\n}} {};\n\n",
        spec.primitive.c_type(),
        max_macro,
        type_name
    )
}

/// Generate typedef only for struct message (wrapper for generate_struct_typedef)
fn generate_struct_typedef_for_types(msg: &MessageDefinition, spec: &StructSpec) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let macro_prefix = format!("H6XSERIAL_MSG_{}", to_macro_ident(&msg.name));
    generate_struct_typedef(&mut out, &type_name, &macro_prefix, spec);
    out.push('\n');
    out
}

/// Generate functions only for scalar message (for _server.h/_client.h)
fn generate_scalar_functions(msg: &MessageDefinition, spec: &ScalarSpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);
    let size = spec.primitive.byte_len();

    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (out_len < {}) {{\n        return 0;\n    }}",
            size
        )
        .unwrap();
        out.push_str(&primitive_encode_stmt(
            spec.primitive,
            spec.endian,
            "msg->value",
            "out_buf",
            "    ",
        ));
        writeln!(&mut out, "    return {};\n}}\n", size).unwrap();
    }

    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");
        writeln!(
            &mut out,
            "    if (data_len != {}) {{\n        return false;\n    }}",
            size
        )
        .unwrap();
        out.push_str(&primitive_decode_stmt(
            spec.primitive,
            spec.endian,
            "msg->value",
            "data",
            "    ",
        ));
        out.push_str("    return true;\n}\n\n");
    }

    out
}

/// Generate functions only for array message (for _server.h/_client.h)
fn generate_array_functions(msg: &MessageDefinition, spec: &ArraySpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);
    let macro_prefix = to_macro_ident(&msg.name);
    let max_macro = format!("H6XSERIAL_MSG_{}_MAX_LENGTH", macro_prefix);
    let elem_size = spec.primitive.byte_len();

    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (msg->length > {}) {{\n        return 0;\n    }}",
            max_macro
        )
        .unwrap();
        writeln!(
            &mut out,
            "    size_t required = msg->length * {};",
            elem_size
        )
        .unwrap();
        out.push_str("    if (out_len < required) {\n        return 0;\n    }\n");
        if elem_size == 1 {
            out.push_str(
                "    if (required > 0) {\n        memcpy(out_buf, msg->data, required);\n    }\n",
            );
            out.push_str("    return required;\n}\n\n");
        } else {
            out.push_str("    size_t offset = 0;\n    for (size_t i = 0; i < msg->length; ++i) {\n");
            out.push_str(&primitive_encode_stmt(
                spec.primitive,
                spec.endian,
                "msg->data[i]",
                "out_buf + offset",
                "        ",
            ));
            writeln!(&mut out, "        offset += {};", elem_size).unwrap();
            out.push_str("    }\n    return offset;\n}\n\n");
        }
    }

    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");
        writeln!(
            &mut out,
            "    if (data_len % {} != 0) {{\n        return false;\n    }}",
            elem_size
        )
        .unwrap();
        writeln!(
            &mut out,
            "    size_t element_count = data_len / {};",
            elem_size
        )
        .unwrap();
        writeln!(
            &mut out,
            "    if (element_count > {}) {{\n        return false;\n    }}",
            max_macro
        )
        .unwrap();
        out.push_str("    msg->length = element_count;\n");
        out.push_str("    if (element_count == 0) {\n");
        if spec.primitive == PrimitiveType::Char {
            out.push_str("        if (");
            out.push_str(&max_macro);
            out.push_str(" > 0) {\n            msg->data[0] = '\\0';\n        }\n");
        }
        out.push_str("        return true;\n    }\n");
        if elem_size == 1 {
            out.push_str("    memcpy(msg->data, data, element_count);\n");
        } else {
            out.push_str("    size_t offset = 0;\n    for (size_t i = 0; i < element_count; ++i) {\n");
            out.push_str(&primitive_decode_stmt(
                spec.primitive,
                spec.endian,
                "msg->data[i]",
                "data + offset",
                "        ",
            ));
            writeln!(&mut out, "        offset += {};", elem_size).unwrap();
            out.push_str("    }\n");
        }
        if spec.primitive == PrimitiveType::Char {
            out.push_str("    if (element_count < ");
            out.push_str(&max_macro);
            out.push_str(") {\n        msg->data[element_count] = '\\0';\n    }\n");
        }
        out.push_str("    return true;\n}\n\n");
    }

    out
}

/// Generate functions only for struct message (for _server.h/_client.h)
fn generate_struct_functions(msg: &MessageDefinition, spec: &StructSpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);
    let macro_prefix = format!("H6XSERIAL_MSG_{}", to_macro_ident(&msg.name));

    let has_variable_arrays = struct_has_variable_arrays(spec);
    let max_size = struct_byte_len(spec);
    let min_size = struct_min_byte_len(spec);

    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (out_len < {}) {{\n        return 0;\n    }}",
            max_size
        )
        .unwrap();
        out.push_str("    size_t offset = 0;\n");
        generate_field_encode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ");
        out.push_str("    return offset;\n}\n\n");
    }

    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");

        if has_variable_arrays {
            writeln!(
                &mut out,
                "    if (data_len < {}) {{\n        return false;\n    }}",
                min_size
            )
            .unwrap();
            writeln!(
                &mut out,
                "    if (data_len > {}) {{\n        return false;\n    }}",
                max_size
            )
            .unwrap();
            out.push_str("    size_t offset = 0;\n");
            out.push_str("    size_t remaining = data_len;\n");
            writeln!(&mut out, "    remaining -= {};", min_size).unwrap();
            generate_field_decode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ", Some("remaining"));
        } else {
            writeln!(
                &mut out,
                "    if (data_len != {}) {{\n        return false;\n    }}",
                max_size
            )
            .unwrap();
            out.push_str("    size_t offset = 0;\n");
            generate_field_decode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ", None);
        }
        out.push_str("    return true;\n}\n\n");
    }

    out
}

fn generate_scalar_block(msg: &MessageDefinition, spec: &ScalarSpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);

    writeln!(
        &mut out,
        "typedef struct {{\n    {} value;\n}} {};\n",
        spec.primitive.c_type(),
        type_name
    )
    .unwrap();

    let size = spec.primitive.byte_len();

    // Generate encode function if needed
    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (out_len < {}) {{\n        return 0;\n    }}",
            size
        )
        .unwrap();
        out.push_str(&primitive_encode_stmt(
            spec.primitive,
            spec.endian,
            "msg->value",
            "out_buf",
            "    ",
        ));
        writeln!(&mut out, "    return {};\n}}\n", size).unwrap();
    }

    // Generate decode function if needed
    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");
        writeln!(
            &mut out,
            "    if (data_len != {}) {{\n        return false;\n    }}",
            size
        )
        .unwrap();
        out.push_str(&primitive_decode_stmt(
            spec.primitive,
            spec.endian,
            "msg->value",
            "data",
            "    ",
        ));
        out.push_str("    return true;\n}\n\n");
    }

    out
}

fn generate_array_block(msg: &MessageDefinition, spec: &ArraySpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);
    let macro_prefix = to_macro_ident(&msg.name);
    let max_macro = format!("H6XSERIAL_MSG_{}_MAX_LENGTH", macro_prefix);

    writeln!(
        &mut out,
        "typedef struct {{\n    size_t length;\n    {} data[{}];\n}} {};\n",
        spec.primitive.c_type(),
        max_macro,
        type_name
    )
    .unwrap();

    let elem_size = spec.primitive.byte_len();

    // Generate encode function if needed
    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (msg->length > {}) {{\n        return 0;\n    }}",
            max_macro
        )
        .unwrap();
        writeln!(
            &mut out,
            "    size_t required = msg->length * {};",
            elem_size
        )
        .unwrap();
        out.push_str("    if (out_len < required) {\n        return 0;\n    }\n");
        if elem_size == 1 {
            out.push_str(
                "    if (required > 0) {\n        memcpy(out_buf, msg->data, required);\n    }\n",
            );
            out.push_str("    return required;\n}\n\n");
        } else {
            out.push_str("    size_t offset = 0;\n    for (size_t i = 0; i < msg->length; ++i) {\n");
            out.push_str(&primitive_encode_stmt(
                spec.primitive,
                spec.endian,
                "msg->data[i]",
                "out_buf + offset",
                "        ",
            ));
            writeln!(&mut out, "        offset += {};", elem_size).unwrap();
            out.push_str("    }\n    return offset;\n}\n\n");
        }
    }

    // Generate decode function if needed
    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");
        writeln!(
            &mut out,
            "    if (data_len % {} != 0) {{\n        return false;\n    }}",
            elem_size
        )
        .unwrap();
        writeln!(
            &mut out,
            "    size_t element_count = data_len / {};",
            elem_size
        )
        .unwrap();
        writeln!(
            &mut out,
            "    if (element_count > {}) {{\n        return false;\n    }}",
            max_macro
        )
        .unwrap();
        out.push_str("    msg->length = element_count;\n");
        out.push_str("    if (element_count == 0) {\n");
        if spec.primitive == PrimitiveType::Char {
            out.push_str("        if (");
            out.push_str(&max_macro);
            out.push_str(" > 0) {\n            msg->data[0] = '\\0';\n        }\n");
        }
        out.push_str("        return true;\n    }\n");
        if elem_size == 1 {
            out.push_str("    memcpy(msg->data, data, element_count);\n");
        } else {
            out.push_str("    size_t offset = 0;\n    for (size_t i = 0; i < element_count; ++i) {\n");
            out.push_str(&primitive_decode_stmt(
                spec.primitive,
                spec.endian,
                "msg->data[i]",
                "data + offset",
                "        ",
            ));
            writeln!(&mut out, "        offset += {};", elem_size).unwrap();
            out.push_str("    }\n");
        }
        if spec.primitive == PrimitiveType::Char {
            out.push_str("    if (element_count < ");
            out.push_str(&max_macro);
            out.push_str(") {\n        msg->data[element_count] = '\\0';\n    }\n");
        }
        out.push_str("    return true;\n}\n\n");
    }

    out
}

/// Calculates the total byte size of a struct field (recursively for nested structs).
/// For array fields, returns the maximum byte size (max_length * element_size).
fn field_byte_len(field: &StructField) -> usize {
    match &field.field_type {
        StructFieldType::Primitive(prim) => prim.byte_len(),
        StructFieldType::Array(arr) => arr.max_length * arr.primitive.byte_len(),
        StructFieldType::Nested(nested) => struct_byte_len(nested),
    }
}

/// Checks if a struct contains any variable-length array fields (recursively).
fn struct_has_variable_arrays(spec: &StructSpec) -> bool {
    spec.fields.iter().any(|f| match &f.field_type {
        StructFieldType::Array(_) => true,
        StructFieldType::Nested(nested) => struct_has_variable_arrays(nested),
        StructFieldType::Primitive(_) => false,
    })
}

/// Calculates the minimum byte size of a struct (arrays contribute 0 minimum).
fn struct_min_byte_len(spec: &StructSpec) -> usize {
    spec.fields.iter().map(|f| match &f.field_type {
        StructFieldType::Primitive(prim) => prim.byte_len(),
        StructFieldType::Array(_) => 0,
        StructFieldType::Nested(nested) => struct_min_byte_len(nested),
    }).sum()
}

/// Calculates the total byte size of a struct (recursively for nested structs).
fn struct_byte_len(spec: &StructSpec) -> usize {
    spec.fields.iter().map(field_byte_len).sum()
}

/// Generates a nested struct type name.
fn nested_struct_type_name(parent_type_name: &str, field_name: &str) -> String {
    format!("{}_{}_t", parent_type_name.trim_end_matches("_t"), to_snake_case(field_name))
}

/// Generates typedef for a struct, including nested struct typedefs.
/// Also emits #define macros for array field max lengths.
fn generate_struct_typedef(
    out: &mut String,
    type_name: &str,
    macro_prefix: &str,
    spec: &StructSpec,
) {
    // First, generate typedefs for any nested structs
    for field in &spec.fields {
        if let StructFieldType::Nested(nested_spec) = &field.field_type {
            let nested_type = nested_struct_type_name(type_name, &field.name);
            let nested_macro_prefix = format!("{}_{}", macro_prefix, to_macro_ident(&field.name));
            generate_struct_typedef(out, &nested_type, &nested_macro_prefix, nested_spec);
        }
    }

    // Generate #define macros for array field max lengths
    for field in &spec.fields {
        if let StructFieldType::Array(arr) = &field.field_type {
            let field_macro = to_macro_ident(&field.name);
            writeln!(
                out,
                "#define {}_{}_MAX_LENGTH {}",
                macro_prefix, field_macro, arr.max_length
            )
            .unwrap();
        }
    }

    // Then generate this struct's typedef
    writeln!(out, "typedef struct {{").unwrap();
    for field in &spec.fields {
        let field_ident = to_snake_case(&field.name);
        match &field.field_type {
            StructFieldType::Primitive(prim) => {
                writeln!(out, "    {} {};", prim.c_type(), field_ident).unwrap();
            }
            StructFieldType::Array(arr) => {
                let field_macro = to_macro_ident(&field.name);
                writeln!(out, "    size_t {}_length;", field_ident).unwrap();
                writeln!(
                    out,
                    "    {} {}[{}_{}_MAX_LENGTH];",
                    arr.primitive.c_type(),
                    field_ident,
                    macro_prefix,
                    field_macro
                )
                .unwrap();
            }
            StructFieldType::Nested(_) => {
                let nested_type = nested_struct_type_name(type_name, &field.name);
                writeln!(out, "    {} {};", nested_type, field_ident).unwrap();
            }
        }
    }
    writeln!(out, "}} {};\n", type_name).unwrap();
}

/// Generates encode statements for struct fields (recursively for nested structs).
fn generate_field_encode_stmts(
    out: &mut String,
    fields: &[StructField],
    parent_accessor: &str,
    macro_prefix: &str,
    indent: &str,
) {
    for field in fields {
        let field_ident = to_snake_case(&field.name);
        let accessor = format!("{}{}", parent_accessor, field_ident);
        match &field.field_type {
            StructFieldType::Primitive(prim) => {
                out.push_str(&primitive_encode_stmt(
                    *prim,
                    field.endian,
                    &accessor,
                    "out_buf + offset",
                    indent,
                ));
                writeln!(out, "{}offset += {};", indent, prim.byte_len()).unwrap();
            }
            StructFieldType::Array(arr) => {
                let field_macro = to_macro_ident(&field.name);
                let max_macro = format!("{}_{}_MAX_LENGTH", macro_prefix, field_macro);
                let length_accessor = format!("{}{}_length", parent_accessor, field_ident);
                let elem_size = arr.primitive.byte_len();

                // Encode array elements
                writeln!(out, "{}for (size_t i = 0; i < {} && i < {}; ++i) {{", indent, length_accessor, max_macro).unwrap();
                let elem_accessor = format!("{}[i]", accessor);
                let next_indent = format!("{}    ", indent);
                out.push_str(&primitive_encode_stmt(
                    arr.primitive,
                    field.endian,
                    &elem_accessor,
                    "out_buf + offset",
                    &next_indent,
                ));
                writeln!(out, "{}    offset += {};", indent, elem_size).unwrap();
                writeln!(out, "{}}}", indent).unwrap();
            }
            StructFieldType::Nested(nested_spec) => {
                // Recursively encode nested struct fields
                let nested_accessor = format!("{}.", accessor);
                let nested_macro_prefix = format!("{}_{}", macro_prefix, to_macro_ident(&field.name));
                generate_field_encode_stmts(out, &nested_spec.fields, &nested_accessor, &nested_macro_prefix, indent);
            }
        }
    }
}

/// Generates decode statements for struct fields (recursively for nested structs).
/// For structs with variable-length arrays, we need to track remaining bytes.
fn generate_field_decode_stmts(
    out: &mut String,
    fields: &[StructField],
    parent_accessor: &str,
    macro_prefix: &str,
    indent: &str,
    remaining_var: Option<&str>,
) {
    for field in fields {
        let field_ident = to_snake_case(&field.name);
        let accessor = format!("{}{}", parent_accessor, field_ident);
        match &field.field_type {
            StructFieldType::Primitive(prim) => {
                out.push_str(&primitive_decode_stmt(
                    *prim,
                    field.endian,
                    &accessor,
                    "data + offset",
                    indent,
                ));
                writeln!(out, "{}offset += {};", indent, prim.byte_len()).unwrap();
            }
            StructFieldType::Array(arr) => {
                let field_macro = to_macro_ident(&field.name);
                let max_macro = format!("{}_{}_MAX_LENGTH", macro_prefix, field_macro);
                let length_accessor = format!("{}{}_length", parent_accessor, field_ident);
                let elem_size = arr.primitive.byte_len();

                // Calculate how many elements we can decode based on remaining bytes
                if let Some(rem_var) = remaining_var {
                    writeln!(out, "{}{{", indent).unwrap();
                    writeln!(out, "{}    size_t elem_count = {} / {};", indent, rem_var, elem_size).unwrap();
                    writeln!(out, "{}    if (elem_count > {}) {{", indent, max_macro).unwrap();
                    writeln!(out, "{}        elem_count = {};", indent, max_macro).unwrap();
                    writeln!(out, "{}    }}", indent).unwrap();
                    writeln!(out, "{}    {} = elem_count;", indent, length_accessor).unwrap();
                    writeln!(out, "{}    for (size_t i = 0; i < elem_count; ++i) {{", indent).unwrap();
                    let elem_accessor = format!("{}[i]", accessor);
                    out.push_str(&primitive_decode_stmt(
                        arr.primitive,
                        field.endian,
                        &elem_accessor,
                        "data + offset",
                        &format!("{}        ", indent),
                    ));
                    writeln!(out, "{}        offset += {};", indent, elem_size).unwrap();
                    writeln!(out, "{}    }}", indent).unwrap();
                    writeln!(out, "{}}}", indent).unwrap();
                } else {
                    // No remaining var tracking - decode max elements
                    writeln!(out, "{}{} = {};", indent, length_accessor, max_macro).unwrap();
                    writeln!(out, "{}for (size_t i = 0; i < {}; ++i) {{", indent, max_macro).unwrap();
                    let elem_accessor = format!("{}[i]", accessor);
                    let next_indent = format!("{}    ", indent);
                    out.push_str(&primitive_decode_stmt(
                        arr.primitive,
                        field.endian,
                        &elem_accessor,
                        "data + offset",
                        &next_indent,
                    ));
                    writeln!(out, "{}    offset += {};", indent, elem_size).unwrap();
                    writeln!(out, "{}}}", indent).unwrap();
                }
            }
            StructFieldType::Nested(nested_spec) => {
                // Recursively decode nested struct fields
                let nested_accessor = format!("{}.", accessor);
                let nested_macro_prefix = format!("{}_{}", macro_prefix, to_macro_ident(&field.name));
                generate_field_decode_stmts(out, &nested_spec.fields, &nested_accessor, &nested_macro_prefix, indent, remaining_var);
            }
        }
    }
}

fn generate_struct_block(msg: &MessageDefinition, spec: &StructSpec, mode: FunctionMode) -> String {
    let mut out = String::new();
    let type_name = type_name(msg);
    let encode_name = encode_fn_name(msg);
    let decode_name = decode_fn_name(msg);
    let macro_prefix = format!("H6XSERIAL_MSG_{}", to_macro_ident(&msg.name));

    // Generate typedef(s) for struct and nested structs
    generate_struct_typedef(&mut out, &type_name, &macro_prefix, spec);

    let has_variable_arrays = struct_has_variable_arrays(spec);
    let max_size = struct_byte_len(spec);
    let min_size = struct_min_byte_len(spec);

    // Generate encode function if needed
    if mode == FunctionMode::EncodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline size_t {}(const {} *msg, uint8_t *out_buf, const size_t out_len) {{",
            encode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !out_buf) {\n        return 0;\n    }\n");
        writeln!(
            &mut out,
            "    if (out_len < {}) {{\n        return 0;\n    }}",
            max_size
        )
        .unwrap();
        out.push_str("    size_t offset = 0;\n");
        generate_field_encode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ");
        out.push_str("    return offset;\n}\n\n");
    }

    // Generate decode function if needed
    if mode == FunctionMode::DecodeOnly || mode == FunctionMode::Both {
        writeln!(
            &mut out,
            "static inline bool {}({} *msg, const uint8_t *data, const size_t data_len) {{",
            decode_name, type_name
        )
        .unwrap();
        out.push_str("    if (!msg || !data) {\n        return false;\n    }\n");

        if has_variable_arrays {
            // For structs with variable-length arrays, check minimum size
            writeln!(
                &mut out,
                "    if (data_len < {}) {{\n        return false;\n    }}",
                min_size
            )
            .unwrap();
            writeln!(
                &mut out,
                "    if (data_len > {}) {{\n        return false;\n    }}",
                max_size
            )
            .unwrap();
            out.push_str("    size_t offset = 0;\n");
            out.push_str("    size_t remaining = data_len;\n");
            // Calculate remaining bytes after fixed fields for the array
            writeln!(&mut out, "    remaining -= {};", min_size).unwrap();
            generate_field_decode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ", Some("remaining"));
        } else {
            writeln!(
                &mut out,
                "    if (data_len != {}) {{\n        return false;\n    }}",
                max_size
            )
            .unwrap();
            out.push_str("    size_t offset = 0;\n");
            generate_field_decode_stmts(&mut out, &spec.fields, "msg->", &macro_prefix, "    ", None);
        }
        out.push_str("    return true;\n}\n\n");
    }

    out
}

fn primitive_encode_stmt(
    primitive: PrimitiveType,
    endian: Endian,
    source: &str,
    dest_ptr: &str,
    indent: &str,
) -> String {
    match primitive {
        PrimitiveType::Bool => format!(
            "{indent}({dest})[0] = ({src}) ? 1 : 0;\n",
            indent = indent,
            dest = dest_ptr,
            src = source
        ),
        PrimitiveType::Char | PrimitiveType::Int8 | PrimitiveType::Uint8 => format!(
            "{indent}({dest})[0] = (uint8_t)({src});\n",
            indent = indent,
            dest = dest_ptr,
            src = source
        ),
        PrimitiveType::Int16 => format!(
            "{indent}h6xserial_write_u16_{suffix}((uint16_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Uint16 => format!(
            "{indent}h6xserial_write_u16_{suffix}((uint16_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Int32 => format!(
            "{indent}h6xserial_write_u32_{suffix}((uint32_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Uint32 => format!(
            "{indent}h6xserial_write_u32_{suffix}((uint32_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Int64 => format!(
            "{indent}h6xserial_write_u64_{suffix}((uint64_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Uint64 => format!(
            "{indent}h6xserial_write_u64_{suffix}((uint64_t)({src}), {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Float32 => format!(
            "{indent}h6xserial_write_f32_{suffix}({src}, {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
        PrimitiveType::Float64 => format!(
            "{indent}h6xserial_write_f64_{suffix}({src}, {dest});\n",
            indent = indent,
            suffix = endian.suffix(),
            src = source,
            dest = dest_ptr
        ),
    }
}

fn primitive_decode_stmt(
    primitive: PrimitiveType,
    endian: Endian,
    dest: &str,
    src_ptr: &str,
    indent: &str,
) -> String {
    match primitive {
        PrimitiveType::Bool => format!(
            "{indent}{dest} = (({src})[0]) != 0;\n",
            indent = indent,
            dest = dest,
            src = src_ptr
        ),
        PrimitiveType::Char => format!(
            "{indent}{dest} = (char)(({src})[0]);\n",
            indent = indent,
            dest = dest,
            src = src_ptr
        ),
        PrimitiveType::Int8 => format!(
            "{indent}{dest} = (int8_t)(({src})[0]);\n",
            indent = indent,
            dest = dest,
            src = src_ptr
        ),
        PrimitiveType::Uint8 => format!(
            "{indent}{dest} = (uint8_t)(({src})[0]);\n",
            indent = indent,
            dest = dest,
            src = src_ptr
        ),
        PrimitiveType::Int16 => format!(
            "{indent}{dest} = (int16_t)h6xserial_read_u16_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Uint16 => format!(
            "{indent}{dest} = h6xserial_read_u16_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Int32 => format!(
            "{indent}{dest} = (int32_t)h6xserial_read_u32_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Uint32 => format!(
            "{indent}{dest} = h6xserial_read_u32_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Int64 => format!(
            "{indent}{dest} = (int64_t)h6xserial_read_u64_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Uint64 => format!(
            "{indent}{dest} = h6xserial_read_u64_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Float32 => format!(
            "{indent}{dest} = h6xserial_read_f32_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
        PrimitiveType::Float64 => format!(
            "{indent}{dest} = h6xserial_read_f64_{suffix}({src});\n",
            indent = indent,
            dest = dest,
            suffix = endian.suffix(),
            src = src_ptr
        ),
    }
}

fn type_name(msg: &MessageDefinition) -> String {
    format!("h6xserial_msg_{}_t", to_snake_case(&msg.name))
}

fn encode_fn_name(msg: &MessageDefinition) -> String {
    format!("h6xserial_msg_{}_encode", to_snake_case(&msg.name))
}

fn decode_fn_name(msg: &MessageDefinition) -> String {
    format!("h6xserial_msg_{}_decode", to_snake_case(&msg.name))
}

fn header_guard_name(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("generated_header");
    header_guard_name_from_str(file_name)
}

fn header_guard_name_from_str(file_name: &str) -> String {
    let mut guard = String::new();
    for ch in file_name.chars() {
        if ch.is_ascii_alphanumeric() {
            guard.push(ch.to_ascii_uppercase());
        } else {
            guard.push('_');
        }
    }
    if !guard.ends_with("_H") {
        if !guard.ends_with('_') {
            guard.push('_');
        }
        guard.push('H');
    }
    guard
}
