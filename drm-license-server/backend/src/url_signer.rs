use crate::crypto::{hex_decode, hex_encode, sign, verify};
use crate::error::DrmError;

pub const PREFIX: &str = "/media";

pub struct UrlSigner {
    secret: Vec<u8>,
}

impl UrlSigner {
    pub fn new(secret: Vec<u8>) -> Self {
        Self { secret }
    }

    fn message(content: &str, expires: u64) -> String {
        format!("acl={PREFIX}/{content}/*~exp={expires}")
    }

    pub fn signature(&self, content: &str, expires: u64) -> Result<String, DrmError> {
        Ok(hex_encode(&sign(&self.secret, Self::message(content, expires).as_bytes())?))
    }

    pub fn signed_path(&self, content: &str, file: &str, expires: u64) -> Result<String, DrmError> {
        let sig = self.signature(content, expires)?;
        Ok(format!("{PREFIX}/{expires}/{sig}/{content}/{file}"))
    }

    pub fn verify(&self, content: &str, expires: u64, sig: &str, now: u64) -> Result<(), DrmError> {
        let bytes = hex_decode(sig).map_err(|_| DrmError::BadSignature)?;
        verify(&self.secret, Self::message(content, expires).as_bytes(), &bytes)?;
        if expires <= now {
            return Err(DrmError::Expired);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signer() -> UrlSigner {
        UrlSigner::new(b"url-secret".to_vec())
    }

    #[test]
    fn signed_path_embeds_token_before_content_so_relative_segment_urls_inherit_it() {
        let path = signer().signed_path("clip", "stream.mpd", 5_000).unwrap();
        let sig = signer().signature("clip", 5_000).unwrap();
        assert_eq!(path, format!("/media/5000/{sig}/clip/stream.mpd"));
    }

    #[test]
    fn valid_signature_before_expiry_is_accepted() {
        let sig = signer().signature("clip", 5_000).unwrap();
        assert!(signer().verify("clip", 5_000, &sig, 4_999).is_ok());
    }

    #[test]
    fn link_stops_working_after_expiry_so_shared_urls_go_stale() {
        let sig = signer().signature("clip", 5_000).unwrap();
        assert!(matches!(signer().verify("clip", 5_000, &sig, 5_000), Err(DrmError::Expired)));
    }

    #[test]
    fn extending_expiry_without_resigning_is_detected() {
        let sig = signer().signature("clip", 5_000).unwrap();
        assert!(matches!(signer().verify("clip", 9_999, &sig, 1), Err(DrmError::BadSignature)));
    }

    #[test]
    fn token_for_one_title_does_not_unlock_another_path() {
        let sig = signer().signature("clip", 5_000).unwrap();
        assert!(matches!(signer().verify("premium", 5_000, &sig, 1), Err(DrmError::BadSignature)));
    }

    #[test]
    fn flipped_signature_char_or_non_hex_is_rejected() {
        let mut sig = signer().signature("clip", 5_000).unwrap();
        let first = if sig.starts_with('0') { "1" } else { "0" };
        sig.replace_range(0..1, first);
        assert!(matches!(signer().verify("clip", 5_000, &sig, 1), Err(DrmError::BadSignature)));
        assert!(matches!(signer().verify("clip", 5_000, "zz", 1), Err(DrmError::BadSignature)));
    }
}
