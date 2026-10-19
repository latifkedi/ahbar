use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Generates a deterministic, stateless HMAC-SHA256 signature for passwordless
/// Magic Link preference management and one-click unsubscribe URLs.
pub fn sign_subscriber_token(email: &str, secret: &str) -> String {
    let normalized = email.trim().to_ascii_lowercase();
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC accepts keys of any byte size");
    mac.update(b"ahbar:v1:");
    mac.update(normalized.as_bytes());
    let result = mac.finalize().into_bytes();
    hex::encode(&result[..16]) // 128-bit hex token (32 chars) for clean URLs
}

/// Verifies a subscriber's magic-link token in constant time.
pub fn verify_subscriber_token(email: &str, token: &str, secret: &str) -> bool {
    let expected = sign_subscriber_token(email, secret);
    constant_time_eq(expected.as_bytes(), token.trim().as_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (&x, &y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_link_token_roundtrip() {
        let secret = "test-secret-key-2026";
        let email = "Researcher@Example.org ";
        let token = sign_subscriber_token(email, secret);
        assert_eq!(token.len(), 32);
        assert!(verify_subscriber_token("researcher@example.org", &token, secret));
        assert!(!verify_subscriber_token("other@example.org", &token, secret));
        assert!(!verify_subscriber_token("researcher@example.org", "deadbeef", secret));
    }
}
