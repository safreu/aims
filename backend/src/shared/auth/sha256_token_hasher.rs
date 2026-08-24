use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use sha2::{Digest, Sha256};

use crate::shared::auth::{TokenHashValue, TokenHasher, TokenValue};

pub struct Sha256TokenHasher;

impl Sha256TokenHasher {
    pub fn new() -> Self {
        Self
    }
}

impl<T, H> TokenHasher<T, H> for Sha256TokenHasher
where
    T: TokenValue,
    H: TokenHashValue,
{
    fn hash(&self, token: &T) -> H {
        let digest = Sha256::digest(token.as_str().as_bytes());
        let encoded = URL_SAFE_NO_PAD.encode(digest);

        H::from_encoded(&encoded).expect("SHA-256 output encoding must be valid token hash")
    }
}

impl Default for Sha256TokenHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::accounts::domain::{SessionToken, SessionTokenHash};

    use super::*;

    fn token(value: &str) -> SessionToken {
        SessionToken::from_string(value.to_owned()).expect("Test session token should be valid")
    }

    #[test]
    fn same_token_produces_same_hash() {
        let hasher = Sha256TokenHasher;

        let token = token("this-is-a-session-token");

        let first: SessionTokenHash = hasher.hash(&token);
        let second = hasher.hash(&token);

        assert_eq!(first, second)
    }

    #[test]
    fn different_tokens_produces_different_hashes() {
        let hasher = Sha256TokenHasher;

        let first: SessionTokenHash = hasher.hash(&token("this-is-a-session-token"));
        let second = hasher.hash(&token("this-is-another-session-token"));

        assert_ne!(first, second)
    }

    #[test]
    fn hash_does_not_contain_raw_token() {
        let hasher = Sha256TokenHasher;

        let token = token("this-is-a-session-token");

        let hash: SessionTokenHash = hasher.hash(&token);

        assert_ne!(hash.as_str(), token.as_str())
    }
}
