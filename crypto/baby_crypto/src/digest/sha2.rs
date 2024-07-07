use super::Digest;

pub struct Sha256 {
    h: [u32; Self::HASH_WORDS],
    m: [u8; Self::BLOCK_BYTES],
    len: u64,
}

impl Sha256 {
    const LEN_BYTES: usize = 8;
    const BLOCK_BITS: usize = 512;
    const BLOCK_BYTES: usize = Self::BLOCK_BITS / 8;
    const BLOCK_WORDS: usize = Self::BLOCK_BITS / 32;
    const HASH_WORDS: usize = 8;
    const ROUNDS: usize = 64;
    const H0: [u32; Self::HASH_WORDS] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
        0x5be0cd19,
    ];
    const K: [u32; Self::ROUNDS] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];

    fn ch(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (!x & z)
    }

    fn maj(x: u32, y: u32, z: u32) -> u32 {
        (x & y) ^ (x & z) ^ (y & z)
    }

    fn big_sigma_0(x: u32) -> u32 {
        x.rotate_right(2) ^ x.rotate_right(13) ^ x.rotate_right(22)
    }

    fn big_sigma_1(x: u32) -> u32 {
        x.rotate_right(6) ^ x.rotate_right(11) ^ x.rotate_right(25)
    }

    fn small_sigma_0(x: u32) -> u32 {
        x.rotate_right(7) ^ x.rotate_right(18) ^ (x >> 3)
    }

    fn small_sigma_1(x: u32) -> u32 {
        x.rotate_right(17) ^ x.rotate_right(19) ^ (x >> 10)
    }

    fn consume_block(&mut self) {
        Self::update_hash_with_block(&mut self.h, &self.m);
    }

    fn process_block(&mut self, m: &[u8]) {
        Self::update_hash_with_block(&mut self.h, m);
    }

    fn update_hash_with_block(hash: &mut [u32; Self::HASH_WORDS], m: &[u8]) {
        let mut w = [0u32; Self::BLOCK_WORDS];

        let mut a = hash[0];
        let mut b = hash[1];
        let mut c = hash[2];
        let mut d = hash[3];
        let mut e = hash[4];
        let mut f = hash[5];
        let mut g = hash[6];
        let mut h = hash[7];

        for i in 0..Self::ROUNDS {
            let wt = if i < Self::BLOCK_WORDS {
                ((m[i * 4] as u32) << 24)
                    | ((m[i * 4 + 1] as u32) << 16)
                    | ((m[i * 4 + 2] as u32) << 8)
                    | (m[i * 4 + 3] as u32)
            } else {
                Self::small_sigma_1(w[(i - 2) % Self::BLOCK_WORDS])
                    .wrapping_add(w[(i - 7) % Self::BLOCK_WORDS])
                    .wrapping_add(Self::small_sigma_0(w[(i - 15) % Self::BLOCK_WORDS]))
                    .wrapping_add(w[i % Self::BLOCK_WORDS])
            };
            w[i % Self::BLOCK_WORDS] = wt;
            let t1 = h
                .wrapping_add(Self::big_sigma_1(e))
                .wrapping_add(Self::ch(e, f, g))
                .wrapping_add(Self::K[i])
                .wrapping_add(wt);
            let t2 = Self::big_sigma_0(a).wrapping_add(Self::maj(a, b, c));
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }

        hash[0] = hash[0].wrapping_add(a);
        hash[1] = hash[1].wrapping_add(b);
        hash[2] = hash[2].wrapping_add(c);
        hash[3] = hash[3].wrapping_add(d);
        hash[4] = hash[4].wrapping_add(e);
        hash[5] = hash[5].wrapping_add(f);
        hash[6] = hash[6].wrapping_add(g);
        hash[7] = hash[7].wrapping_add(h);
    }
}

impl Digest<32> for Sha256 {
    const BLOCK_BYTES: usize = Self::BLOCK_BYTES;

    fn new() -> Self {
        Self {
            h: Self::H0,
            m: [0; Self::BLOCK_BYTES],
            len: 0,
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        let mut data = data.as_ref();

        let p = self.len as usize % Self::BLOCK_BYTES;
        self.len = match self.len.checked_add(data.len() as u64) {
            Some(len) => len,
            None => panic!("message too long"),
        };

        if p != 0 {
            let block_remain_bytes = Self::BLOCK_BYTES - p;
            if data.len() < block_remain_bytes {
                self.m[p..p + data.len()].copy_from_slice(data);
                return;
            }
            let (l, r) = data.split_at(block_remain_bytes);
            self.m[p..].copy_from_slice(l);
            self.consume_block();
            data = r;
        }

        while data.len() >= Self::BLOCK_BYTES {
            let (l, r) = data.split_at(Self::BLOCK_BYTES);
            self.process_block(l);
            data = r;
        }

        self.m[..data.len()].copy_from_slice(data);
    }

    fn finalize(mut self) -> [u8; Self::HASH_WORDS * 4] {
        let l = self.len * 8;
        let p = self.len as usize % Self::BLOCK_BYTES;
        self.m[p] = 1 << 7;
        if p + Self::LEN_BYTES < Self::BLOCK_BYTES {
            self.m[p + 1..Self::BLOCK_BYTES - Self::LEN_BYTES].fill(0);
        } else {
            self.m[p + 1..].fill(0);
            self.consume_block();
            self.m[..Self::BLOCK_BYTES - Self::LEN_BYTES].fill(0);
        }
        self.m[Self::BLOCK_BYTES - Self::LEN_BYTES..].copy_from_slice(&l.to_be_bytes());
        self.consume_block();

        let mut result = [0; Self::HASH_WORDS * 4];
        for (i, h) in self.h.iter().enumerate() {
            result[i * 4..i * 4 + 4].copy_from_slice(&h.to_be_bytes());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod sha256 {
        use super::*;
        use crate::hex::*;

        #[test]
        fn hash_empty() {
            assert_eq!(
                to_hex_without_spaces(Sha256::hash([])),
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
            );
        }

        #[test]
        fn hash_abc() {
            assert_eq!(
                to_hex_without_spaces(Sha256::hash(b"abc")),
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
            );
        }

        #[test]
        fn hash_440() {
            let mut hasher = Sha256::new();
            for _ in 0..55 {
                hasher.update([0]);
            }
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "02779466cdec163811d078815c633f21901413081449002f24aa3e80f0b88ef7"
            );
        }

        #[test]
        fn hash_448() {
            assert_eq!(
                to_hex_without_spaces(Sha256::hash([0].repeat(56))),
                "d4817aa5497628e7c77e6b606107042bbba3130888c5f47a375e6179be789fbb"
            );
        }

        #[test]
        fn hash_1600() {
            assert_eq!(
                to_hex_without_spaces(Sha256::hash(b"1234".repeat(50))),
                "e8c353210c16f16369403d2cf5ede4b9b430a88bfe6abd7d084137ee21c29120"
            );
        }

        #[test]
        fn hash_2000() {
            let mut hasher = Sha256::new();
            for _ in 0..50 {
                hasher.update(b"ouuan");
            }
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "e226eb4685fbf04653c8fa577fbd6a6626776959e33e6c4073106ba5dcd101dd"
            );
        }

        #[test]
        fn hash_2048() {
            let mut hasher = Sha256::new();
            for len in 62..=65 {
                hasher.update([1].repeat(len));
            }
            hasher.update("rs");
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "1354805aebd504cf44850ef5fe25de5d126e71d2ef7607bb2105e85e0010f0ca"
            );
        }
    }
}
