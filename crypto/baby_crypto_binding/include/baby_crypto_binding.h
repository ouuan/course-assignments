#pragma once

#include <cstddef>
#include <cstdint>

extern "C"
{
    void rust_sha256(const uint8_t *src, size_t len, uint8_t *dst);

    void rust_hmac_sha256(const uint8_t *key_src, size_t key_len, const uint8_t *msg_src,
                          size_t msg_len, uint8_t *dst);

    bool rust_aes_128_gcm_encrypt(const uint8_t *p_src, size_t p_len, const uint8_t *iv_src,
                                  const uint8_t *a_src, size_t a_len, const uint8_t *k_src,
                                  uint8_t *c_dst, uint8_t *t_dst);

    bool rust_aes_128_gcm_decrypt(const uint8_t *c_src, size_t c_len, const uint8_t *iv_src,
                                  const uint8_t *a_src, size_t a_len, const uint8_t *k_src,
                                  uint8_t *p_dst, const uint8_t *t_src);

    void rust_hkdf_sha256(const uint8_t *salt_src, size_t salt_len, const uint8_t *ikm_src,
                          size_t ikm_len, const uint8_t *info_src, size_t info_len, size_t okm_len,
                          uint8_t *okm_dst);
}
