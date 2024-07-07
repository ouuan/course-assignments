mod aes;
mod ctr;
mod gcm;

pub trait BlockEncrypt<const B: usize, const K: usize> {
    fn new(key: &[u8; K]) -> Self;

    fn encrypt(&self, data: &[u8; B]) -> [u8; B];
}

pub use aes::*;
pub use ctr::CtrCipher;

pub type Aes128Ctr = CtrCipher<Aes128, 16, 16>;
pub type Aes192Ctr = CtrCipher<Aes192, 16, 24>;
pub type Aes256Ctr = CtrCipher<Aes256, 16, 32>;

pub use gcm::GcmFail;

pub fn aes_128_gcm_encrypt(
    k: &[u8; 16],
    iv: &[u8; 12],
    p: impl AsRef<[u8]>,
    a: impl AsRef<[u8]>,
) -> Result<(Vec<u8>, [u8; 16]), GcmFail> {
    gcm::encrypt::<Aes128, 16>(k, iv, p, a)
}

pub fn aes_128_gcm_decrypt(
    k: &[u8; 16],
    iv: &[u8; 12],
    c: impl AsRef<[u8]>,
    a: impl AsRef<[u8]>,
    t: &[u8; 16],
) -> Result<Vec<u8>, GcmFail> {
    gcm::decrypt::<Aes128, 16>(k, iv, c, a, t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::*;

    fn data<const SIZE: usize>(s: &str) -> [u8; SIZE] {
        from_hex_without_spaces(s).try_into().unwrap()
    }

    #[test]
    fn aes128ctr() {
        let mut cipher = Aes128Ctr::new(
            &data("2b7e151628aed2a6abf7158809cf4f3c"),
            data("f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"),
        );

        assert_eq!(
            to_hex_without_spaces(cipher.encrypt(from_hex_without_spaces("6bc1bee2"))),
            "874d6191"
        );

        assert_eq!(
            to_hex_without_spaces(cipher.encrypt(from_hex_without_spaces("2e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e5130c81c46a35ce411e5fbc1191a0a52ef"))),
            "b620e3261bef6864990db6ce9806f66b7970fdff8617187bb9fffdff5ae4df3edbd5d35e5b4f09020db03eab"
        );

        assert_eq!(
            to_hex_without_spaces(
                cipher.encrypt(from_hex_without_spaces("f69f2445df4f9b17ad2b417be66c3710"))
            ),
            "1e031dda2fbe03d1792170a0f3009cee"
        );
    }

    #[test]
    fn aes_128_gcm_empty() {
        let k = from_hex_without_spaces("d480429666d48b400633921c5407d1d1");
        let iv = from_hex_without_spaces("3388c676dc754acfa66e172a");
        let a = [];
        let p = [];
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            p,
            a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(c), "");
        assert_eq!(to_hex_without_spaces(t), "7d7daf44850921a34e636b01adeb104f");
    }

    #[test]
    fn aes_128_gcm_384p() {
        let k = from_hex_without_spaces("00000000000000000000000000000000");
        let iv = from_hex_without_spaces("000000000000000000000000");
        let a = [];
        let p = from_hex_without_spaces("000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            p,
            a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(c), "0388dace60b6a392f328c2b971b2fe78f795aaab494b5923f7fd89ff948bc1e0200211214e7394da2089b6acd093abe0");
        assert_eq!(to_hex_without_spaces(t), "9dd0a376b08e40eb00c35f29f9ea61a4");
    }

    #[test]
    fn aes_128_gcm_512p() {
        let k = from_hex_without_spaces("feffe9928665731c6d6a8f9467308308");
        let iv = from_hex_without_spaces("cafebabefacedbaddecaf888");
        let a = [];
        let p = from_hex_without_spaces("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255");
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            p,
            a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(c), "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091473f5985");
        assert_eq!(to_hex_without_spaces(t), "4d5c2af327cd64a62cf35abd2ba6fab4");
    }

    #[test]
    fn aes_128_gcm_1024a() {
        let k = from_hex_without_spaces("00000000000000000000000000000000");
        let iv = from_hex_without_spaces("000000000000000000000000");
        let p = [];
        let a = from_hex_without_spaces("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b391aafd255522dc1f099567d07f47f37a32a84427d643a8cdcbfe5c0c97598a2bd2555d1aa8cb08e48590dbb3da7b08b1056828838c5f61e6393ba7a0abcc9f662898015ad");
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            p,
            a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(c), "");
        assert_eq!(to_hex_without_spaces(t), "5fea793a2d6f974d37e68e0cb8ff9492");
    }

    #[test]
    fn aes_128_gcm_80p_80a() {
        let k = from_hex_without_spaces("3881e7be1bb3bbcaff20bdb78e5d1b67");
        let iv = from_hex_without_spaces("dcf5b7ae2d7552e2297fcfa9");
        let a = from_hex_without_spaces("c60c64bbf7");
        let p = from_hex_without_spaces("0a2714aa7d");
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            p,
            a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(c), "5626f96ecb");
        assert_eq!(to_hex_without_spaces(t), "ff4c4f1d92b0abb1d0820833d9eb83c7");
    }

    #[test]
    fn aes_128_gcm_480p_160a() {
        let k = from_hex_without_spaces("feffe9928665731c6d6a8f9467308308");
        let iv = from_hex_without_spaces("cafebabefacedbaddecaf888");
        let a = from_hex_without_spaces("feedfacedeadbeeffeedfacedeadbeefabaddad2");
        let p = from_hex_without_spaces("d9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39");
        let (c, t) = aes_128_gcm_encrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            &p,
            &a,
        )
        .unwrap();
        assert_eq!(to_hex_without_spaces(&c), "42831ec2217774244b7221b784d0d49ce3aa212f2c02a4e035c17e2329aca12e21d514b25466931c7d8f6a5aac84aa051ba30b396a0aac973d58e091");
        assert_eq!(to_hex_without_spaces(t), "5bc94fbc3221a5db94fae95ae7121a47");
        assert!(aes_128_gcm_decrypt(
            k.as_slice().try_into().unwrap(),
            iv.as_slice().try_into().unwrap(),
            &c,
            from_hex_without_spaces("feedfacedeadbeeffeedfacedeadbeefabaddad1"),
            &t
        )
        .is_err());
        assert_eq!(
            aes_128_gcm_decrypt(
                k.as_slice().try_into().unwrap(),
                iv.as_slice().try_into().unwrap(),
                &c,
                &a,
                &t
            )
            .unwrap(),
            p
        );
    }
}
