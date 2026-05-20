use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use rand::RngCore;
use std::env;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64_STANDARD};


type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

const IV_LENGTH: usize = 16;

fn get_key() -> [u8; 32] {
    let key_str = env::var("ENCRYPTION_KEY")
        .unwrap_or_else(|_| "a-very-secret-key-32-chars-long-!!".to_string());
    let mut key = [0u8; 32];
    let bytes = key_str.as_bytes();
    let len = bytes.len().min(32);
    key[..len].copy_from_slice(&bytes[..len]);
    key
}

/// Encrypt a string using AES-256-CBC.
/// Returns `iv_hex:encrypted_hex` format (matches aztea-store crypto.ts)
pub fn encrypt(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }
    let key = get_key();
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut iv);

    let encryptor = Aes256CbcEnc::new(&key.into(), &iv.into());
    let encrypted = encryptor.encrypt_padded_vec_mut::<Pkcs7>(text.as_bytes());

    format!("{}:{}", hex::encode(iv), hex::encode(encrypted))
}

/// Decrypt a string using AES-256-CBC.
/// Expects `iv_hex:encrypted_hex` format (matches aztea-store crypto.ts)
/// If the format is invalid, returns the original text (handles plain-text legacy values).
pub fn decrypt(text: &str) -> String {
    if text.is_empty() || !text.contains(':') {
        return text.to_string();
    }
    let parts: Vec<&str> = text.splitn(2, ':').collect();
    if parts.len() != 2 || parts[0].len() != 32 {
        // IV must be 16 bytes = 32 hex chars
        return text.to_string();
    }

    let iv_bytes = match hex::decode(parts[0]) {
        Ok(b) => b,
        Err(_) => return text.to_string(),
    };
    let enc_bytes = match hex::decode(parts[1]) {
        Ok(b) => b,
        Err(_) => return text.to_string(),
    };

    let key = get_key();
    let iv: [u8; IV_LENGTH] = match iv_bytes.try_into() {
        Ok(v) => v,
        Err(_) => return text.to_string(),
    };

    let decryptor = Aes256CbcDec::new(&key.into(), &iv.into());
    match decryptor.decrypt_padded_vec_mut::<Pkcs7>(&enc_bytes) {
        Ok(decrypted) => String::from_utf8(decrypted).unwrap_or_else(|_| text.to_string()),
        Err(_) => text.to_string(),
    }
}

fn is_valid_fingerprint_format(text: &str) -> bool {
    text.starts_with("MAC@") && text.contains(":CPU_ID@") && text.contains(":OS@")
}

pub fn validate_and_decrypt_fingerprint(b64_fingerprint: &str) -> Result<String, String> {
    let generic_err = "Empreinte non valide".to_string();

    let combined_bytes = BASE64_STANDARD
        .decode(b64_fingerprint)
        .map_err(|_| generic_err.clone())?;

    if combined_bytes.len() < IV_LENGTH {
        return Err(generic_err.clone());
    }

    let (iv_bytes, enc_bytes) = combined_bytes.split_at(IV_LENGTH);

    let key = get_key();
    let iv: [u8; IV_LENGTH] = iv_bytes.try_into().map_err(|_| generic_err.clone())?;

    let decryptor = Aes256CbcDec::new(&key.into(), &iv.into());
    
    let decrypted = decryptor.decrypt_padded_vec_mut::<Pkcs7>(enc_bytes)
        .map_err(|_| generic_err.clone())?;

    let decrypted_text = String::from_utf8(decrypted)
        .map_err(|_| generic_err.clone())?;

    if !is_valid_fingerprint_format(&decrypted_text) {
        return Err(generic_err.clone());
    }

    Ok(decrypted_text)
}
