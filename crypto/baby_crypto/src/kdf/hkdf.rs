use crate::digest::*;
use crate::mac::{Hmac, Mac};

fn hkdf_extract<H: Digest<L>, const L: usize>(
    salt: impl AsRef<[u8]>,
    ikm: impl AsRef<[u8]>,
) -> [u8; L] {
    Hmac::<H, L>::mac(salt, ikm)
}

fn hkdf_expand<H: Digest<L>, const L: usize>(
    prk: &[u8; L],
    info: impl AsRef<[u8]>,
    okm_len: usize,
) -> Vec<u8> {
    let n = okm_len.div_ceil(L);
    let mut t = Vec::with_capacity(n * L);
    let mut tp = None;
    for i in 1..=n {
        let mut h = Hmac::<H, L>::new(prk);
        if let Some(tp) = tp {
            h.update(tp);
        }
        h.update(info.as_ref());
        h.update([i as u8]);
        let ti = h.finalize();
        t.extend_from_slice(&ti);
        tp = Some(ti);
    }
    t.truncate(okm_len);
    t
}

pub fn hkdf<H: Digest<L>, const L: usize>(
    salt: impl AsRef<[u8]>,
    ikm: impl AsRef<[u8]>,
    info: impl AsRef<[u8]>,
    okm_len: usize,
) -> Vec<u8> {
    let prk = hkdf_extract::<H, L>(salt, ikm);
    hkdf_expand::<H, L>(&prk, info, okm_len)
}

pub fn hkdf_sha256(
    salt: impl AsRef<[u8]>,
    ikm: impl AsRef<[u8]>,
    info: impl AsRef<[u8]>,
    okm_len: usize,
) -> Vec<u8> {
    hkdf::<Sha256, 32>(salt, ikm, info, okm_len)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::*;

    #[test]
    fn hkdf_sha256_test1() {
        let salt = from_hex_without_spaces("000102030405060708090a0b0c");
        let ikm = from_hex_without_spaces("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b");
        let info = from_hex_without_spaces("f0f1f2f3f4f5f6f7f8f9");
        let key_len = 42;
        let key = hkdf_sha256(salt, ikm, info, key_len);
        assert_eq!(
            to_hex_without_spaces(key),
            "3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"
        )
    }

    #[test]
    fn hkdf_sha256_test2() {
        let salt = from_hex_without_spaces("606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9fa0a1a2a3a4a5a6a7a8a9aaabacadaeaf");
        let ikm = from_hex_without_spaces("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f");
        let info = from_hex_without_spaces("b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff");
        let key_len = 82;
        let key = hkdf_sha256(salt, ikm, info, key_len);
        assert_eq!(
            to_hex_without_spaces(key),
            "b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71cc30c58179ec3e87c14c01d5c1f3434f1d87"
        )
    }
}
