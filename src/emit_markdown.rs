//! Markdown documentation generator for message definitions.
//!
//! Generates protocol documentation in Markdown format similar to concept/protocol.md

use std::fmt::Write as FmtWrite;
use std::path::Path;

use anyhow::Result;

use crate::{MessageDefinition, Metadata};

/// Generates Markdown documentation for command definitions.
///
/// # Arguments
/// * `metadata` - Protocol metadata (version, max_address)
/// * `messages` - List of message definitions to document
/// * `input_path` - Path to input JSON file (for documentation)
///
/// # Returns
/// * `Ok(String)` - Generated Markdown documentation
/// * `Err(...)` - Generation error with context
///
/// # Generated Documentation
/// - Protocol overview with metadata
/// - Command definitions table (sorted by packet_id)
/// - Includes command names, values, and descriptions
pub fn generate(
    metadata: &Metadata,
    messages: &[MessageDefinition],
    input_path: &Path,
) -> Result<String> {
    let mut out = String::new();

    // Generate header
    writeln!(&mut out, "# Command Definitions").unwrap();
    writeln!(&mut out).unwrap();
    writeln!(&mut out, "Auto-generated from: `{}`", input_path.display()).unwrap();

    if let Some(version) = &metadata.version {
        writeln!(&mut out, "Protocol version: {}", version).unwrap();
    }
    if let Some(max_address) = metadata.max_address {
        writeln!(&mut out, "Max address: {}", max_address).unwrap();
    }
    writeln!(&mut out).unwrap();

    // Group commands by ranges
    let base_commands: Vec<_> = messages.iter().filter(|m| m.packet_id < 20).collect();
    let custom_commands: Vec<_> = messages.iter().filter(|m| m.packet_id >= 20).collect();

    // Generate Base Commands section
    if !base_commands.is_empty() {
        generate_command_section(&mut out, "Base Commands (0~19)", &base_commands)?;
    }

    // Generate Custom Commands section
    if !custom_commands.is_empty() {
        generate_command_section(&mut out, "Custom Commands (20+)", &custom_commands)?;
    }

    Ok(out)
}

fn generate_command_section(
    out: &mut String,
    title: &str,
    commands: &[&MessageDefinition],
) -> Result<()> {
    writeln!(out, "## {}", title).unwrap();
    writeln!(out).unwrap();

    if commands.is_empty() {
        writeln!(out, "*No commands defined in this range.*").unwrap();
        writeln!(out).unwrap();
        return Ok(());
    }

    // Generate table header
    writeln!(out, "| Command | Value | Description |").unwrap();
    writeln!(out, "|---------|-------|-------------|").unwrap();

    // Generate table rows
    for msg in commands {
        let command_name = format_command_name(&msg.name);
        let description = msg
            .description
            .as_ref()
            .map(|s| s.as_str())
            .unwrap_or("No description");

        writeln!(
            out,
            "| `{}` | {} | {} |",
            command_name, msg.packet_id, description
        )
        .unwrap();
    }

    writeln!(out).unwrap();
    Ok(())
}

fn format_command_name(name: &str) -> String {
    // Convert to SCREAMING_SNAKE_CASE for command names
    let mut result = String::new();
    let mut last_was_underscore = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            let upper = ch.to_ascii_uppercase();
            if result.is_empty() && upper.is_ascii_digit() {
                result.push_str("CMD_");
            }
            result.push(upper);
            last_was_underscore = false;
        } else if !last_was_underscore && !result.is_empty() {
            result.push('_');
            last_was_underscore = true;
        }
    }

    if result.ends_with('_') {
        result.pop();
    }

    // Add CMD_ prefix if not already present (case insensitive check)
    let upper_result = result.to_uppercase();
    if !upper_result.starts_with("CMD_") {
        result = format!("CMD_{}", result);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_command_name() {
        assert_eq!(format_command_name("ping"), "CMD_PING");
        assert_eq!(
            format_command_name("internal_led_on_off"),
            "CMD_INTERNAL_LED_ON_OFF"
        );
        assert_eq!(format_command_name("reboot_device"), "CMD_REBOOT_DEVICE");
        assert_eq!(
            format_command_name("request_general_status"),
            "CMD_REQUEST_GENERAL_STATUS"
        );
        // If the input already starts with "cmd_", it becomes "CMD_" when uppercased,
        // so we should not add the prefix again
        assert_eq!(
            format_command_name("cmd_firmware_version"),
            "CMD_FIRMWARE_VERSION"
        );
    }
}
