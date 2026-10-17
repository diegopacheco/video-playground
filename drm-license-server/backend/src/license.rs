use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::crypto::{b64url_decode, b64url_encode};
use crate::error::DrmError;
use crate::keystore::KeyStore;
use crate::session::Claims;

pub struct LicenseGrant {
    pub body: Vec<u8>,
    pub content_type: &'static str,
    pub contents: Vec<String>,
}

pub trait LicenseService: Send + Sync {
    fn key_system(&self) -> &'static str;
    fn issue(&self, request: &[u8], claims: &Claims) -> Result<LicenseGrant, DrmError>;
}

#[derive(Debug, Deserialize)]
pub struct ClearKeyRequest {
    pub kids: Vec<String>,
    #[serde(rename = "type", default = "temporary")]
    pub session_type: String,
}

fn temporary() -> String {
    "temporary".into()
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Jwk {
    pub kty: String,
    pub kid: String,
    pub k: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ClearKeyResponse {
    pub keys: Vec<Jwk>,
    #[serde(rename = "type")]
    pub session_type: String,
}

pub struct ClearKeyService {
    store: Arc<dyn KeyStore>,
}

impl ClearKeyService {
    pub fn new(store: Arc<dyn KeyStore>) -> Self {
        Self { store }
    }
}

impl LicenseService for ClearKeyService {
    fn key_system(&self) -> &'static str {
        "org.w3.clearkey"
    }

    fn issue(&self, request: &[u8], claims: &Claims) -> Result<LicenseGrant, DrmError> {
        let req: ClearKeyRequest =
            serde_json::from_slice(request).map_err(|e| DrmError::Malformed(e.to_string()))?;
        if req.kids.is_empty() {
            return Err(DrmError::Malformed("kids must not be empty".into()));
        }
        if req.session_type != "temporary" {
            return Err(DrmError::Malformed("only temporary sessions are supported".into()));
        }
        let mut keys = Vec::new();
        let mut contents = Vec::new();
        for kid_text in &req.kids {
            let kid = b64url_decode(kid_text)?;
            let found = self
                .store
                .find(&kid)
                .ok_or_else(|| DrmError::UnknownKey(kid_text.clone()))?;
            if !claims.entitled_to(&found.content) {
                return Err(DrmError::NotEntitled(found.content));
            }
            keys.push(Jwk { kty: "oct".into(), kid: b64url_encode(&found.kid), k: b64url_encode(&found.key) });
            contents.push(found.content);
        }
        let response = ClearKeyResponse { keys, session_type: req.session_type };
        let body = serde_json::to_vec(&response).map_err(|e| DrmError::Malformed(e.to_string()))?;
        Ok(LicenseGrant { body, content_type: "application/json", contents })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keystore::MemoryKeyStore;

    const KID: &str = "a1b2c3d4e5f60718293a4b5c6d7e8f90";
    const KEY: &str = "0123456789abcdef0123456789abcdef";

    fn service() -> ClearKeyService {
        let store = MemoryKeyStore::from_hex(&[("clip".into(), KID.into(), KEY.into())]).unwrap();
        ClearKeyService::new(Arc::new(store))
    }

    fn claims(ent: &[&str]) -> Claims {
        Claims { sub: "alice".into(), ent: ent.iter().map(|s| s.to_string()).collect(), exp: u64::MAX }
    }

    fn kid_b64() -> String {
        b64url_encode(&crate::crypto::hex_decode(KID).unwrap())
    }

    #[test]
    fn response_is_a_w3c_clear_key_jwk_set_the_cdm_can_load() {
        let request = format!(r#"{{"kids":["{}"],"type":"temporary"}}"#, kid_b64());
        let grant = service().issue(request.as_bytes(), &claims(&["clip"])).unwrap();
        let json: serde_json::Value = serde_json::from_slice(&grant.body).unwrap();
        assert_eq!(json["type"], "temporary");
        let key = &json["keys"][0];
        assert_eq!(key["kty"], "oct");
        assert_eq!(key["kid"], kid_b64());
        let k = key["k"].as_str().unwrap();
        assert!(!k.contains('=') && !k.contains('+') && !k.contains('/'));
        assert_eq!(crate::crypto::hex_encode(&b64url_decode(k).unwrap()), KEY);
        assert_eq!(grant.content_type, "application/json");
    }

    #[test]
    fn user_without_entitlement_gets_no_key_material() {
        let request = format!(r#"{{"kids":["{}"]}}"#, kid_b64());
        let result = service().issue(request.as_bytes(), &claims(&[]));
        assert!(matches!(result, Err(DrmError::NotEntitled(c)) if c == "clip"));
    }

    #[test]
    fn unknown_kid_is_rejected_instead_of_returning_partial_licenses() {
        let other = b64url_encode(&[7u8; 16]);
        let request = format!(r#"{{"kids":["{}","{}"]}}"#, kid_b64(), other);
        assert!(matches!(service().issue(request.as_bytes(), &claims(&["clip"])), Err(DrmError::UnknownKey(_))));
    }

    #[test]
    fn malformed_or_empty_requests_are_client_errors() {
        assert!(matches!(service().issue(b"not json", &claims(&["clip"])), Err(DrmError::Malformed(_))));
        assert!(matches!(service().issue(br#"{"kids":[]}"#, &claims(&["clip"])), Err(DrmError::Malformed(_))));
        assert!(matches!(
            service().issue(br#"{"kids":["AAAA"],"type":"persistent-license"}"#, &claims(&["clip"])),
            Err(DrmError::Malformed(_))
        ));
    }

    #[test]
    fn key_system_identifier_matches_eme_name() {
        assert_eq!(service().key_system(), "org.w3.clearkey");
    }
}
