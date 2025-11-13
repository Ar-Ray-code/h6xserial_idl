# h6xserial_idl

[![Testing](https://github.com/Ar-Ray-code/h6xserial_idl/actions/workflows/test.yml/badge.svg)](https://github.com/Ar-Ray-code/h6xserial_idl/actions/workflows/test.yml)

A Rust-based code generator that reads JSON intermediate representations (like `msgs/intermediate_msg.json`) and generates serializer/deserializer headers for C99.

## Supported Languages
- C99
- C++ (TODO)
- Python (TODO)
- Rust (TODO)

## How It Works

1. Reads JSON input and parses metadata and message definitions into internal structures.
2. Switches templates and code generators based on the specified target language.
3. For C, generates C99 headers with header guards, encode/decode functions, and helper functions.
4. Creates output directories if they don't exist and writes the generated files.

Templates are located in `src/msg_template/<lang>/`, where language-specific helper functions and common code are stored.

## Usage

### Running the Generator

```bash
# Generate C99 header (default)
cargo run

# Generate documentation
cargo run -- --export_docs

# Specify input/output paths
cargo run -- [--export_docs] [input_json] [output_path]
```

- If language is omitted, `c` is the default.
- If input path is omitted, it looks for `msgs/intermediate_msg.json` (or `../msgs/intermediate_msg.json`).
- If output path is omitted, it uses language-specific default paths (C: `generated_c/seridl_generated_messages.h`, docs: `docs/COMMANDS.md`).

### Documentation Generation

Use the `--export_docs` flag to automatically generate command documentation in Markdown format:

```bash
# Generate documentation to default location (docs/COMMANDS.md)
cargo run -- --export_docs

# Specify custom input and output paths
cargo run -- --export_docs msgs/intermediate_msg.json docs/MY_COMMANDS.md
```

The generated documentation includes:
- Command definitions table sorted by packet ID
- Base Commands (0~19) and Custom Commands (20+) sections
- Command names, values, and descriptions in a readable format

Example output:

```markdown
## Base Commands (0~19)

| Command | Value | Description |
|---------|-------|-------------|
| `CMD_PING` | 0 | Ping/keep-alive command |
| `CMD_INTERNAL_LED_ON_OFF` | 1 | Toggle internal LED |
| `CMD_REBOOT_DEVICE` | 2 | Reboot target device |
...
```

### Installing to `/usr/local/bin`

To avoid running via `cargo run` every time, install the binary to `/usr/local/bin`:

```bash
cd h6xserial_idl
cargo build --release
sudo install -m 0755 target/release/h6xserial_idl /usr/local/bin/h6xserial_idl
```

If `install` is not available, use `sudo cp` and `sudo chmod 755` instead. After installation, you can run `h6xserial_idl` from anywhere.

### Intermediate Representation

Define messages in JSON format like `msgs/intermediate_msg.json`. Example:

```json
{
  "version": "0.0.1",
  "max_address": 255,
  "get_temperatures": {
    "packet_id": 20,
    "msg_type": "float32",
    "array": true,
    "endianess": "big",
    "max_length": 4,
    "msg_desc": "Get temperature readings"
  }
}
```

- For `msg_type: "struct"`, enumerate fields in a `fields` object.
- For arrays (`array: true`), `max_length` is required.
- `endianess` can be `little` or `big` (defaults to little if omitted).

### Output

- C99: Generates `typedef`, `#define`, and `static inline` functions in `generated_c/seridl_generated_messages.h`.
- Documentation: Generates Markdown documentation in `docs/COMMANDS.md` when using `--export_docs`.

## License

See LICENSE file for details.
