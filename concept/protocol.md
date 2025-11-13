# h6xserial_idl - Dynamic Serial Interface Protocol Specification

## Overview

The `h6xserial_idl` Dynamic Serial Interface Protocol is a variable-length packet-based communication protocol designed for reliable data transmission between host and client devices over serial connections. It provides low overhead, CRC-based error detection, and multi-architecture support (C and C++).

## Protocol Architecture

### Communication Model
- **Host**: Initiates communication by sending commands
- **Client**: Responds to host commands with status and data

### Packet Structure

Each packet consists of the following fields:

```
| Header | Client ID | Command/Status | Data Length | Data | CRC8 |
|   1B   |    1B     |       1B       |     1B      | 1-251B | 1B |
```

**Total packet size**: 5 + Data Length bytes (minimum 6 bytes, maximum 256 bytes)

#### Field Descriptions

| Field | Size | Description |
|-------|------|-------------|
| **Header** | 1 byte | Packet type identifier<br>- `#` (0x23): Host packet<br>- `$` (0x24): Client packet |
| **Client ID** | 1 byte | Target device identifier (0x01-0xFF) |
| **Command/Status** | 1 byte | Command code (host) or status code (client) |
| **Data Length** | 1 byte | Length of data payload (1-251 bytes) |
| **Data** | 1-251 bytes | Payload data |
| **CRC8** | 1 byte | CRC-8 checksum for error detection |

### CRC-8 Calculation

The protocol uses CRC-8 with polynomial 0x8C (Dallas/Maxim). The CRC is calculated over:
- Header (1 byte)
- Client ID (1 byte) 
- Command/Status (1 byte)
- Data Length (1 byte)
- Data payload (variable length)

**CRC-8 Algorithm**:
```c
uint8_t crc8_calculate(const uint8_t * input, const uint16_t len)
{
  uint8_t crc = 0;
  for (uint16_t i = 0; i < len; i++) {
    uint8_t extract = input[i];
    for (uint8_t j = 8; j > 0; j--) {
      uint8_t sum = (crc ^ extract) & 0x01;
      crc >>= 1;
      if (sum) {
        crc ^= 0x8C;
      }
      extract >>= 1;
    }
  }
  return crc;
}
```

## Command Definitions

### Base Commands (0~19)

These commands are predefined and used for basic operations. Each command has a unique identifier.

| Command | Value | Description |
|---------|-------|-------------|
| `CMD_PING` | 0 | Ping/keep-alive command |
| `CMD_INTERNAL_LED_ON_OFF` | 1 | Toggle internal LED |
| `CMD_REBOOT_DEVICE` | 2 | Reboot target device |
| `CMD_REQUEST_GENERAL_STATUS` | 3 | Request device status |
| `CMD_REQUEST_FIRMWARE_VERSION` | 4 | Request firmware version |
| `CMD_REQUEST_DEVICE_TICK` | 10 | Request device tick counter |
| `CMD_REQUEST_INTERNAL_ID` | 11 | Request internal device ID |
| `CMD_REQUEST_FIRMWARE_WRITE_DATE` | 12 | Request firmware build date |
| `CMD_REQUEST_DEVICE_VENDOR` | 13 | Request device vendor info |
| `CMD_REQUEST_DEVICE_NAME` | 14 | Request device name |
| `CMD_REQUEST_CURRENT_STATE` | 15 | Request current device state |

### Custom Commands (20+)
Applications can define custom commands starting from 20 (0x14) onwards. These commands should be documented in the application-specific protocol documentation.

## Status/Error Codes

### Standard Error Codes
| Code | Value | Description |
|------|-------|-------------|
| `ERR_SUCCESS` | 0 | Operation successful |
| `ERR_FAILURE` | 1 | General failure |
| `ERR_UNKNOWN_COMMAND` | 2 | Unknown command received |
| `ERR_CRC_ERROR` | 3 | CRC validation failed |
| `ERR_TIMEOUT` | 4 | Operation timeout |
| `ERR_BUSY` | 5 | Device busy |
| `ERR_BUFFER_FULL` | 6 | Buffer overflow |
| `ERR_INVALID_PACKET` | 7 | Invalid packet format |
| `ERR_NOT_IMPLEMENTED` | 8 | Feature not implemented |
| `ERR_OTHER` | 255 | Other error |


## Communication Flow

### Basic Request-Response Pattern

1. **Host -> Client**: Command packet with header `#`
2. **Client -> Host**: Response packet with header `$`

### Example Communication

**Host Request (Ping)**:
```
Header: # (0x23)
Client ID: 0x01
Command: 0x00 (CMD_PING)
Data Length: 0x01
Data: 0x00
CRC8: 0x8A
```

**Client Response**:
```
Header: $ (0x24)
Client ID: 0x01
Status: 0x00 (ERR_SUCCESS)
Data Length: 0x01
Data: 0x00
CRC8: 0x15
```

## Packet Validation

### Validation Rules
1. **Header Check**: Must be `#` or `$`
2. **Client ID Match**: Must match expected client ID
3. **Length Validation**: Data length must be 1-251 bytes
4. **CRC Verification**: Calculated CRC must match received CRC
5. **Packet Size**: Total received length must match expected packet size

### Error Handling
- Invalid packets are discarded
- CRC mismatches result in `ERR_CRC_ERROR`
- Unknown commands return `ERR_UNKNOWN_COMMAND`
- Timeout conditions return `ERR_TIMEOUT`

## Implementation Guidelines

### Host Implementation
```c
// Send command to client
Packet pkt = init_packet();
pkt.client_id = 0x01;
pkt.mode = SERIAL_MODE_HOST;
pkt.command = CMD_PING;
pkt.data_len = 1;
pkt.data[0] = 0x00;

char send_buffer[256];
if (create_packet(&pkt, send_buffer)) {
    // Send packet via serial
    serial_write(send_buffer, pkt.data_len + ADDITIONAL_PACKET_LENGTH);
}
```

### Client Implementation
```c
// Receive and process packet
char recv_buffer[256];
int recv_len = serial_read(recv_buffer, sizeof(recv_buffer));

Packet received_pkt = get_received_packet(recv_buffer, recv_len, MY_CLIENT_ID);
if (received_pkt.is_valid) {
    // Process command and send response
    Packet response = init_packet();
    response.client_id = MY_CLIENT_ID;
    response.mode = SERIAL_MODE_CLIENT;
    response.status = ERR_SUCCESS;
    // ... populate response data
    
    char response_buffer[256];
    create_packet(&response, response_buffer);
    serial_write(response_buffer, response.data_len + ADDITIONAL_PACKET_LENGTH);
}
```

## Protocol Limitations

- **Maximum Data Length**: 251 bytes per packet
- **Client ID Range**: 1-255 (0x01-0xFF)
- **Command/Status Range**: 0-255 (0x00-0xFF)
- **Single-threaded**: Protocol assumes sequential request-response pattern

## Serial Configuration

- **Baud Rate**: Configurable (commonly 115200)
- **Data Bits**: 8
- **Parity**: None
- **Stop Bits**: 1
- **Flow Control**: None

## Multi-Architecture Support

The protocol is implemented in both C and C++ with the following compatibility:

- **C Implementation**: Core protocol functions in `.c` files
- **C++ Wrapper**: Object-oriented interface for C++ applications
- **Cross-platform**: Supports embedded systems, microcontrollers, and PC applications

## Security Considerations

- **CRC-8 Limitation**: Provides error detection but not cryptographic security
- **No Authentication**: Protocol does not include device authentication
- **Replay Protection**: Not implemented - applications should add if required
- **Data Integrity**: CRC-8 detects most transmission errors but not malicious tampering
