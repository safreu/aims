use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use rand::{TryRngCore, rngs::OsRng};

use crate::shared::auth::{TokenGenerator, TokenGeneratorError, TokenValue};
pub struct SecureTokenGenerator;

impl SecureTokenGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl<T> TokenGenerator<T> for SecureTokenGenerator
where
    T: TokenValue,
{
    fn generate(&self) -> Result<T, TokenGeneratorError> {
        let mut bytes = [0_u8; 32];

        OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| TokenGeneratorError::GenerationFailed)?;

        let encoded = URL_SAFE_NO_PAD.encode(bytes);

        T::from_string(encoded).map_err(|_| TokenGeneratorError::GenerationFailed)
    }
}

impl Default for SecureTokenGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::modules::accounts::domain::SessionToken;

    use super::*;

    #[test]
    fn generated_token_is_not_empty() {
        let generator = SecureTokenGenerator;

        let token: SessionToken = generator
            .generate()
            .expect("Token generation should succeed");

        assert!(!token.as_str().is_empty())
    }

    #[test]
    fn generated_tokens_are_different() {
        let generator = SecureTokenGenerator;

        let first: SessionToken = generator
            .generate()
            .expect("First token generation should succeed");

        let second = generator
            .generate()
            .expect("Second token generation should succeed");

        assert_ne!(first, second)
    }
}
