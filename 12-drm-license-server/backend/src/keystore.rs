use std::collections::HashMap;

use crate::crypto::hex_decode;
use crate::error::DrmError;

#[derive(Debug, Clone, PartialEq)]
pub struct ContentKey {
    pub content: String,
    pub kid: Vec<u8>,
    pub key: Vec<u8>,
}

pub trait KeyStore: Send + Sync {
    fn find(&self, kid: &[u8]) -> Option<ContentKey>;
}

pub struct MemoryKeyStore {
    keys: HashMap<Vec<u8>, ContentKey>,
}

impl MemoryKeyStore {
    pub fn from_hex(entries: &[(String, String, String)]) -> Result<Self, DrmError> {
        let mut keys = HashMap::new();
        for (content, kid, key) in entries {
            let kid = hex_decode(kid)?;
            let key = hex_decode(key)?;
            if kid.len() != 16 || key.len() != 16 {
                return Err(DrmError::Config(format!("{content}: kid and key must be 16 bytes")));
            }
            keys.insert(kid.clone(), ContentKey { content: content.clone(), kid, key });
        }
        Ok(Self { keys })
    }
}

impl KeyStore for MemoryKeyStore {
    fn find(&self, kid: &[u8]) -> Option<ContentKey> {
        self.keys.get(kid).cloned()
    }
}
