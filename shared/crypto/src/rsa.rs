use aes::Aes256;
use base64::prelude::*;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use rand::RngCore;
use rsa::{
    Oaep, RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey},
    pkcs8::{DecodePrivateKey, DecodePublicKey},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::encryption::CryptoError;

type Aes256CbcEnc = cbc::Encryptor<Aes256>;
type Aes256CbcDec = cbc::Decryptor<Aes256>;

/// The JSON envelope format produced and consumed by `hybrid-crypto-js`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HybridPayload {
    pub v: String,
    pub iv: String,
    pub keys: HashMap<String, String>,
    pub cipher: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Decrypts a payload that was encrypted with `hybrid-crypto-js` using the server's RSA private key.
///
/// Steps:
/// 1. Parse the JSON `HybridPayload` envelope.
/// 2. Extract and Base64-decode the RSA-encrypted AES key from `payload.keys`.
/// 3. Decrypt the 32-byte AES key using RSA-OAEP (SHA-1) with `private_key_pem`.
/// 4. Base64-decode the IV (taking the first 16 bytes for AES-256-CBC).
/// 5. Base64-decode the AES ciphertext.
/// 6. Decrypt the ciphertext using AES-256-CBC with PKCS#7 unpadding.
pub fn decrypt_hybrid_rsa(
    payload_json: &str,
    private_key_pem: &str,
) -> Result<String, CryptoError> {
    // 1. Parse JSON envelope
    let payload: HybridPayload =
        serde_json::from_str(payload_json.trim()).map_err(CryptoError::JsonError)?;

    // 2. Parse RSA private key (support PKCS#1 format standard in Virat)
    let private_key = RsaPrivateKey::from_pkcs1_pem(private_key_pem)
        .or_else(|_| RsaPrivateKey::from_pkcs8_pem(private_key_pem))
        .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid RSA private key: {e}")))?;

    // 3. Find and decode the encrypted AES key from payload.keys
    let encrypted_key_b64 =
        payload.keys.values().next().ok_or_else(|| {
            CryptoError::DecryptionFailed("Missing encrypted key in payload".into())
        })?;

    let encrypted_key_bytes = BASE64_STANDARD
        .decode(encrypted_key_b64.trim())
        .map_err(|e| CryptoError::InvalidBase64(e.to_string()))?;

    // 4. Decrypt AES key with RSA-OAEP (SHA-1)
    let padding = Oaep::new::<sha1::Sha1>();
    let aes_key = private_key
        .decrypt(padding, &encrypted_key_bytes)
        .map_err(|e| CryptoError::DecryptionFailed(format!("RSA key decryption failed: {e}")))?;

    if aes_key.len() < 32 {
        return Err(CryptoError::DecryptionFailed(format!(
            "Invalid decrypted AES key length: expected 32, got {}",
            aes_key.len()
        )));
    }

    // 5. Decode IV (take first 16 bytes for AES block size)
    let iv_bytes = BASE64_STANDARD
        .decode(payload.iv.trim())
        .map_err(|e| CryptoError::InvalidBase64(e.to_string()))?;

    if iv_bytes.len() < 16 {
        return Err(CryptoError::DecryptionFailed(format!(
            "IV too short for AES-CBC: expected >= 16 bytes, got {}",
            iv_bytes.len()
        )));
    }
    let iv = &iv_bytes[..16];

    // 6. Decode and decrypt ciphertext with AES-256-CBC
    let cipher_bytes = BASE64_STANDARD
        .decode(payload.cipher.trim())
        .map_err(|e| CryptoError::InvalidBase64(e.to_string()))?;

    let decryptor = Aes256CbcDec::new(aes_key[..32].into(), iv.into());
    let mut buf = cipher_bytes;
    let decrypted = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buf)
        .map_err(|e| CryptoError::DecryptionFailed(format!("AES-CBC decrypt failed: {e:?}")))?;

    String::from_utf8(decrypted.to_vec()).map_err(|e| CryptoError::InvalidUtf8(e.to_string()))
}

/// Encrypts plaintext using `hybrid-crypto-js` compatible format with a client's RSA public key.
///
/// Used for outbound responses in VAPT mode when `!is_default_encryption_url`.
pub fn encrypt_hybrid_rsa(plaintext: &str, public_key_pem: &str) -> Result<String, CryptoError> {
    // 1. Parse RSA public key (support SubjectPublicKeyInfo / SPKI and PKCS#1)
    let public_key = RsaPublicKey::from_public_key_pem(public_key_pem)
        .or_else(|_| RsaPublicKey::from_pkcs1_pem(public_key_pem))
        .map_err(|e| CryptoError::DecryptionFailed(format!("Invalid RSA public key: {e}")))?;

    let mut rng = rand::thread_rng();

    // 2. Generate random 32-byte AES key and 16-byte IV
    let mut aes_key = [0u8; 32];
    let mut iv = [0u8; 16];
    rng.fill_bytes(&mut aes_key);
    rng.fill_bytes(&mut iv);

    // 3. Encrypt AES key using RSA-OAEP (SHA-1)
    let padding = Oaep::new::<sha1::Sha1>();
    let encrypted_key = public_key
        .encrypt(&mut rng, padding, &aes_key)
        .map_err(|e| CryptoError::DecryptionFailed(format!("RSA encryption failed: {e}")))?;

    // 4. Encrypt plaintext with AES-256-CBC
    let encryptor = Aes256CbcEnc::new(&aes_key.into(), &iv.into());
    let ciphertext = encryptor.encrypt_padded_vec_mut::<Pkcs7>(plaintext.as_bytes());

    // 5. Construct fingerprint (or default identifier)
    let mut keys = HashMap::new();
    keys.insert(
        "client_key".to_string(),
        BASE64_STANDARD.encode(encrypted_key),
    );

    let payload = HybridPayload {
        v: "hybrid-crypto-js_0.2.4".to_string(),
        iv: BASE64_STANDARD.encode(iv),
        keys,
        cipher: BASE64_STANDARD.encode(ciphertext),
        signature: None,
    };

    serde_json::to_string(&payload).map_err(CryptoError::JsonError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsa_roundtrip() {
        let mut rng = rand::thread_rng();
        let private_key = RsaPrivateKey::new(&mut rng, 2048).expect("failed to generate key");
        let public_key = RsaPublicKey::from(&private_key);

        use rsa::pkcs1::EncodeRsaPrivateKey;
        use rsa::pkcs8::EncodePublicKey;

        let priv_pem = private_key
            .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
            .unwrap();
        let pub_pem = public_key
            .to_public_key_pem(rsa::pkcs8::LineEnding::LF)
            .unwrap();

        let original = r#"{"user_id":"12345","action":"verify_otp"}"#;

        let encrypted = encrypt_hybrid_rsa(original, &pub_pem).expect("encryption failed");
        let decrypted = decrypt_hybrid_rsa(&encrypted, &priv_pem).expect("decryption failed");

        assert_eq!(decrypted, original);
    }
}
