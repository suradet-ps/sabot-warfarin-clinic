use aes_gcm::{
  aead::{Aead, KeyInit},
  Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::RngCore;
use serde::{Deserialize, Serialize};

const NONCE_SIZE: usize = 12;
const KEY_SIZE: usize = 32;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncryptedData {
  pub nonce: String,
  pub ciphertext: String,
}

pub fn generate_key() -> [u8; KEY_SIZE] {
  let mut key = [0u8; KEY_SIZE];
  rand::thread_rng().fill_bytes(&mut key);
  key
}

pub fn encrypt(plaintext: &str, key: &[u8; KEY_SIZE]) -> Result<EncryptedData, String> {
  let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

  let mut nonce_bytes = [0u8; NONCE_SIZE];
  rand::thread_rng().fill_bytes(&mut nonce_bytes);
  let nonce = Nonce::from_slice(&nonce_bytes);

  let ciphertext = cipher
    .encrypt(nonce, plaintext.as_bytes())
    .map_err(|e| e.to_string())?;

  Ok(EncryptedData {
    nonce: BASE64.encode(nonce_bytes),
    ciphertext: BASE64.encode(ciphertext),
  })
}

pub fn decrypt(encrypted: &EncryptedData, key: &[u8; KEY_SIZE]) -> Result<String, String> {
  let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;

  let nonce_bytes = BASE64
    .decode(&encrypted.nonce)
    .map_err(|e| e.to_string())?;
  let ciphertext = BASE64
    .decode(&encrypted.ciphertext)
    .map_err(|e| e.to_string())?;

  let nonce = Nonce::from_slice(&nonce_bytes);

  let plaintext = cipher
    .decrypt(nonce, ciphertext.as_ref())
    .map_err(|e| e.to_string())?;

  String::from_utf8(plaintext).map_err(|e| e.to_string())
}

pub fn encrypt_json<T: Serialize>(data: &T, key: &[u8; KEY_SIZE]) -> Result<String, String> {
  let plaintext = serde_json::to_string(data).map_err(|e| e.to_string())?;
  let encrypted = encrypt(&plaintext, key)?;
  serde_json::to_string(&encrypted).map_err(|e| e.to_string())
}

pub fn decrypt_json<T: for<'de> serde::Deserialize<'de>>(
  encrypted_json: &str,
  key: &[u8; KEY_SIZE],
) -> Result<T, String> {
  let encrypted: EncryptedData =
    serde_json::from_str(encrypted_json).map_err(|e| e.to_string())?;
  let plaintext = decrypt(&encrypted, key)?;
  serde_json::from_str(&plaintext).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_encrypt_decrypt() {
    let key = generate_key();
    let plaintext = "my_secret_password";

    let encrypted = encrypt(plaintext, &key).unwrap();
    let decrypted = decrypt(&encrypted, &key).unwrap();

    assert_eq!(plaintext, decrypted);
  }

  #[test]
  fn test_encrypt_json() {
    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Config {
      host: String,
      password: String,
    }

    let key = generate_key();
    let config = Config {
      host: "localhost".to_string(),
      password: "secret123".to_string(),
    };

    let encrypted_json = encrypt_json(&config, &key).unwrap();
    let decrypted: Config = decrypt_json(&encrypted_json, &key).unwrap();

    assert_eq!(config, decrypted);
  }
}