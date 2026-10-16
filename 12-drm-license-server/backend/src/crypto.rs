use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::DrmError;

type HmacSha256 = Hmac<Sha256>;

pub fn b64url_encode(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn b64url_decode(text: &str) -> Result<Vec<u8>, DrmError> {
    URL_SAFE_NO_PAD
        .decode(text)
        .map_err(|_| DrmError::Malformed("invalid base64url".into()))
}

pub fn hex_decode(text: &str) -> Result<Vec<u8>, DrmError> {
    if !text.is_ascii() || !text.len().is_multiple_of(2) {
        return Err(DrmError::Malformed("odd hex length".into()));
    }
    (0..text.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&text[i..i + 2], 16)
                .map_err(|_| DrmError::Malformed("invalid hex".into()))
        })
        .collect()
}

pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn mac(secret: &[u8]) -> Result<HmacSha256, DrmError> {
    HmacSha256::new_from_slice(secret).map_err(|_| DrmError::Config("invalid hmac secret".into()))
}

pub fn sign(secret: &[u8], message: &[u8]) -> Result<Vec<u8>, DrmError> {
    let mut m = mac(secret)?;
    m.update(message);
    Ok(m.finalize().into_bytes().to_vec())
}

pub fn verify(secret: &[u8], message: &[u8], signature: &[u8]) -> Result<(), DrmError> {
    let mut m = mac(secret)?;
    m.update(message);
    m.verify_slice(signature).map_err(|_| DrmError::BadSignature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_roundtrip_keeps_key_bytes_exact() {
        let key = hex_decode("0123456789abcdef0123456789abcdef").unwrap();
        assert_eq!(key.len(), 16);
        assert_eq!(hex_encode(&key), "0123456789abcdef0123456789abcdef");
    }

    #[test]
    fn base64url_has_no_padding_or_unsafe_chars_as_jwk_requires() {
        let text = b64url_encode(&[0xfb, 0xff, 0xfe, 0x01]);
        assert!(!text.contains('=') && !text.contains('+') && !text.contains('/'));
        assert_eq!(b64url_decode(&text).unwrap(), vec![0xfb, 0xff, 0xfe, 0x01]);
    }

    #[test]
    fn hmac_rejects_signature_from_another_secret() {
        let sig = sign(b"secret-a", b"payload").unwrap();
        assert!(verify(b"secret-a", b"payload", &sig).is_ok());
        assert!(matches!(verify(b"secret-b", b"payload", &sig), Err(DrmError::BadSignature)));
    }
}
