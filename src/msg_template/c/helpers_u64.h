static inline void h6xserial_write_u64_le(uint64_t value, uint8_t *out) {
    out[0] = (uint8_t)(value & 0xFFu);
    out[1] = (uint8_t)((value >> 8) & 0xFFu);
    out[2] = (uint8_t)((value >> 16) & 0xFFu);
    out[3] = (uint8_t)((value >> 24) & 0xFFu);
    out[4] = (uint8_t)((value >> 32) & 0xFFu);
    out[5] = (uint8_t)((value >> 40) & 0xFFu);
    out[6] = (uint8_t)((value >> 48) & 0xFFu);
    out[7] = (uint8_t)((value >> 56) & 0xFFu);
}

static inline uint64_t h6xserial_read_u64_le(const uint8_t *in) {
    return ((uint64_t)in[0]) |
           ((uint64_t)in[1] << 8) |
           ((uint64_t)in[2] << 16) |
           ((uint64_t)in[3] << 24) |
           ((uint64_t)in[4] << 32) |
           ((uint64_t)in[5] << 40) |
           ((uint64_t)in[6] << 48) |
           ((uint64_t)in[7] << 56);
}

static inline void h6xserial_write_u64_be(uint64_t value, uint8_t *out) {
    out[0] = (uint8_t)((value >> 56) & 0xFFu);
    out[1] = (uint8_t)((value >> 48) & 0xFFu);
    out[2] = (uint8_t)((value >> 40) & 0xFFu);
    out[3] = (uint8_t)((value >> 32) & 0xFFu);
    out[4] = (uint8_t)((value >> 24) & 0xFFu);
    out[5] = (uint8_t)((value >> 16) & 0xFFu);
    out[6] = (uint8_t)((value >> 8) & 0xFFu);
    out[7] = (uint8_t)(value & 0xFFu);
}

static inline uint64_t h6xserial_read_u64_be(const uint8_t *in) {
    return ((uint64_t)in[0] << 56) |
           ((uint64_t)in[1] << 48) |
           ((uint64_t)in[2] << 40) |
           ((uint64_t)in[3] << 32) |
           ((uint64_t)in[4] << 24) |
           ((uint64_t)in[5] << 16) |
           ((uint64_t)in[6] << 8) |
           ((uint64_t)in[7]);
}
