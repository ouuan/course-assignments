use super::BlockEncrypt;

const NB: usize = 4;
const B: usize = NB * 4;

const fn xtime(x: u8) -> u8 {
    (x << 1) ^ ((x >> 7) * 0x1b)
}

const fn gfmul(x: u8, mut y: u8) -> u8 {
    let mut result = 0;
    let mut t = x;
    while y > 0 {
        if y & 1 == 1 {
            result ^= t;
        }
        t = xtime(t);
        y >>= 1;
    }
    result
}

const fn sbox() -> [u8; 1 << 8] {
    let mut s = [0; 1 << 8];
    let mut i = 1;
    loop {
        if s[i as usize] == 0 {
            let mut j = 1;
            loop {
                if gfmul(i, j) == 1 {
                    s[i as usize] = j;
                    s[j as usize] = i;
                    break;
                }
                j += 1;
            }
        }
        if i == u8::MAX {
            break;
        }
        i += 1;
    }

    i = 0;
    loop {
        let x = s[i as usize];
        s[i as usize] ^=
            0x63 ^ x.rotate_right(4) ^ x.rotate_right(5) ^ x.rotate_right(6) ^ x.rotate_right(7);
        if i == 0xff {
            break;
        }
        i += 1;
    }

    s
}

const S: [u8; 1 << 8] = sbox();

const fn tbox(mix: [u8; 4]) -> [u32; 1 << 8] {
    let mut t = [0; 1 << 8];
    let mut i = 0;
    while i < (1 << 8) {
        let s = S[i];
        t[i] = u32::from_ne_bytes([
            gfmul(s, mix[0]),
            gfmul(s, mix[1]),
            gfmul(s, mix[2]),
            gfmul(s, mix[3]),
        ]);
        i += 1;
    }
    t
}

const T: [[u32; 1 << 8]; 4] = [
    tbox([2, 1, 1, 3]),
    tbox([3, 2, 1, 1]),
    tbox([1, 3, 2, 1]),
    tbox([1, 1, 3, 2]),
];

struct Aes<const K: usize, const R: usize> {
    w: Vec<u32>,
}

impl<const K: usize, const R: usize> BlockEncrypt<B, K> for Aes<K, R> {
    fn new(key: &[u8; K]) -> Self {
        let mut w = Vec::with_capacity(NB * (R + 1));

        for chunk in key.chunks_exact(4) {
            w.push(u32::from_ne_bytes(chunk.try_into().unwrap()));
        }

        let mut rcon = 1;

        for i in K / 4..NB * (R + 1) {
            let mut temp = w[i - 1];
            if i % (K / 4) == 0 {
                let a = temp.to_ne_bytes();
                let b = [
                    S[a[1] as usize] ^ rcon,
                    S[a[2] as usize],
                    S[a[3] as usize],
                    S[a[0] as usize],
                ];
                rcon = xtime(rcon);
                temp = u32::from_ne_bytes(b);
            } else if K / 4 > 6 && i % (K / 4) == 4 {
                temp = u32::from_ne_bytes(temp.to_ne_bytes().map(|x| S[x as usize]));
            }
            w.push(w[i - K / 4] ^ temp);
        }

        Self { w }
    }

    fn encrypt(&self, data: &[u8; B]) -> [u8; B] {
        let mut state = [0; NB];
        for i in 0..NB {
            state[i] = u32::from_ne_bytes(data[i * 4..i * 4 + 4].try_into().unwrap()) ^ self.w[i];
        }

        for r in 1..R {
            let mut s = [0; NB];
            s.copy_from_slice(&self.w[r * NB..r * NB + 4]);
            for i in 0..NB {
                for j in 0..4 {
                    s[(i + NB - j) % NB] ^= T[j][(state[i] as usize >> (8 * j)) & 0xff];
                }
            }
            state = s;
        }

        let s = state.map(|x| x.to_ne_bytes());
        for i in 0..NB {
            state[i] = u32::from_ne_bytes([
                S[s[i][0] as usize],
                S[s[(i + 1) % NB][1] as usize],
                S[s[(i + 2) % NB][2] as usize],
                S[s[(i + 3) % NB][3] as usize],
            ]) ^ self.w[R * NB + i];
        }

        let mut result = [0; B];
        for i in 0..NB {
            result[i * 4..i * 4 + 4].copy_from_slice(&state[i].to_ne_bytes());
        }
        result
    }
}

macro_rules! aes_variant {
    ($name:ident, $k:expr, $r:expr) => {
        pub struct $name(Aes<$k, $r>);

        impl BlockEncrypt<B, $k> for $name {
            fn new(key: &[u8; $k]) -> Self {
                Self(BlockEncrypt::new(key))
            }

            fn encrypt(&self, data: &[u8; B]) -> [u8; B] {
                self.0.encrypt(data)
            }
        }
    };
}

aes_variant!(Aes128, 16, 10);
aes_variant!(Aes192, 24, 12);
aes_variant!(Aes256, 32, 14);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::*;

    fn data<const SIZE: usize>(s: &str) -> [u8; SIZE] {
        from_hex_without_spaces(s).try_into().unwrap()
    }

    #[test]
    fn aes128_key() {
        let cipher = Aes128::new(&data("2b7e151628aed2a6abf7158809cf4f3c"));
        assert_eq!(cipher.0.w[0], 0x16157e2b);
        assert_eq!(cipher.0.w[4], 0x17fefaa0);
        assert_eq!(cipher.0.w[5], 0xb12c5488);
        assert_eq!(cipher.0.w[8], 0xf295c2f2);
        assert_eq!(cipher.0.w[35], 0x2f298d7f);
        assert_eq!(cipher.0.w[36], 0xf36677ac);
        assert_eq!(cipher.0.w[40], 0xa8f914d0);
        assert_eq!(cipher.0.w[41], 0x8925eec9);
        assert_eq!(cipher.0.w[42], 0xc80c3fe1);
        assert_eq!(cipher.0.w[43], 0xa60c63b6);
    }

    #[test]
    fn aes128_encrypt() {
        let cipher = Aes128::new(&data("2b7e151628aed2a6abf7158809cf4f3c"));

        assert_eq!(
            cipher.encrypt(&data("3243f6a8885a308d313198a2e0370734")),
            data("3925841d02dc09fbdc118597196a0b32"),
        );

        assert_eq!(
            cipher.encrypt(&data("6bc1bee22e409f96e93d7e117393172a")),
            data("3ad77bb40d7a3660a89ecaf32466ef97"),
        );

        let cipher = Aes128::new(&data("000102030405060708090a0b0c0d0e0f"));
        assert_eq!(
            cipher.encrypt(&data("00112233445566778899aabbccddeeff")),
            data("69c4e0d86a7b0430d8cdb78070b4c55a"),
        );
    }

    #[test]
    fn aes256_encrypt() {
        let cipher = Aes256::new(&data(
            "603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4",
        ));

        assert_eq!(
            cipher.encrypt(&data("6bc1bee22e409f96e93d7e117393172a")),
            data("f3eed1bdb5d2a03c064b5a7e3db181f8"),
        );
    }
}
