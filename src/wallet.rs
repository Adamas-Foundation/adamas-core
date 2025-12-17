use pqcrypto_dilithium::dilithium5::*; 
use pqcrypto_traits::sign::{SecretKey as _, PublicKey as _, DetachedSignature as _};

pub struct Wallet {
    pub public_key: String,
    secret_key_bytes: Box<[u8]>,
}

impl Wallet {
    pub fn new() -> Self {
        println!("   [CRYPTO] Initializing Post-Quantum Engine (Dilithium-5)...");
        
        let (pk, sk) = keypair();
        let pk_hex = hex::encode(pk.as_bytes());
        
        Wallet {
            public_key: pk_hex,
            secret_key_bytes: Box::from(sk.as_bytes()),
        }
    }

    pub fn sign(&self, message: &str) -> String {
        let sk = SecretKey::from_bytes(&self.secret_key_bytes).expect("Key Error");
        let signature = detached_sign(message.as_bytes(), &sk);
        hex::encode(signature.as_bytes())
    }

    pub fn verify(message: &str, signature_hex: &str, public_key_hex: &str) -> bool {
        let pk_bytes = match hex::decode(public_key_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };
        
        let sig_bytes = match hex::decode(signature_hex) {
            Ok(b) => b,
            Err(_) => return false,
        };

        let pk = match PublicKey::from_bytes(&pk_bytes) {
            Ok(k) => k,
            Err(_) => return false,
        };

        let sig = match DetachedSignature::from_bytes(&sig_bytes) {
            Ok(s) => s,
            Err(_) => return false,
        };

        verify_detached_signature(&sig, message.as_bytes(), &pk).is_ok()
    }
}