# Command Definitions

Auto-generated from: `msgs/intermediate_msg.json`
Protocol version: 1.0.0
Max address: 255

## Base Commands (0~19)

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

## Custom Commands (20+)

| Command | Value | Description |
|---------|-------|-------------|
| `CMD_CUSTOM_TEMPERATURE_READING` | 20 | Read temperature sensor value |
| `CMD_CUSTOM_SENSOR_DATA` | 21 | Multi-sensor data readings |

