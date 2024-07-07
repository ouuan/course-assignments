mod hmac;

pub trait Mac<const L: usize>: Sized {
    fn new(key: impl AsRef<[u8]>) -> Self;

    fn update(&mut self, data: impl AsRef<[u8]>);

    fn finalize(self) -> [u8; L];

    fn mac(key: impl AsRef<[u8]>, data: impl AsRef<[u8]>) -> [u8; L] {
        let mut mac = Self::new(key);
        mac.update(data);
        mac.finalize()
    }
}

pub use hmac::*;
