use super::{BlockEncrypt, CtrCipher};

const B: usize = 128 / 8;
const IV_LEN: usize = 96 / 8;
const P_MAX_LEN: usize = ((1 << 39) - 256) / 8;
const A_MAX_LEN: usize = (u64::MAX / 8) as usize;

macro_rules! unroll {
    (4, $body:block) => {
        $body
        $body
        $body
        $body
    };
    (16, $body:block) => {
        unroll!(4, $body);
        unroll!(4, $body);
        unroll!(4, $body);
        unroll!(4, $body);
    };
    (64, $body:block) => {
        unroll!(16, $body);
        unroll!(16, $body);
        unroll!(16, $body);
        unroll!(16, $body);
    };
    (128, $body:block) => {
        unroll!(64, $body);
        unroll!(64, $body);
    };
}

#[allow(unused_assignments)]
const fn gmul(mut x: u128, mut y: u128) -> u128 {
    const R: u128 = 0b11100001 << 120;
    let mut z = 0;
    unroll!(128, {
        if x >> 127 != 0 {
            z ^= y;
        }
        x <<= 1;
        let t = y >> 1;
        y = if y & 1 == 0 { t } else { t ^ R };
    });
    z
}

fn ghash(h: u128, s: u128, x: impl AsRef<[u8]>) -> u128 {
    let mut y = s;
    let mut chunks = x.as_ref().chunks_exact(B);
    for b in &mut chunks {
        y = gmul(y ^ u128::from_be_bytes(b.try_into().unwrap()), h);
    }
    let remainder = chunks.remainder();
    if remainder.is_empty() {
        return y;
    }
    let mut v = remainder.to_vec();
    v.resize(B, 0);
    gmul(y ^ u128::from_be_bytes(v.try_into().unwrap()), h)
}

#[derive(Debug)]
pub enum GcmFail {
    UnsupportedTextLength,
    IncorrectAuthTag,
}

pub fn encrypt<C, const K: usize>(
    k: &[u8; K],
    iv: &[u8; IV_LEN],
    p: impl AsRef<[u8]>,
    a: impl AsRef<[u8]>,
) -> Result<(Vec<u8>, [u8; B]), GcmFail>
where
    C: BlockEncrypt<B, K>,
{
    if p.as_ref().len() > P_MAX_LEN || a.as_ref().len() > A_MAX_LEN {
        return Err(GcmFail::UnsupportedTextLength);
    }

    let mut j0 = [0; B];
    j0[B - 1] = 2; // incremented
    j0.split_at_mut(IV_LEN).0.copy_from_slice(iv.as_ref());
    let c = CtrCipher::<C, B, K, 4>::new(k, j0).encrypt(p.as_ref());

    let cipher = C::new(k);
    let h = u128::from_be_bytes(cipher.encrypt(&[0; B]));
    let s = ghash(h, 0, &a);
    let s = ghash(h, s, &c);
    let s = gmul(
        s ^ ((a.as_ref().len() as u128 * 8) << 64) ^ (c.len() as u128 * 8),
        h,
    );

    j0[B - 1] = 1;
    let t = (s ^ u128::from_be_bytes(cipher.encrypt(&j0))).to_be_bytes();

    Ok((c, t))
}

pub fn decrypt<C, const K: usize>(
    k: &[u8; K],
    iv: &[u8; IV_LEN],
    c: impl AsRef<[u8]>,
    a: impl AsRef<[u8]>,
    t: &[u8; B],
) -> Result<Vec<u8>, GcmFail>
where
    C: BlockEncrypt<B, K>,
{
    if c.as_ref().len() > P_MAX_LEN || a.as_ref().len() > A_MAX_LEN {
        return Err(GcmFail::UnsupportedTextLength);
    }

    let h = u128::from_be_bytes(C::new(k).encrypt(&[0; B]));
    let s = ghash(h, 0, &a);
    let s = ghash(h, s, &c);
    let s = gmul(
        s ^ ((a.as_ref().len() as u128 * 8) << 64) ^ (c.as_ref().len() as u128 * 8),
        h,
    );

    let mut j0 = [0; B];
    j0[B - 1] = 1;
    j0.split_at_mut(IV_LEN).0.copy_from_slice(iv.as_ref());
    let mut ctr = CtrCipher::<C, B, K, 4>::new(k, j0);

    if t != ctr.encrypt(s.to_be_bytes()).as_slice() {
        return Err(GcmFail::IncorrectAuthTag);
    }

    Ok(ctr.encrypt(c.as_ref()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::*;

    #[test]
    fn gmul_1() {
        let x = from_hex_without_spaces("00000000000000000000000000000001");
        let y = from_hex_without_spaces("00000000000000000000000000000001");
        let z = gmul(
            u128::from_be_bytes(x.as_slice().try_into().unwrap()),
            u128::from_be_bytes(y.as_slice().try_into().unwrap()),
        );
        assert_eq!(
            to_hex_without_spaces(z.to_be_bytes()),
            "e6080000000000000000000000000003"
        );
    }

    #[test]
    fn gmul_2() {
        let x = from_hex_without_spaces("acbef20579b4b8ebce889bac8732dad7");
        let y = from_hex_without_spaces("ed95f8e164bf3213febc740f0bd9c4af");
        let z = gmul(
            u128::from_be_bytes(x.as_slice().try_into().unwrap()),
            u128::from_be_bytes(y.as_slice().try_into().unwrap()),
        );
        assert_eq!(
            to_hex_without_spaces(z.to_be_bytes()),
            "4db870d37cb75fcb46097c36230d1612"
        );
    }
}
