use super::Mac;
use crate::digest::*;
use std::borrow::Cow;
use std::cmp::Ordering;

const IPAD: u8 = 0x36;
const OPAD: u8 = 0x5c;

pub struct Hmac<H, const L: usize> {
    outer_hasher: H,
    inner_hasher: H,
}

impl<H: Digest<L>, const L: usize> Mac<L> for Hmac<H, L> {
    fn new(key: impl AsRef<[u8]>) -> Self {
        let k = key.as_ref();
        let k0: Cow<[u8]> = match k.len().cmp(&H::BLOCK_BYTES) {
            Ordering::Equal => k.into(),
            Ordering::Greater => {
                let mut hash_k = H::hash(k).to_vec();
                hash_k.resize(H::BLOCK_BYTES, 0);
                hash_k.into()
            }
            Ordering::Less => {
                let mut k = k.to_vec();
                k.resize(H::BLOCK_BYTES, 0);
                k.into()
            }
        };
        let mut outer_hasher = H::new();
        outer_hasher.update(k0.iter().map(|x| x ^ OPAD).collect::<Vec<_>>());
        let mut inner_hasher = H::new();
        inner_hasher.update(k0.iter().map(|x| x ^ IPAD).collect::<Vec<_>>());
        Self {
            outer_hasher,
            inner_hasher,
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        self.inner_hasher.update(data);
    }

    fn finalize(mut self) -> [u8; L] {
        let inner_hash = self.inner_hasher.finalize();
        self.outer_hasher.update(inner_hash);
        self.outer_hasher.finalize()
    }
}

pub type HmacSha256 = Hmac<Sha256, 32>;
pub type HmacSha3_224 = Hmac<Sha3_224, 28>;
pub type HmacSha3_256 = Hmac<Sha3_256, 32>;
pub type HmacSha3_384 = Hmac<Sha3_384, 48>;
pub type HmacSha3_512 = Hmac<Sha3_512, 64>;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::*;

    #[test]
    fn hmac_sha256_1() {
        let k = from_hex_without_spaces(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
        );
        let d = b"Sample message for keylen=blocklen";
        let m = HmacSha256::mac(k, d);
        assert_eq!(
            to_hex_without_spaces(m),
            "8bb9a1db9806f20df7f77b82138c7914d174d59e13dc4d0169c9057b133e1d62"
        );
    }

    #[test]
    fn hmac_sha256_2() {
        let k = from_hex_without_spaces(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
        );
        let d = b"Sample message for keylen<blocklen";
        let m = HmacSha256::mac(k, d);
        assert_eq!(
            to_hex_without_spaces(m),
            "a28cf43130ee696a98f14a37678b56bcfcbdd9e5cf69717fecf5480f0ebdf790"
        );
    }

    #[test]
    fn hmac_sha256_3() {
        let k = from_hex_without_spaces(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f60616263"
        );
        let d = b"Sample message for keylen=blocklen";
        let m = HmacSha256::mac(k, d);
        assert_eq!(
            to_hex_without_spaces(m),
            "bdccb6c72ddeadb500ae768386cb38cc41c63dbb0878ddb9c7a38a431b78378d"
        );
    }
}
