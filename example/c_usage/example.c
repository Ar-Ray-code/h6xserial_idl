/*
 * Example usage of h6xserial_idl generated message definitions
 *
 * This demonstrates:
 * - Encoding/decoding scalar messages
 * - Encoding/decoding array messages
 * - Encoding/decoding struct messages
 * - Validating encoded/decoded data
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include "sensor_messages.h"

/* ANSI color codes for terminal output */
#define COLOR_RESET   "\033[0m"
#define COLOR_GREEN   "\033[32m"
#define COLOR_BLUE    "\033[34m"
#define COLOR_YELLOW  "\033[33m"

/* Test result tracking */
static int tests_passed = 0;
static int tests_failed = 0;

#define TEST_ASSERT(condition, message) \
    do { \
        if (condition) { \
            printf(COLOR_GREEN "[PASS]" COLOR_RESET " %s\n", message); \
            tests_passed++; \
        } else { \
            printf("\033[31m[FAIL]\033[0m %s\n", message); \
            tests_failed++; \
        } \
    } while(0)

/* Helper function to print buffer in hex */
static void print_hex(const char* label, const uint8_t* data, size_t len) {
    printf(COLOR_BLUE "%s: " COLOR_RESET, label);
    for (size_t i = 0; i < len; i++) {
        printf("%02X ", data[i]);
    }
    printf("\n");
}

/* Test 1: Scalar message (uint8) */
static void test_ping_message(void) {
    printf(COLOR_YELLOW "\n=== Test 1: Ping Message (scalar uint8) ===" COLOR_RESET "\n");

    seridl_msg_ping_t ping_msg = {0};
    ping_msg.value = 42;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_ping_encode(&ping_msg, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 1, "Ping encode returns correct length");
    TEST_ASSERT(buffer[0] == 42, "Ping encoded value is correct");
    print_hex("Encoded ping", buffer, encoded_len);

    seridl_msg_ping_t decoded_ping = {0};
    bool decode_ok = seridl_msg_ping_decode(&decoded_ping, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Ping decode succeeds");
    TEST_ASSERT(decoded_ping.value == 42, "Ping decoded value matches");
}

/* Test 2: Scalar message (float32 with big endian) */
static void test_temperature_message(void) {
    printf(COLOR_YELLOW "\n=== Test 2: Temperature Message (scalar float32, big-endian) ===" COLOR_RESET "\n");

    seridl_msg_temperature_t temp_msg = {0};
    temp_msg.value = 23.5f;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_temperature_encode(&temp_msg, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 4, "Temperature encode returns 4 bytes");
    print_hex("Encoded temperature", buffer, encoded_len);

    seridl_msg_temperature_t decoded_temp = {0};
    bool decode_ok = seridl_msg_temperature_decode(&decoded_temp, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Temperature decode succeeds");
    TEST_ASSERT(decoded_temp.value == 23.5f, "Temperature decoded value matches");
    printf("Temperature: %.2f°C\n", decoded_temp.value);
}

/* Test 3: String array message (char array) */
static void test_firmware_version_message(void) {
    printf(COLOR_YELLOW "\n=== Test 3: Firmware Version Message (char array) ===" COLOR_RESET "\n");

    seridl_msg_firmware_version_t fw_msg = {0};
    const char* version_string = "v1.2.3-beta";
    fw_msg.length = strlen(version_string);
    memcpy(fw_msg.data, version_string, fw_msg.length);

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_firmware_version_encode(&fw_msg, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == fw_msg.length, "Firmware version encode returns correct length");
    print_hex("Encoded firmware version", buffer, encoded_len);

    seridl_msg_firmware_version_t decoded_fw = {0};
    bool decode_ok = seridl_msg_firmware_version_decode(&decoded_fw, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Firmware version decode succeeds");
    TEST_ASSERT(decoded_fw.length == fw_msg.length, "Firmware version length matches");
    TEST_ASSERT(memcmp(decoded_fw.data, version_string, fw_msg.length) == 0,
                "Firmware version data matches");
    printf("Firmware version: %.*s\n", (int)decoded_fw.length, decoded_fw.data);
}

/* Test 4: Array message (float32 array with big endian) */
static void test_multi_temperature_message(void) {
    printf(COLOR_YELLOW "\n=== Test 4: Multi-Temperature Message (float32 array, big-endian) ===" COLOR_RESET "\n");

    seridl_msg_multi_temperature_t multi_temp = {0};
    multi_temp.length = 4;
    multi_temp.data[0] = 22.5f;
    multi_temp.data[1] = 23.0f;
    multi_temp.data[2] = 21.8f;
    multi_temp.data[3] = 24.2f;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_multi_temperature_encode(&multi_temp, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 16, "Multi-temperature encode returns 16 bytes (4 floats)");
    print_hex("Encoded multi-temperature", buffer, encoded_len);

    seridl_msg_multi_temperature_t decoded_multi = {0};
    bool decode_ok = seridl_msg_multi_temperature_decode(&decoded_multi, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Multi-temperature decode succeeds");
    TEST_ASSERT(decoded_multi.length == 4, "Multi-temperature length matches");

    printf("Temperature readings: ");
    for (size_t i = 0; i < decoded_multi.length; i++) {
        printf("%.1f°C ", decoded_multi.data[i]);
        TEST_ASSERT(decoded_multi.data[i] == multi_temp.data[i],
                    "Multi-temperature value matches");
    }
    printf("\n");
}

/* Test 5: Struct message with mixed types and endianness */
static void test_sensor_data_message(void) {
    printf(COLOR_YELLOW "\n=== Test 5: Sensor Data Message (struct) ===" COLOR_RESET "\n");

    seridl_msg_sensor_data_t sensor = {0};
    sensor.temperature = 25.3f;
    sensor.humidity = 65;
    sensor.pressure = 101325;
    sensor.co2_level = 450;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_sensor_data_encode(&sensor, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 11, "Sensor data encode returns 11 bytes");
    print_hex("Encoded sensor data", buffer, encoded_len);

    seridl_msg_sensor_data_t decoded_sensor = {0};
    bool decode_ok = seridl_msg_sensor_data_decode(&decoded_sensor, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Sensor data decode succeeds");
    TEST_ASSERT(decoded_sensor.temperature == sensor.temperature, "Temperature matches");
    TEST_ASSERT(decoded_sensor.humidity == sensor.humidity, "Humidity matches");
    TEST_ASSERT(decoded_sensor.pressure == sensor.pressure, "Pressure matches");
    TEST_ASSERT(decoded_sensor.co2_level == sensor.co2_level, "CO2 level matches");

    printf("Sensor readings:\n");
    printf("  Temperature: %.1f°C\n", decoded_sensor.temperature);
    printf("  Humidity: %u%%\n", decoded_sensor.humidity);
    printf("  Pressure: %u Pa\n", decoded_sensor.pressure);
    printf("  CO2: %u ppm\n", decoded_sensor.co2_level);
}

/* Test 6: LED control struct message */
static void test_led_control_message(void) {
    printf(COLOR_YELLOW "\n=== Test 6: LED Control Message (struct) ===" COLOR_RESET "\n");

    seridl_msg_led_control_t led = {0};
    led.led_id = 1;
    led.red = 255;
    led.green = 128;
    led.blue = 64;
    led.brightness = 200;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_led_control_encode(&led, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 5, "LED control encode returns 5 bytes");
    print_hex("Encoded LED control", buffer, encoded_len);

    seridl_msg_led_control_t decoded_led = {0};
    bool decode_ok = seridl_msg_led_control_decode(&decoded_led, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "LED control decode succeeds");
    TEST_ASSERT(decoded_led.led_id == led.led_id, "LED ID matches");
    TEST_ASSERT(decoded_led.red == led.red, "Red value matches");
    TEST_ASSERT(decoded_led.green == led.green, "Green value matches");
    TEST_ASSERT(decoded_led.blue == led.blue, "Blue value matches");
    TEST_ASSERT(decoded_led.brightness == led.brightness, "Brightness matches");

    printf("LED control: ID=%u RGB(%u,%u,%u) Brightness=%u\n",
           decoded_led.led_id, decoded_led.red, decoded_led.green,
           decoded_led.blue, decoded_led.brightness);
}

/* Test 7: Motor speeds array (int16 with little endian) */
static void test_motor_speeds_message(void) {
    printf(COLOR_YELLOW "\n=== Test 7: Motor Speeds Message (int16 array, little-endian) ===" COLOR_RESET "\n");

    seridl_msg_motor_speeds_t motors = {0};
    motors.length = 4;
    motors.data[0] = 1000;
    motors.data[1] = -500;
    motors.data[2] = 750;
    motors.data[3] = 0;

    uint8_t buffer[256];
    size_t encoded_len = seridl_msg_motor_speeds_encode(&motors, buffer, sizeof(buffer));

    TEST_ASSERT(encoded_len == 8, "Motor speeds encode returns 8 bytes (4 int16)");
    print_hex("Encoded motor speeds", buffer, encoded_len);

    seridl_msg_motor_speeds_t decoded_motors = {0};
    bool decode_ok = seridl_msg_motor_speeds_decode(&decoded_motors, buffer, encoded_len);

    TEST_ASSERT(decode_ok, "Motor speeds decode succeeds");
    TEST_ASSERT(decoded_motors.length == 4, "Motor speeds length matches");

    printf("Motor speeds: ");
    for (size_t i = 0; i < decoded_motors.length; i++) {
        printf("%d ", decoded_motors.data[i]);
        TEST_ASSERT(decoded_motors.data[i] == motors.data[i], "Motor speed value matches");
    }
    printf("\n");
}

/* Test 8: Error conditions */
static void test_error_conditions(void) {
    printf(COLOR_YELLOW "\n=== Test 8: Error Conditions ===" COLOR_RESET "\n");

    uint8_t buffer[256];
    seridl_msg_ping_t ping = {0};

    /* Test NULL pointer handling */
    size_t len = seridl_msg_ping_encode(NULL, buffer, sizeof(buffer));
    TEST_ASSERT(len == 0, "Encode with NULL message returns 0");

    len = seridl_msg_ping_encode(&ping, NULL, sizeof(buffer));
    TEST_ASSERT(len == 0, "Encode with NULL buffer returns 0");

    bool ok = seridl_msg_ping_decode(NULL, buffer, 1);
    TEST_ASSERT(!ok, "Decode with NULL message returns false");

    ok = seridl_msg_ping_decode(&ping, NULL, 1);
    TEST_ASSERT(!ok, "Decode with NULL buffer returns false");

    /* Test buffer too small */
    len = seridl_msg_ping_encode(&ping, buffer, 0);
    TEST_ASSERT(len == 0, "Encode with zero-size buffer returns 0");

    /* Test decode with wrong size */
    buffer[0] = 42;
    ok = seridl_msg_ping_decode(&ping, buffer, 0);
    TEST_ASSERT(!ok, "Decode with zero size returns false");

    ok = seridl_msg_ping_decode(&ping, buffer, 2);
    TEST_ASSERT(!ok, "Decode with wrong size returns false");
}

/* Test 9: Packet ID definitions */
static void test_packet_ids(void) {
    printf(COLOR_YELLOW "\n=== Test 9: Packet ID Definitions ===" COLOR_RESET "\n");

    TEST_ASSERT(SERIDL_MSG_PING_PACKET_ID == 0, "Ping packet ID is 0");
    TEST_ASSERT(SERIDL_MSG_FIRMWARE_VERSION_PACKET_ID == 4, "Firmware version packet ID is 4");
    TEST_ASSERT(SERIDL_MSG_DEVICE_NAME_PACKET_ID == 14, "Device name packet ID is 14");
    TEST_ASSERT(SERIDL_MSG_TEMPERATURE_PACKET_ID == 20, "Temperature packet ID is 20");
    TEST_ASSERT(SERIDL_MSG_MULTI_TEMPERATURE_PACKET_ID == 21, "Multi-temperature packet ID is 21");
    TEST_ASSERT(SERIDL_MSG_HUMIDITY_PACKET_ID == 22, "Humidity packet ID is 22");
    TEST_ASSERT(SERIDL_MSG_SENSOR_DATA_PACKET_ID == 30, "Sensor data packet ID is 30");
    TEST_ASSERT(SERIDL_MSG_LED_CONTROL_PACKET_ID == 40, "LED control packet ID is 40");
    TEST_ASSERT(SERIDL_MSG_MOTOR_SPEEDS_PACKET_ID == 50, "Motor speeds packet ID is 50");

    printf("All packet IDs verified\n");
}

int main(void) {
    printf(COLOR_BLUE "\n========================================\n");
    printf("  h6xserial_idl C Usage Example\n");
    printf("========================================\n" COLOR_RESET);

    test_ping_message();
    test_temperature_message();
    test_firmware_version_message();
    test_multi_temperature_message();
    test_sensor_data_message();
    test_led_control_message();
    test_motor_speeds_message();
    test_error_conditions();
    test_packet_ids();

    printf(COLOR_BLUE "\n========================================\n" COLOR_RESET);
    printf(COLOR_GREEN "Tests passed: %d\n" COLOR_RESET, tests_passed);
    if (tests_failed > 0) {
        printf("\033[31mTests failed: %d\033[0m\n", tests_failed);
    }
    printf(COLOR_BLUE "========================================\n" COLOR_RESET);

    return tests_failed > 0 ? 1 : 0;
}
