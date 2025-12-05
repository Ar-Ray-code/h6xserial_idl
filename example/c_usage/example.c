/*
 * Example usage of h6xserial_idl generated message definitions
 *
 * This demonstrates:
 * - Encoding/decoding scalar messages
 * - Encoding/decoding array messages
 * - Encoding/decoding struct messages
 * - Server and Client role separation
 * - Bool type support
 * - Encode/decode round-trip verification
 *
 * This example includes both server and client headers to verify
 * encode/decode round-trip functionality. In real applications,
 * each side would only include their respective header.
 */

#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <math.h>

/* Include server header (has encode for pub, decode for sub) */
#include "sensor_messages_server.h"

/* Include client headers (have decode for pub, encode for sub) */
#include "sensor_messages_client_2.h"
#include "sensor_messages_client_3.h"
#include "sensor_messages_client_4.h"

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

/* Helper to compare floats */
static bool float_eq(float a, float b) {
    return fabsf(a - b) < 0.001f;
}

/*
 * Test 1: Ping Message (pub from server)
 * Server: encode
 * Client: decode
 * Round-trip: server encode -> client decode -> verify
 */
static void test_ping_message(void) {
    printf(COLOR_YELLOW "\n=== Test 1: Ping Message (scalar uint8, pub) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_ping_t original = {0};
    original.value = 42;

    /* Server encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_ping_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 1, "Ping encode returns correct length");
    print_hex("Encoded ping", buffer, encoded_len);

    /* Client decodes (using client_2 which has decode for pub messages) */
    h6xserial_msg_ping_t decoded = {0};
    bool decode_ok = h6xserial_msg_ping_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Ping decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.value == original.value, "Ping round-trip: value matches");
    printf("Original: %u, Decoded: %u\n", original.value, decoded.value);
}

/*
 * Test 2: Temperature Message (sub from client 2)
 * Client 2: encode
 * Server: decode
 * Round-trip: client encode -> server decode -> verify
 */
static void test_temperature_message(void) {
    printf(COLOR_YELLOW "\n=== Test 2: Temperature Message (scalar float32, sub from client 2) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_temperature_t original = {0};
    original.value = 23.5f;

    /* Client 2 encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_temperature_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 4, "Temperature encode returns 4 bytes");
    print_hex("Encoded temperature", buffer, encoded_len);

    /* Server decodes */
    h6xserial_msg_temperature_t decoded = {0};
    bool decode_ok = h6xserial_msg_temperature_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Temperature decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(float_eq(decoded.value, original.value), "Temperature round-trip: value matches");
    printf("Original: %.2f°C, Decoded: %.2f°C\n", original.value, decoded.value);
}

/*
 * Test 3: Firmware Version Message (sub from all clients)
 * Client: encode
 * Server: decode
 */
static void test_firmware_version_message(void) {
    printf(COLOR_YELLOW "\n=== Test 3: Firmware Version Message (char array, sub) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_firmware_version_t original = {0};
    const char* version_string = "v1.2.3-beta";
    original.length = strlen(version_string);
    memcpy(original.data, version_string, original.length);

    /* Client encodes (using client_2) */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_firmware_version_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == original.length, "Firmware version encode returns correct length");
    print_hex("Encoded firmware version", buffer, encoded_len);

    /* Server decodes */
    h6xserial_msg_firmware_version_t decoded = {0};
    bool decode_ok = h6xserial_msg_firmware_version_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Firmware version decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.length == original.length, "Firmware version round-trip: length matches");
    TEST_ASSERT(memcmp(decoded.data, original.data, original.length) == 0,
                "Firmware version round-trip: data matches");
    printf("Original: %.*s, Decoded: %.*s\n",
           (int)original.length, original.data, (int)decoded.length, decoded.data);
}

/*
 * Test 4: Multi-Temperature Message (sub from client 3)
 * Client 3: encode
 * Server: decode
 */
static void test_multi_temperature_message(void) {
    printf(COLOR_YELLOW "\n=== Test 4: Multi-Temperature Message (float32 array, sub from client 3) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_multi_temperature_t original = {0};
    original.length = 4;
    original.data[0] = 22.5f;
    original.data[1] = 23.0f;
    original.data[2] = 21.8f;
    original.data[3] = 24.2f;

    /* Client 3 encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_multi_temperature_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 16, "Multi-temperature encode returns 16 bytes");
    print_hex("Encoded multi-temperature", buffer, encoded_len);

    /* Server decodes */
    h6xserial_msg_multi_temperature_t decoded = {0};
    bool decode_ok = h6xserial_msg_multi_temperature_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Multi-temperature decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.length == original.length, "Multi-temperature round-trip: length matches");
    bool all_match = true;
    for (size_t i = 0; i < original.length; i++) {
        if (!float_eq(decoded.data[i], original.data[i])) {
            all_match = false;
            break;
        }
    }
    TEST_ASSERT(all_match, "Multi-temperature round-trip: all values match");

    printf("Original: ");
    for (size_t i = 0; i < original.length; i++) printf("%.1f ", original.data[i]);
    printf("\nDecoded:  ");
    for (size_t i = 0; i < decoded.length; i++) printf("%.1f ", decoded.data[i]);
    printf("\n");
}

/*
 * Test 5: Sensor Data Message (sub from client 2)
 * Client 2: encode
 * Server: decode
 * Complex struct with nested struct and array
 */
static void test_sensor_data_message(void) {
    printf(COLOR_YELLOW "\n=== Test 5: Sensor Data Message (struct with nested struct + array, sub from client 2) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_sensor_data_t original = {0};
    original.temperature = 25.3f;
    original.humidity = 65;
    original.pressure = 101325;
    original.co2_level = 450;

    /* Nested struct */
    original.room_b.temperatures_length = 3;
    original.room_b.temperatures[0] = 22.5f;
    original.room_b.temperatures[1] = 23.0f;
    original.room_b.temperatures[2] = 21.8f;
    original.room_b.humidity = 58;
    original.room_b.pressure = 101200;
    original.room_b.co2_level = 420;

    /* Client 2 encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_sensor_data_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 30, "Sensor data encode returns 30 bytes");
    print_hex("Encoded sensor data", buffer, encoded_len);

    /* Server decodes */
    h6xserial_msg_sensor_data_t decoded = {0};
    bool decode_ok = h6xserial_msg_sensor_data_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Sensor data decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(float_eq(decoded.temperature, original.temperature), "Sensor data round-trip: temperature matches");
    TEST_ASSERT(decoded.humidity == original.humidity, "Sensor data round-trip: humidity matches");
    TEST_ASSERT(decoded.pressure == original.pressure, "Sensor data round-trip: pressure matches");
    TEST_ASSERT(decoded.co2_level == original.co2_level, "Sensor data round-trip: co2_level matches");

    /* Verify nested struct */
    TEST_ASSERT(decoded.room_b.temperatures_length == original.room_b.temperatures_length,
                "Sensor data round-trip: nested temperatures_length matches");
    bool temps_match = true;
    for (size_t i = 0; i < original.room_b.temperatures_length; i++) {
        if (!float_eq(decoded.room_b.temperatures[i], original.room_b.temperatures[i])) {
            temps_match = false;
            break;
        }
    }
    TEST_ASSERT(temps_match, "Sensor data round-trip: nested temperatures match");
    TEST_ASSERT(decoded.room_b.humidity == original.room_b.humidity,
                "Sensor data round-trip: nested humidity matches");
    TEST_ASSERT(decoded.room_b.pressure == original.room_b.pressure,
                "Sensor data round-trip: nested pressure matches");
    TEST_ASSERT(decoded.room_b.co2_level == original.room_b.co2_level,
                "Sensor data round-trip: nested co2_level matches");

    printf("Room A: temp=%.1f, humidity=%u, pressure=%u, co2=%u\n",
           decoded.temperature, decoded.humidity, decoded.pressure, decoded.co2_level);
    printf("Room B: temps=[");
    for (size_t i = 0; i < decoded.room_b.temperatures_length; i++) {
        printf("%.1f%s", decoded.room_b.temperatures[i],
               i < decoded.room_b.temperatures_length - 1 ? ", " : "");
    }
    printf("], humidity=%u, pressure=%u, co2=%u\n",
           decoded.room_b.humidity, decoded.room_b.pressure, decoded.room_b.co2_level);
}

/*
 * Test 6: LED Control Message (pub from server)
 * Server: encode
 * Client: decode
 * Tests bool type support
 */
static void test_led_control_message(void) {
    printf(COLOR_YELLOW "\n=== Test 6: LED Control Message (struct with bool, pub) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_led_control_t original = {0};
    original.led_id = 1;
    original.red = true;
    original.green = false;
    original.blue = true;
    original.brightness = 200;

    /* Server encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_led_control_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 5, "LED control encode returns 5 bytes");
    print_hex("Encoded LED control", buffer, encoded_len);

    /* Client decodes (using client_2 which has decode for pub messages) */
    h6xserial_msg_led_control_t decoded = {0};
    bool decode_ok = h6xserial_msg_led_control_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "LED control decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.led_id == original.led_id, "LED control round-trip: led_id matches");
    TEST_ASSERT(decoded.red == original.red, "LED control round-trip: red matches");
    TEST_ASSERT(decoded.green == original.green, "LED control round-trip: green matches");
    TEST_ASSERT(decoded.blue == original.blue, "LED control round-trip: blue matches");
    TEST_ASSERT(decoded.brightness == original.brightness, "LED control round-trip: brightness matches");

    printf("Original: ID=%u R=%s G=%s B=%s brightness=%u\n",
           original.led_id, original.red ? "ON" : "OFF", original.green ? "ON" : "OFF",
           original.blue ? "ON" : "OFF", original.brightness);
    printf("Decoded:  ID=%u R=%s G=%s B=%s brightness=%u\n",
           decoded.led_id, decoded.red ? "ON" : "OFF", decoded.green ? "ON" : "OFF",
           decoded.blue ? "ON" : "OFF", decoded.brightness);
}

/*
 * Test 7: Motor Speeds Message (pub from server to client 3)
 * Server: encode
 * Client 3: decode
 */
static void test_motor_speeds_message(void) {
    printf(COLOR_YELLOW "\n=== Test 7: Motor Speeds Message (int16 array, pub to client 3) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_motor_speeds_t original = {0};
    original.length = 4;
    original.data[0] = 1000;
    original.data[1] = -500;
    original.data[2] = 750;
    original.data[3] = 0;

    /* Server encodes */
    uint8_t buffer[256];
    size_t encoded_len = h6xserial_msg_motor_speeds_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 8, "Motor speeds encode returns 8 bytes");
    print_hex("Encoded motor speeds", buffer, encoded_len);

    /* Client 3 decodes */
    h6xserial_msg_motor_speeds_t decoded = {0};
    bool decode_ok = h6xserial_msg_motor_speeds_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Motor speeds decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.length == original.length, "Motor speeds round-trip: length matches");
    bool all_match = true;
    for (size_t i = 0; i < original.length; i++) {
        if (decoded.data[i] != original.data[i]) {
            all_match = false;
            break;
        }
    }
    TEST_ASSERT(all_match, "Motor speeds round-trip: all values match");

    printf("Original: ");
    for (size_t i = 0; i < original.length; i++) printf("%d ", original.data[i]);
    printf("\nDecoded:  ");
    for (size_t i = 0; i < decoded.length; i++) printf("%d ", decoded.data[i]);
    printf("\n");
}

/*
 * Test 8: Large Data Message (pub from server to client 4)
 * Server: encode
 * Client 4: decode
 */
static void test_large_data_message(void) {
    printf(COLOR_YELLOW "\n=== Test 8: Large Data Message (struct with 108 uint16 array, pub to client 4) ===" COLOR_RESET "\n");

    /* Original data */
    h6xserial_msg_large_data_t original = {0};
    original.segment = 2;
    original.data_length = 108;
    for (size_t i = 0; i < 108; i++) {
        original.data[i] = (uint16_t)(1000 + i * 10);
    }

    /* Server encodes */
    uint8_t buffer[512];
    size_t encoded_len = h6xserial_msg_large_data_encode(&original, buffer, sizeof(buffer));
    TEST_ASSERT(encoded_len == 217, "Large data encode returns 217 bytes");
    print_hex("Encoded large data (first 20 bytes)", buffer, 20);

    /* Client 4 decodes */
    h6xserial_msg_large_data_t decoded = {0};
    bool decode_ok = h6xserial_msg_large_data_decode(&decoded, buffer, encoded_len);
    TEST_ASSERT(decode_ok, "Large data decode succeeds");

    /* Verify round-trip */
    TEST_ASSERT(decoded.segment == original.segment, "Large data round-trip: segment matches");
    TEST_ASSERT(decoded.data_length == original.data_length, "Large data round-trip: data_length matches");
    bool all_match = true;
    for (size_t i = 0; i < original.data_length; i++) {
        if (decoded.data[i] != original.data[i]) {
            all_match = false;
            printf("Mismatch at index %zu: expected %u, got %u\n",
                   i, original.data[i], decoded.data[i]);
            break;
        }
    }
    TEST_ASSERT(all_match, "Large data round-trip: all 108 values match");

    printf("Segment: %u, Data: [%u, %u, ... , %u] (%zu values)\n",
           decoded.segment, decoded.data[0], decoded.data[1],
           decoded.data[decoded.data_length - 1], decoded.data_length);
}

/*
 * Test 9: Error Conditions
 */
static void test_error_conditions(void) {
    printf(COLOR_YELLOW "\n=== Test 9: Error Conditions ===" COLOR_RESET "\n");

    uint8_t buffer[256];
    h6xserial_msg_ping_t ping = {0};

    /* Test NULL pointer handling for encode */
    size_t len = h6xserial_msg_ping_encode(NULL, buffer, sizeof(buffer));
    TEST_ASSERT(len == 0, "Encode with NULL message returns 0");

    len = h6xserial_msg_ping_encode(&ping, NULL, sizeof(buffer));
    TEST_ASSERT(len == 0, "Encode with NULL buffer returns 0");

    /* Test buffer too small for encode */
    len = h6xserial_msg_ping_encode(&ping, buffer, 0);
    TEST_ASSERT(len == 0, "Encode with zero-size buffer returns 0");

    /* Test NULL pointer handling for decode */
    bool ok = h6xserial_msg_ping_decode(NULL, buffer, 1);
    TEST_ASSERT(!ok, "Decode with NULL message returns false");

    ok = h6xserial_msg_ping_decode(&ping, NULL, 1);
    TEST_ASSERT(!ok, "Decode with NULL buffer returns false");

    /* Test decode with wrong size */
    ok = h6xserial_msg_ping_decode(&ping, buffer, 0);
    TEST_ASSERT(!ok, "Decode with zero size returns false");

    ok = h6xserial_msg_ping_decode(&ping, buffer, 2);
    TEST_ASSERT(!ok, "Decode with wrong size returns false");
}

/*
 * Test 10: Packet ID Definitions
 */
static void test_packet_ids(void) {
    printf(COLOR_YELLOW "\n=== Test 10: Packet ID Definitions ===" COLOR_RESET "\n");

    TEST_ASSERT(H6XSERIAL_MSG_PING_PACKET_ID == 0, "Ping packet ID is 0");
    TEST_ASSERT(H6XSERIAL_MSG_FIRMWARE_VERSION_PACKET_ID == 4, "Firmware version packet ID is 4");
    TEST_ASSERT(H6XSERIAL_MSG_DEVICE_NAME_PACKET_ID == 14, "Device name packet ID is 14");
    TEST_ASSERT(H6XSERIAL_MSG_TEMPERATURE_PACKET_ID == 20, "Temperature packet ID is 20");
    TEST_ASSERT(H6XSERIAL_MSG_MULTI_TEMPERATURE_PACKET_ID == 21, "Multi-temperature packet ID is 21");
    TEST_ASSERT(H6XSERIAL_MSG_HUMIDITY_PACKET_ID == 22, "Humidity packet ID is 22");
    TEST_ASSERT(H6XSERIAL_MSG_SENSOR_DATA_PACKET_ID == 30, "Sensor data packet ID is 30");
    TEST_ASSERT(H6XSERIAL_MSG_LED_CONTROL_PACKET_ID == 40, "LED control packet ID is 40");
    TEST_ASSERT(H6XSERIAL_MSG_MOTOR_SPEEDS_PACKET_ID == 50, "Motor speeds packet ID is 50");
    TEST_ASSERT(H6XSERIAL_MSG_LARGE_DATA_PACKET_ID == 60, "Large data packet ID is 60");

    /* Verify max length macros */
    TEST_ASSERT(H6XSERIAL_MSG_LARGE_DATA_DATA_MAX_LENGTH == 108,
                "Large data.data max length is 108");

    printf("All packet IDs and max lengths verified\n");
}

int main(void) {
    printf(COLOR_BLUE "\n========================================\n");
    printf("  h6xserial_idl Server/Client Example\n");
    printf("  (Round-trip encode/decode verification)\n");
    printf("========================================\n" COLOR_RESET);
    printf("\nThis example tests encode/decode round-trip:\n");
    printf("- For 'pub' messages: server encode -> client decode\n");
    printf("- For 'sub' messages: client encode -> server decode\n\n");

    test_ping_message();
    test_temperature_message();
    test_firmware_version_message();
    test_multi_temperature_message();
    test_sensor_data_message();
    test_led_control_message();
    test_motor_speeds_message();
    test_large_data_message();
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
