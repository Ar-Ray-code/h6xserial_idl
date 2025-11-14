static inline void h6xserial_write_f32_le(float value, uint8_t *out) {
    uint32_t u;
    memcpy(&u, &value, sizeof(uint32_t));
    h6xserial_write_u32_le(u, out);
}

static inline float h6xserial_read_f32_le(const uint8_t *in) {
    uint32_t u = h6xserial_read_u32_le(in);
    float f;
    memcpy(&f, &u, sizeof(float));
    return f;
}

static inline void h6xserial_write_f32_be(float value, uint8_t *out) {
    uint32_t u;
    memcpy(&u, &value, sizeof(uint32_t));
    h6xserial_write_u32_be(u, out);
}

static inline float h6xserial_read_f32_be(const uint8_t *in) {
    uint32_t u = h6xserial_read_u32_be(in);
    float f;
    memcpy(&f, &u, sizeof(float));
    return f;
}
