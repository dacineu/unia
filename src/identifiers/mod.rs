use sha2::{Sha256, Digest};
use serde_json::Value;
use uuid::Uuid;
use aes_gcm::{Aes256Gcm, Key, Nonce, KeyInit, aead::Aead};

#[derive(Debug, thiserror::Error)]
pub enum UreIdError {
    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("General error: {0}")]
    General(String),
}

/// Deterministic URE-UUID (DU-UUID) implementation.
/// Generates a UUID based on the content of a .ure manifest.
pub struct DuUuid;

impl DuUuid {
    /// Generates a UUID from a .ure manifest.
    /// 
    /// # Arguments
    /// * `manifest` - The .ure manifest as a JSON Value.
    /// * `encryption_key` - Optional 32-byte key for AES-256-GCM encryption.
    pub fn generate(manifest: &Value, encryption_key: Option<&[u8; 32]>) -> Result<Uuid, UreIdError> {
        // 1. Externalize the ID to prevent circular dependency
        let mut content = manifest.clone();
        if let Some(obj) = content.as_object_mut() {
            obj.remove("resource_id");
        }

        // 2. Deterministic Serialization
        // Note: serde_json's Value doesn't guarantee key order in the map
        // For true determinism, we would use a BTreeMap or canonical JSON.
        let serialized = serde_json::to_vec(&content)?;

        // 3. Payload Processing (Compression simulation & Encryption)
        // In a full implementation, a deterministic compression like zstd would go here.
        let payload = if let Some(key_bytes) = encryption_key {
            Self::encrypt_payload(&serialized, key_bytes)?
        } else {
            serialized
        };

        // 4. Hashing (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(&payload);
        let result = hasher.finalize();

        // 5. Map to UUID (First 16 bytes of SHA-256)
        let mut bytes = [0u8; 16];
        bytes.copy_from_slice(&result[..16]);

        // Set Version 4 (0100) at index 6 (bits 48-51)
        bytes[6] = (bytes[6] & 0x0f) | 0x40;
        // Set Variant RFC 4122 (10xx) at index 8 (bits 64-65)
        bytes[8] = (bytes[8] & 0x3f) | 0x80;

        Ok(Uuid::from_bytes(bytes))
    }

    fn encrypt_payload(data: &[u8], key_bytes: &[u8; 32]) -> Result<Vec<u8>, UreIdError> {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        // For deterministic UUIDs, the nonce must be deterministic.
        // WARNING: In standard encryption, nonces must NEVER be reused.
        // Here, since the purpose is generating a unique ID from content (not secrecy),
        // a fixed nonce is used to ensure the same content -> same UUID.
        let nonce = Nonce::from_slice(b"ure_det_nonc"); // exactly 12 bytes

        cipher.encrypt(nonce, data)
            .map_err(|e| UreIdError::Encryption(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deterministic_generation() {
        let manifest = json!({
            "resource_id": "old-id",
            "quality_of_product": 0.95,
            "checksum": "abc123hash"
        });

        let id1 = DuUuid::generate(&manifest, None).unwrap();
        let id2 = DuUuid::generate(&manifest, None).unwrap();

        assert_eq!(id1, id2);
        assert_eq!(id1.get_version(), Some(uuid::Version::Random));
    }

    #[test]
    fn test_content_change_changes_id() {
        let manifest1 = json!({ "quality_of_product": 0.95 });
        let manifest2 = json!({ "quality_of_product": 0.96 });

        let id1 = DuUuid::generate(&manifest1, None).unwrap();
        let id2 = DuUuid::generate(&manifest2, None).unwrap();

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_encryption_changes_id() {
        let manifest = json!({ "quality_of_product": 0.95 });
        let key1 = [1u8; 32];
        let key2 = [2u8; 32];

        let id1 = DuUuid::generate(&manifest, Some(&key1)).unwrap();
        let id2 = DuUuid::generate(&manifest, Some(&key2)).unwrap();

        assert_ne!(id1, id2);
    }
}
