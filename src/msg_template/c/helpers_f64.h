static inline void h6xserial_write_f64_le(double value, uint8_t *out) {
    uint64_t u;
    memcpy(&u, &value, sizeof(uint64_t));
    h6xserial_write_u64_le(u, out);
}

static inline double h6xserial_read_f64_le(const uint8_t *in) {
    uint64_t u = h6xserial_read_u64_le(in);
    double f;
    memcpy(&f, &u, sizeof(double));
    return f;
}

static inline void h6xserial_write_f64_be(double value, uint8_t *out) {
    uint64_t u;
    memcpy(&u, &value, sizeof(uint64_t));
    h6xserial_write_u64_be(u, out);
}

static inline double h6xserial_read_f64_be(const uint8_t *in) {
    uint64_t u = h6xserial_read_u64_be(in);
    double f;
    memcpy(&f, &u, sizeof(double));
    return f;
}
