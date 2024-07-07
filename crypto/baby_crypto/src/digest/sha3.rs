use super::Digest;

#[derive(Clone, Copy, Default)]
struct KeccakF1600 {
    a: [u64; Self::SLICE_AREA],
}

impl KeccakF1600 {
    const SLICE_LEN: usize = 5;
    const SLICE_AREA: usize = Self::SLICE_LEN * Self::SLICE_LEN;
    const L: usize = 6;
    const W: usize = 1 << Self::L;
    const B: usize = Self::SLICE_AREA * Self::W;
    const NR: usize = 12 + 2 * Self::L;

    fn get(&self, x: usize, y: usize) -> u64 {
        self.a[x + y * Self::SLICE_LEN]
    }

    fn get_mut(&mut self, x: usize, y: usize) -> &mut u64 {
        &mut self.a[x + y * Self::SLICE_LEN]
    }

    fn theta(&mut self) {
        let mut chunks = self.a.chunks_exact(Self::SLICE_LEN);
        let mut c: [u64; Self::SLICE_LEN] = chunks.next().unwrap().try_into().unwrap();
        for chunk in chunks {
            for (x, y) in c.iter_mut().zip(chunk) {
                *x ^= y;
            }
        }

        for x in 0..Self::SLICE_LEN {
            let d = c[(x + 1) % Self::SLICE_LEN].rotate_left(1)
                ^ c[(x + Self::SLICE_LEN - 1) % Self::SLICE_LEN];
            for y in 0..Self::SLICE_LEN {
                *self.get_mut(x, y) ^= d;
            }
        }
    }

    const fn calc_rho_offset() -> [u32; Self::SLICE_AREA] {
        let mut offset = [0; Self::SLICE_AREA];
        let mut t = 1;
        let mut x = 1;
        let mut y = 0;
        while t < Self::SLICE_AREA as u32 {
            offset[x + y * 5] = t * (t + 1) / 2;
            (x, y) = (y, (2 * x + 3 * y) % Self::SLICE_LEN);
            t += 1;
        }
        offset
    }

    const RHO_OFFSET: [u32; Self::SLICE_AREA] = Self::calc_rho_offset();

    fn rho(&mut self) {
        for i in 1..Self::SLICE_AREA {
            self.a[i] = self.a[i].rotate_left(Self::RHO_OFFSET[i]);
        }
    }

    fn pi(&mut self) {
        let old = *self;
        for x in 0..Self::SLICE_LEN {
            for y in 0..Self::SLICE_LEN {
                *self.get_mut(x, y) = old.get((x + 3 * y) % Self::SLICE_LEN, x);
            }
        }
    }

    fn chi(&mut self) {
        let old = *self;
        for x in 0..Self::SLICE_LEN {
            for y in 0..Self::SLICE_LEN {
                *self.get_mut(x, y) ^=
                    !old.get((x + 1) % Self::SLICE_LEN, y) & old.get((x + 2) % Self::SLICE_LEN, y);
            }
        }
    }

    const fn rc(t: usize) -> bool {
        let mut r = 1u8;
        let mut i = 0;
        while i < t % 255 {
            r = (r << 1) ^ (((r >> 7) & 1) * 0b1110001);
            i += 1;
        }
        r & 1 == 1
    }

    const fn calc_rc() -> [u64; Self::NR] {
        let mut result = [0; Self::NR];
        let mut ir = 0;
        while ir < Self::NR {
            let mut j = 0;
            while j <= Self::L {
                if Self::rc(j + 7 * ir) {
                    result[ir] ^= 1 << ((1 << j) - 1);
                }
                j += 1;
            }
            ir += 1;
        }
        result
    }

    const RC: [u64; Self::NR] = Self::calc_rc();

    fn iota(&mut self, ir: usize) {
        self.a[0] ^= Self::RC[ir];
    }

    fn f(&mut self) {
        for ir in 0..Self::NR {
            self.theta();
            self.rho();
            self.pi();
            self.chi();
            self.iota(ir);
        }
    }
}

impl std::fmt::Debug for KeccakF1600 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for lane in self.a {
            writeln!(f)?;
            for b in lane.to_le_bytes() {
                write!(f, "{b:02x} ")?;
            }
        }
        Ok(())
    }
}

// generic `Self` types are currently not permitted in anonymous constants
// So we need all three const generic parameters
struct Sha3<const C: usize, const D: usize, const BLOCK_BYTES: usize> {
    s: KeccakF1600,
    m: [u8; BLOCK_BYTES],
    len_mod_block_bytes: usize,
}

impl<const C: usize, const D: usize, const BLOCK_BYTES: usize> Sha3<C, D, BLOCK_BYTES> {
    fn consume_block(&mut self) {
        Self::update_state_with_block(&mut self.s, &self.m);
    }

    fn process_block(&mut self, m: &[u8]) {
        Self::update_state_with_block(&mut self.s, m);
    }

    fn update_state_with_block(state: &mut KeccakF1600, m: &[u8]) {
        for i in 0..BLOCK_BYTES / 8 {
            state.a[i] ^= u64::from_le_bytes(m[i * 8..i * 8 + 8].try_into().unwrap());
        }
        state.f();
    }
}

impl<const C: usize, const D: usize, const BLOCK_BYTES: usize> Digest<D>
    for Sha3<C, D, BLOCK_BYTES>
{
    const BLOCK_BYTES: usize = BLOCK_BYTES;

    fn new() -> Self {
        Self {
            s: Default::default(),
            m: [0; BLOCK_BYTES],
            len_mod_block_bytes: 0,
        }
    }

    fn update(&mut self, data: impl AsRef<[u8]>) {
        let mut data = data.as_ref();

        let p = self.len_mod_block_bytes;
        self.len_mod_block_bytes = p.wrapping_add(data.len()) % BLOCK_BYTES;

        if p != 0 {
            let block_remain_bytes = BLOCK_BYTES - p;
            if data.len() < block_remain_bytes {
                self.m[p..p + data.len()].copy_from_slice(data);
                return;
            }
            let (l, r) = data.split_at(block_remain_bytes);
            self.m[p..].copy_from_slice(l);
            self.consume_block();
            data = r;
        }

        while data.len() >= BLOCK_BYTES {
            let (l, r) = data.split_at(BLOCK_BYTES);
            self.process_block(l);
            data = r;
        }

        self.m[..data.len()].copy_from_slice(data);
    }

    fn finalize(mut self) -> [u8; D] {
        self.m[self.len_mod_block_bytes] = 0b110;
        self.m[self.len_mod_block_bytes + 1..].fill(0);
        self.m[BLOCK_BYTES - 1] |= 1 << 7;
        self.consume_block();

        assert!(BLOCK_BYTES >= D); // holds for all SHA3 variants
        let mut result = [0; D];
        for i in 0..D / 8 {
            result[i * 8..i * 8 + 8].copy_from_slice(&self.s.a[i].to_le_bytes());
        }
        result
    }
}

macro_rules! sha3_variant {
    ($name:ident, $d:expr) => {
        pub struct $name(Sha3<{ $d * 2 }, { $d / 8 }, { (KeccakF1600::B - $d * 2) / 8 }>);

        impl Digest<{ $d / 8 }> for $name {
            const BLOCK_BYTES: usize = (KeccakF1600::B - $d * 2) / 8;

            fn new() -> Self {
                Self(Digest::new())
            }

            fn update(&mut self, data: impl AsRef<[u8]>) {
                self.0.update(data)
            }

            fn finalize(self) -> [u8; $d / 8] {
                self.0.finalize()
            }
        }
    };
}

sha3_variant!(Sha3_224, 224);
sha3_variant!(Sha3_256, 256);
sha3_variant!(Sha3_384, 384);
sha3_variant!(Sha3_512, 512);

#[cfg(test)]
mod tests {
    use super::*;

    mod sha3_256 {
        use super::*;
        use crate::hex::*;

        #[test]
        fn hash_empty() {
            assert_eq!(
                to_hex_without_spaces(Sha3_256::hash([])),
                "a7ffc6f8bf1ed76651c14756a061d662f580ff4de43b49fa82d80a4b80f8434a"
            );
        }

        #[test]
        fn hash_abc() {
            assert_eq!(
                to_hex_without_spaces(Sha3_256::hash(b"abc")),
                "3a985da74fe225b2045c172d6bd390bd855f086e3e9d525b46bfe24511431532"
            );
        }

        #[test]
        fn hash_1080() {
            let mut hasher = Sha3_256::new();
            for _ in 0..135 {
                hasher.update(b"a");
            }
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "8094bb53c44cfb1e67b7c30447f9a1c33696d2463ecc1d9c92538913392843c9"
            );
        }

        #[test]
        fn hash_1088() {
            assert_eq!(
                to_hex_without_spaces(Sha3_256::hash(b"b".repeat(136))),
                "491d43679ebf9eeb191b33432034caebed97df8be9125a6db9b133c7ce660ca7"
            );
        }

        #[test]
        fn hash_1600() {
            assert_eq!(
                to_hex_without_spaces(Sha3_256::hash(b"ouuan".repeat(40))),
                "920937562d4e7587c7029657ac422bf833283c45944cf0847e9cb95d1357c060"
            );
        }

        #[test]
        fn hash_1608() {
            let mut hasher = Sha3_256::new();
            for _ in 0..67 {
                hasher.update(b"417");
            }
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "2fe4e24b78aca771f4042a79b272bd08490f550adc77487914fd8de4eeb89e9e"
            );
        }

        #[test]
        fn hash_2048() {
            let mut hasher = Sha3_256::new();
            for len in 62..=65 {
                hasher.update(b"x".repeat(len));
            }
            hasher.update("rs");
            assert_eq!(
                to_hex_without_spaces(hasher.finalize()),
                "029b5e7fa8a1d7c2f0ddc193dcfda42911af27e936fe4b069f53cd0ed30540f0"
            );
        }
    }
}
