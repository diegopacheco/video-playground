use serde::{Deserialize, Serialize};

use crate::crypto::{b64url_decode, b64url_encode, sign, verify};
use crate::error::DrmError;

const HEADER: &str = r#"{"alg":"HS256","typ":"JWT"}"#;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub ent: Vec<String>,
    pub exp: u64,
}

impl Claims {
    pub fn entitled_to(&self, content: &str) -> bool {
        self.ent.iter().any(|c| c == content)
    }
}

pub struct SessionSigner {
    secret: Vec<u8>,
}

impl SessionSigner {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    pub fn issue(&self, claims: &Claims) -> Result<String, DrmError> {
        let payload = serde_json::to_vec(claims).map_err(|e| DrmError::Malformed(e.to_string()))?;
        let signing_input = format!("{}.{}", b64url_encode(HEADER.as_bytes()), b64url_encode(&payload));
        let signature = sign(&self.secret, signing_input.as_bytes())?;
        Ok(format!("{signing_input}.{}", b64url_encode(&signature)))
    }

    pub fn verify(&self, token: &str, now: u64) -> Result<Claims, DrmError> {
        let parts: Vec<&str> = token.split('.').collect();
        let [header, payload, signature] = parts.as_slice() else {
            return Err(DrmError::Malformed("token must have 3 parts".into()));
        };
        let signing_input = format!("{header}.{payload}");
        verify(&self.secret, signing_input.as_bytes(), &b64url_decode(signature)?)?;
        if b64url_decode(header)? != HEADER.as_bytes() {
            return Err(DrmError::Malformed("unsupported header".into()));
        }
        let claims: Claims = serde_json::from_slice(&b64url_decode(payload)?)
            .map_err(|e| DrmError::Malformed(e.to_string()))?;
        if claims.exp <= now {
            return Err(DrmError::Expired);
        }
        Ok(claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer() -> SessionSigner {
        SessionSigner::new(b"session-secret".to_vec())
    }

    fn claims(exp: u64) -> Claims {
        Claims { sub: "alice".into(), ent: vec!["clip".into()], exp }
    }

    #[test]
    fn valid_token_returns_the_entitlements_the_license_server_will_trust() {
        let token = signer().issue(&claims(2_000)).unwrap();
        let back = signer().verify(&token, 1_000).unwrap();
        assert!(back.entitled_to("clip"));
        assert!(!back.entitled_to("other"));
    }

    #[test]
    fn expired_session_cannot_obtain_licenses() {
        let token = signer().issue(&claims(1_000)).unwrap();
        assert!(matches!(signer().verify(&token, 1_000), Err(DrmError::Expired)));
    }

    #[test]
    fn user_cannot_grant_himself_entitlements_by_editing_the_payload() {
        let token = signer().issue(&Claims { sub: "bob".into(), ent: vec![], exp: 2_000 }).unwrap();
        let parts: Vec<&str> = token.split('.').collect();
        let forged = b64url_encode(&serde_json::to_vec(&claims(2_000)).unwrap());
        let tampered = format!("{}.{}.{}", parts[0], forged, parts[2]);
        assert!(matches!(signer().verify(&tampered, 1_000), Err(DrmError::BadSignature)));
    }

    #[test]
    fn token_signed_with_another_secret_is_rejected() {
        let token = SessionSigner::new(b"attacker".to_vec()).issue(&claims(2_000)).unwrap();
        assert!(matches!(signer().verify(&token, 1_000), Err(DrmError::BadSignature)));
    }

    #[test]
    fn garbage_is_malformed_not_a_panic() {
        assert!(matches!(signer().verify("abc", 1), Err(DrmError::Malformed(_))));
        assert!(matches!(signer().verify("a.b.!!", 1), Err(DrmError::Malformed(_))));
    }
}
