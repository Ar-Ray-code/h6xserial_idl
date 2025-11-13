static inline void seridl_write_f64_le(double value, uint8_t *out) {
    union {
        double f;
        uint64_t u;
    } conv;
    conv.f = value;
    seridl_write_u64_le(conv.u, out);
}

static inline double seridl_read_f64_le(const uint8_t *in) {
    union {
        double f;
        uint64_t u;
    } conv;
    conv.u = seridl_read_u64_le(in);
    return conv.f;
}

static inline void seridl_write_f64_be(double value, uint8_t *out) {
    union {
        double f;
        uint64_t u;
    } conv;
    conv.f = value;
    seridl_write_u64_be(conv.u, out);
}

static inline double seridl_read_f64_be(const uint8_t *in) {
    union {
        double f;
        uint64_t u;
    } conv;
    conv.u = seridl_read_u64_be(in);
    return conv.f;
}
