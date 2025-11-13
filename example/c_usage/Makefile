# Makefile for h6xserial_idl C usage example

CC = gcc
CFLAGS = -Wall -Wextra -Werror -std=c99 -O2 -I.
TARGET = example
SOURCES = example.c
HEADERS = sensor_messages.h
JSON_DEF = sensor_messages.json

# H6XSERIAL_IDL tool (can be overridden)
H6XSERIAL_IDL ?= cargo run --manifest-path=../../Cargo.toml --

.PHONY: all clean regenerate run test help

all: $(TARGET)

# Build the example program
$(TARGET): $(SOURCES) $(HEADERS)
	$(CC) $(CFLAGS) -o $(TARGET) $(SOURCES)
	@echo "Build complete: $(TARGET)"

# Generate the header file from JSON definition
$(HEADERS): $(JSON_DEF)
	@echo "Generating C header from JSON definition..."
	$(H6XSERIAL_IDL) c $(JSON_DEF) $(HEADERS)
	@echo "Generated: $(HEADERS)"

# Regenerate the header file
regenerate:
	@echo "Regenerating C header from JSON definition..."
	$(H6XSERIAL_IDL) c $(JSON_DEF) $(HEADERS)
	@echo "Regenerated: $(HEADERS)"

# Build and run the example
run: $(TARGET)
	@echo "Running example..."
	@./$(TARGET)

# Run tests (same as run in this case)
test: run

# Clean build artifacts
clean:
	rm -f $(TARGET)
	@echo "Cleaned build artifacts"

# Clean everything including generated header
distclean: clean
	rm -f $(HEADERS)
	@echo "Cleaned all generated files"

# Help target
help:
	@echo "h6xserial_idl C Usage Example Makefile"
	@echo ""
	@echo "Targets:"
	@echo "  all         - Build the example program (default)"
	@echo "  regenerate  - Regenerate the header from JSON definition"
	@echo "  run         - Build and run the example"
	@echo "  test        - Run tests (same as run)"
	@echo "  clean       - Remove build artifacts"
	@echo "  distclean   - Remove all generated files including header"
	@echo "  help        - Show this help message"
	@echo ""
	@echo "Variables:"
	@echo "  CC          - C compiler (default: gcc)"
	@echo "  CFLAGS      - Compiler flags"
	@echo "  H6XSERIAL_IDL - Path to h6xserial_idl tool"
