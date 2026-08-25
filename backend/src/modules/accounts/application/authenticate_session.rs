use std::sync::Arc;

use chrono::Utc;

use crate::{
    modules::accounts::{
        domain::{SessionToken, SessionTokenHash, UserId},
        ports::SessionRepository,
    },
    shared::{application::InternalError, auth::TokenHasher},
};

pub struct AuthenticateSessionCommand {
    pub token: SessionToken,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthenticatedUser {
    pub user_id: UserId,
}

pub struct AuthenticateSessionService {
    session_repository: Arc<dyn SessionRepository>,
    token_hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
}

impl AuthenticateSessionService {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        token_hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
    ) -> Self {
        Self {
            session_repository,
            token_hasher,
        }
    }

    pub async fn execute(
        &self,
        command: AuthenticateSessionCommand,
    ) -> Result<AuthenticatedUser, AuthenticateSessionError> {
        let token_hash = self.token_hasher.hash(&command.token);

        let session = self
            .session_repository
            .find_by_token_hash(&token_hash)
            .await
            .map_err(|error| {
                tracing::error!(error=?error, "Failed to load session during authentication");
                AuthenticateSessionError::Internal(InternalError::Failed)
            })?
            .ok_or(AuthenticateSessionError::InvalidSession)?;

        if session.is_expired_at(Utc::now()) {
            return Err(AuthenticateSessionError::SessionExpired);
        }

        Ok(AuthenticatedUser {
            user_id: session.user_id(),
        })
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum AuthenticateSessionError {
    #[error("Session is invalid")]
    InvalidSession,
    #[error("Session has expired")]
    SessionExpired,
    #[error(transparent)]
    Internal(#[from] InternalError),
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::{
        modules::accounts::domain::{Session, SessionId},
        test_helpers::build_auth_service,
    };

    use super::*;

    #[tokio::test]
    async fn valid_session_returns_authenticated_user() {
        let (service, repository, hasher) = build_auth_service();

        let user_id = UserId::new();
        let token = SessionToken::from_string("this-is-a-session-token".to_owned())
            .expect("Test session token should be valid");
        let token_hash = hasher.hash(&token);

        let created_at = Utc::now();
        let expires_at = created_at + Duration::hours(1);

        let session = Session::new(
            SessionId::new(),
            user_id,
            token_hash,
            expires_at,
            created_at,
        )
        .expect("Test session should be valid");

        repository
            .insert(&session)
            .await
            .expect("Test session should be insertable");

        let result = service
            .execute(AuthenticateSessionCommand { token })
            .await
            .expect("Session authentication should succeed");

        assert_eq!(result.user_id, user_id)
    }

    #[tokio::test]
    async fn unknown_token_returns_invalid_session() {
        let (service, _, _) = build_auth_service();

        let token = SessionToken::from_string("unknown-token".to_owned())
            .expect("Test session token should be valid");

        let result = service.execute(AuthenticateSessionCommand { token }).await;

        assert_eq!(result, Err(AuthenticateSessionError::InvalidSession))
    }

    #[tokio::test]
    async fn expired_session_returns_session_expired() {
        let (service, repository, hasher) = build_auth_service();

        let user_id = UserId::new();
        let token = SessionToken::from_string("this-is-a-session-token".to_owned())
            .expect("Test session token should be valid");
        let token_hash = hasher.hash(&token);

        let created_at = Utc::now() - Duration::hours(2);
        let expires_at = Utc::now() - Duration::hours(1);

        let session = Session::new(
            SessionId::new(),
            user_id,
            token_hash,
            expires_at,
            created_at,
        )
        .expect("Test session should be valid");

        repository
            .insert(&session)
            .await
            .expect("Test session should be insertable");

        let result = service.execute(AuthenticateSessionCommand { token }).await;

        assert_eq!(result, Err(AuthenticateSessionError::SessionExpired))
    }
}
