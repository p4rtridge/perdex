use ring::{
    rand::SystemRandom,
    signature::{Ed25519KeyPair, RSA_PKCS1_SHA256, RsaKeyPair, Signature},
};

pub trait SigningKey {
    type Output: AsRef<[u8]>;

    fn sign(&self, payload: &[u8]) -> Self::Output;
}

impl SigningKey for Ed25519KeyPair {
    type Output = Signature;

    #[inline]
    fn sign(&self, payload: &[u8]) -> Self::Output {
        self.sign(payload)
    }
}

impl SigningKey for RsaKeyPair {
    type Output = Vec<u8>;

    #[inline]
    fn sign(&self, payload: &[u8]) -> Self::Output {
        let mut buf = vec![0; self.public().modulus_len()];

        let rng = SystemRandom::new();
        self.sign(&RSA_PKCS1_SHA256, &rng, payload, &mut buf)
            .expect("Failed to sign message");

        buf
    }
}

pub fn sign<K>(payload: &[u8], key: &K) -> String
where
    K: SigningKey,
{
    base64_simd::STANDARD.encode_to_string(key.sign(payload))
}
