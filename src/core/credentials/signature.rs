use base64::{engine::general_purpose, Engine};
use signature::{Signer, Verifier};
use ssh_encoding::{Decode, Encode};
use ssh_key::{rand_core::OsRng, Algorithm, PrivateKey, PublicKey};

use crate::core::credentials;

const DEFAULT_ALGO: Algorithm = Algorithm::Ed25519;

#[derive(Debug, Clone)]
pub struct Signature {
    pub signature: ssh_key::Signature,
}

impl Signature {
    pub fn generate(identity: &credentials::Identity) -> Result<PrivateKey, ssh_key::Error> {
        PrivateKey::random(&mut OsRng, DEFAULT_ALGO).and_then(|mut p| {
            p.set_comment(identity.serialise());
            Ok(p)
        })
    }

    pub fn sign(data: &[u8], private_key: &PrivateKey) -> Result<Signature, ssh_key::Error> {
        Ok(private_key.try_sign(data)?.into())
    }

    pub fn verify(&self, public_key: &PublicKey, data: &[u8]) -> bool {
        public_key.key_data().verify(data, &self.signature).is_ok()
    }

    pub fn serialise(&self) -> String {
        let mut buf = Vec::new();
        self.signature.encode(&mut buf).unwrap();
        general_purpose::STANDARD.encode(buf)
    }

    pub fn deserialise(data: &String) -> Option<Signature> {
        let Ok(decoded_bytes) = general_purpose::STANDARD.decode(data) else {
            return None;
        };
        ssh_key::Signature::decode(&mut decoded_bytes.as_slice())
            .and_then(|s| Ok(s.into()))
            .ok()
    }
}

impl Into<Signature> for ssh_key::Signature {
    fn into(self) -> Signature {
        Signature { signature: self }
    }
}
