#![allow(clippy::missing_safety_doc)]

use baby_crypto::digest::{Digest, Sha256};
use baby_crypto::encrypt::{aes_128_gcm_decrypt, aes_128_gcm_encrypt};
use baby_crypto::kdf::hkdf_sha256;
use baby_crypto::mac::{HmacSha256, Mac};
use core::slice::from_raw_parts;

#[no_mangle]
pub unsafe extern "C" fn rust_sha256(src: *const u8, len: usize, dst: *mut u8) {
    let message = unsafe { from_raw_parts(src, len) };
    let result = Sha256::hash(message);
    unsafe { result.as_ptr().copy_to_nonoverlapping(dst, result.len()) }
}

#[no_mangle]
pub unsafe extern "C" fn rust_hmac_sha256(
    key_src: *const u8,
    key_len: usize,
    msg_src: *const u8,
    msg_len: usize,
    dst: *mut u8,
) {
    let key = unsafe { from_raw_parts(key_src, key_len) };
    let msg = unsafe { from_raw_parts(msg_src, msg_len) };
    let result = HmacSha256::mac(key, msg);
    unsafe { result.as_ptr().copy_to_nonoverlapping(dst, result.len()) }
}

#[no_mangle]
pub unsafe extern "C" fn rust_aes_128_gcm_encrypt(
    p_src: *const u8,
    p_len: usize,
    iv_src: *const u8,
    a_src: *const u8,
    a_len: usize,
    k_src: *const u8,
    c_dst: *mut u8,
    t_dst: *mut u8,
) -> bool {
    let p = unsafe { from_raw_parts(p_src, p_len) };
    let iv = unsafe { from_raw_parts(iv_src, 12) }.try_into().unwrap();
    let a = unsafe { from_raw_parts(a_src, a_len) };
    let k = unsafe { from_raw_parts(k_src, 16) }.try_into().unwrap();
    match aes_128_gcm_encrypt(k, iv, p, a) {
        Ok((c, t)) => {
            unsafe {
                c.as_ptr().copy_to_nonoverlapping(c_dst, p_len);
                t.as_ptr().copy_to_nonoverlapping(t_dst, 16);
            }
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_aes_128_gcm_decrypt(
    c_src: *const u8,
    c_len: usize,
    iv_src: *const u8,
    a_src: *const u8,
    a_len: usize,
    k_src: *const u8,
    p_dst: *mut u8,
    t_src: *const u8,
) -> bool {
    let c = unsafe { from_raw_parts(c_src, c_len) };
    let iv = unsafe { from_raw_parts(iv_src, 12) }.try_into().unwrap();
    let a = unsafe { from_raw_parts(a_src, a_len) };
    let k = unsafe { from_raw_parts(k_src, 16) }.try_into().unwrap();
    let t = unsafe { from_raw_parts(t_src, 16) }.try_into().unwrap();
    match aes_128_gcm_decrypt(k, iv, c, a, t) {
        Ok(p) => {
            unsafe { p.as_ptr().copy_to_nonoverlapping(p_dst, c_len) }
            true
        }
        Err(_) => false,
    }
}

#[no_mangle]
pub unsafe extern "C" fn rust_hkdf_sha256(
    salt_src: *const u8,
    salt_len: usize,
    ikm_src: *const u8,
    ikm_len: usize,
    info_src: *const u8,
    info_len: usize,
    okm_len: usize,
    okm_dst: *mut u8,
) {
    let salt = unsafe { from_raw_parts(salt_src, salt_len) };
    let ikm = unsafe { from_raw_parts(ikm_src, ikm_len) };
    let info = unsafe { from_raw_parts(info_src, info_len) };
    let key = hkdf_sha256(salt, ikm, info, okm_len);
    unsafe { key.as_ptr().copy_to_nonoverlapping(okm_dst, okm_len) }
}
