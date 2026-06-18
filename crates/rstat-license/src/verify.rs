use base64::{Engine as _, engine::general_purpose};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub const EMBEDDED_PUBLIC_KEY_HEX: &str =
    "a3eb9cc3d582dd90324368ab3e7e613c2e887d39497cf33c6fc73c7a706d16a9";

#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("Invalid public key format: {0}")]
    InvalidPublicKey(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Licensee {
    pub name: String,
    pub email: String,
    pub order_id: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct LicenseFile {
    pub licensee: Licensee,
    pub tier: String,
    pub features: Vec<String>,
    pub signature: String,
}

impl LicenseFile {
    pub fn to_signing_payload(&self) -> Result<Vec<u8>, serde_json::Error> {
        #[derive(Serialize)]
        struct Payload<'a> {
            licensee: &'a Licensee,
            tier: &'a str,
            features: &'a [String],
        }
        let payload = Payload {
            licensee: &self.licensee,
            tier: &self.tier,
            features: &self.features,
        };
        serde_json::to_vec(&payload)
    }
}

pub struct LicenseVerifier;

impl LicenseVerifier {
    pub fn verify(license: &LicenseFile) -> Result<(), LicenseError> {
        // Parse embedded public key
        let pub_key_bytes = hex::decode(EMBEDDED_PUBLIC_KEY_HEX)
            .map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;
        let pub_key_array: [u8; 32] = pub_key_bytes.try_into().map_err(|_| {
            LicenseError::InvalidPublicKey("Public key must be 32 bytes".to_string())
        })?;
        let verifying_key = VerifyingKey::from_bytes(&pub_key_array)
            .map_err(|e| LicenseError::InvalidPublicKey(e.to_string()))?;

        // Calculate signing payload
        let payload_bytes = license
            .to_signing_payload()
            .map_err(|e| LicenseError::SerializationError(e.to_string()))?;

        // Parse signature
        let sig_bytes = general_purpose::STANDARD
            .decode(&license.signature)
            .map_err(|e| LicenseError::InvalidSignature(e.to_string()))?;
        let signature = Signature::from_slice(&sig_bytes)
            .map_err(|e| LicenseError::InvalidSignature(e.to_string()))?;

        // Verify signature
        verifying_key
            .verify(&payload_bytes, &signature)
            .map_err(|e| LicenseError::VerificationFailed(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    #[test]
    fn test_dynamic_sign_and_verify() {
        // 1. Generate keypair dynamically
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        // 2. Setup licensee and license file
        let licensee = Licensee {
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
            order_id: "12345".to_string(),
        };
        let features = vec!["spc".to_string(), "capability".to_string()];

        let mut license = LicenseFile {
            licensee,
            tier: "pro".to_string(),
            features,
            signature: String::new(),
        };

        // 3. Create signing payload
        let payload_bytes = license.to_signing_payload().unwrap();

        // 4. Sign and set signature
        let signature = signing_key.sign(&payload_bytes);
        license.signature = general_purpose::STANDARD.encode(signature.to_bytes());

        // 5. Verify manually with dynamic key
        let sig_bytes = general_purpose::STANDARD
            .decode(&license.signature)
            .unwrap();
        let sig = Signature::from_slice(&sig_bytes).unwrap();
        assert!(verifying_key.verify(&payload_bytes, &sig).is_ok());
    }

    #[test]
    fn test_embedded_key_verification() {
        // Our recorded private key matching EMBEDDED_PUBLIC_KEY_HEX
        let priv_key_hex = "6866f18b94014e2aedb0082362f3656791f589a97c70e1265c15622986b01147";
        let priv_key_bytes = hex::decode(priv_key_hex).unwrap();
        let priv_key_array: [u8; 32] = priv_key_bytes.try_into().unwrap();
        let signing_key = SigningKey::from_bytes(&priv_key_array);

        let licensee = Licensee {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            order_id: "ORD-999".to_string(),
        };
        let mut license = LicenseFile {
            licensee,
            tier: "pro".to_string(),
            features: vec!["spc".to_string(), "capability".to_string()],
            signature: String::new(),
        };

        let payload_bytes = license.to_signing_payload().unwrap();
        let signature = signing_key.sign(&payload_bytes);
        license.signature = general_purpose::STANDARD.encode(signature.to_bytes());

        // Verify using the real verifier (which uses EMBEDDED_PUBLIC_KEY_HEX)
        assert!(LicenseVerifier::verify(&license).is_ok());

        // Mutate one char of license and verify it fails
        license.licensee.name = "John Doe2".to_string();
        assert!(LicenseVerifier::verify(&license).is_err());
    }
}
