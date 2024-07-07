use super::BlockEncrypt;

/// T: block cipher
/// B: block size
/// K: key length
/// S: increment part length (S = B in CTR mode, S = 32/8 = 4 in GCM mode)
pub struct CtrCipher<C, const B: usize, const K: usize, const S: usize = B>
where
    C: BlockEncrypt<B, K>,
{
    cipher: C,
    counter: [u8; B],
    encrypted_counter: [u8; B],
    p: usize,
}

impl<C, const B: usize, const K: usize, const S: usize> CtrCipher<C, B, K, S>
where
    C: BlockEncrypt<B, K>,
{
    pub fn new(key: &[u8; K], nonce: [u8; B]) -> Self {
        let cipher = C::new(key);
        let encrypted_counter = cipher.encrypt(&nonce);
        Self {
            cipher,
            counter: nonce,
            encrypted_counter,
            p: 0,
        }
    }

    fn increment_counter(&mut self) {
        for i in (B - S..B).rev() {
            let (result, overflow) = self.counter[i].overflowing_add(1);
            self.counter[i] = result;
            if !overflow {
                break;
            }
        }
        self.encrypted_counter = self.cipher.encrypt(&self.counter);
    }

    pub fn encrypt(&mut self, data: impl AsRef<[u8]>) -> Vec<u8> {
        let data = data.as_ref();
        let mut result = Vec::with_capacity(data.len());

        if data.len() <= B - self.p {
            for (i, x) in data.iter().enumerate() {
                result.push(x ^ self.encrypted_counter[self.p + i]);
            }
            self.p += data.len();
            return result;
        }

        let (l, data) = data.split_at(B - self.p);
        for (i, x) in l.iter().enumerate() {
            result.push(x ^ self.encrypted_counter[self.p + i]);
        }

        for chunk in data.chunks(B) {
            self.increment_counter();
            for (i, x) in chunk.iter().enumerate() {
                result.push(x ^ self.encrypted_counter[i]);
            }
        }

        self.p = data.len() % B;
        if self.p == 0 {
            self.p = B;
        }

        result
    }
}
