mod sha2;
mod sha3;

pub trait Digest<const L: usize>: Sized {
    const BLOCK_BYTES: usize;

    fn new() -> Self;

    fn update(&mut self, data: impl AsRef<[u8]>);

    fn finalize(self) -> [u8; L];

    fn hash(data: impl AsRef<[u8]>) -> [u8; L] {
        let mut hasher = Self::new();
        hasher.update(data);
        hasher.finalize()
    }
}

pub use sha2::*;
pub use sha3::*;
