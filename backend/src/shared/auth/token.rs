pub trait TokenValue: Sized {
    type Error;

    fn from_string(value: String) -> Result<Self, Self::Error>;
    fn as_str(&self) -> &str;
}

pub trait TokenHashValue: Sized {
    type Error: std::fmt::Debug;

    fn from_encoded(value: &str) -> Result<Self, Self::Error>;
    fn as_str(&self) -> &str;
}

/// Generates cryptographically secure tokens.
///
/// Generated tokens are intended to be sent to clients as opaque
/// authenticated credentials.
pub trait TokenGenerator<T>: Send + Sync
where
    T: TokenValue,
{
    /// Generates a new token.
    fn generate(&self) -> Result<T, TokenGeneratorError>;
}

/// Errors returned while generating session tokens.
#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum TokenGeneratorError {
    #[error("Failed to generate token")]
    GenerationFailed,
}

/// Produces the persistent representation of tokens.
///
/// Implementations must be deterministic so the same token
/// always produces the same hash
pub trait TokenHasher<T, H>: Send + Sync
where
    T: TokenValue,
    H: TokenHashValue,
{
    /// Computes the hash of a token.
    fn hash(&self, token: &T) -> H;
}
