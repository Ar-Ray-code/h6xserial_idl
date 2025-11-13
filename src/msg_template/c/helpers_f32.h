static inline void seridl_write_f32_le(float value, uint8_t *out) {
    union {
        float f;
        uint32_t u;
    } conv;
    conv.f = value;
    seridl_write_u32_le(conv.u, out);
}

static inline float seridl_read_f32_le(const uint8_t *in) {
    union {
        float f;
        uint32_t u;
    } conv;
    conv.u = seridl_read_u32_le(in);
    return conv.f;
}

static inline void seridl_write_f32_be(float value, uint8_t *out) {
    union {
        float f;
        uint32_t u;
    } conv;
    conv.f = value;
    seridl_write_u32_be(conv.u, out);
}

static inline float seridl_read_f32_be(const uint8_t *in) {
    union {
        float f;
        uint32_t u;
    } conv;
    conv.u = seridl_read_u32_be(in);
    return conv.f;
}
