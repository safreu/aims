use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::{
    modules::accounts::{
        domain::{Session, SessionId, SessionToken, SessionTokenHash, UserId},
        ports::SessionRepository,
    },
    shared::{
        application::InternalError,
        auth::{TokenGenerator, TokenHasher},
    },
};

pub struct CreateSessionCommand {
    pub user_id: UserId,
}

pub struct CreateSessionResult {
    pub token: SessionToken,
    pub expires_at: DateTime<Utc>,
}

pub struct CreateSessionService {
    session_repository: Arc<dyn SessionRepository>,
    token_generator: Arc<dyn TokenGenerator<SessionToken>>,
    token_hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
    session_lifetime: Duration,
}

impl CreateSessionService {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        token_generator: Arc<dyn TokenGenerator<SessionToken>>,
        token_hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
        session_lifetime: Duration,
    ) -> Self {
        Self {
            session_repository,
            token_generator,
            token_hasher,
            session_lifetime,
        }
    }

    pub async fn execute(
        &self,
        command: CreateSessionCommand,
    ) -> Result<CreateSessionResult, InternalError> {
        let token = self.token_generator.generate().map_err(|error| {
            tracing::error!(error=?error, user_id=%command.user_id, "failed to generate session token");
            InternalError::Failed

        })?;

        let token_hash = self.token_hasher.hash(&token);

        let created_at = Utc::now();
        let expires_at = created_at + self.session_lifetime;

        let session = Session::new(
            SessionId::new(),
            command.user_id,
            token_hash,
            expires_at,
            created_at,
        )
        .map_err(|error| {
            tracing::error!(error=?error, user_id=%command.user_id, "failed to construct session");
            InternalError::Failed
        })?;

        self.session_repository.insert(&session).await.map_err(|error| {
            tracing::error!(error=?error, user_id=%command.user_id, session_id=%session.id(), "failed to persist session");
            InternalError::Failed
        })?;

        Ok(CreateSessionResult { token, expires_at })
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        modules::accounts::adapters::InMemorySessionRepository,
        shared::auth::Sha256TokenHasher,
        test_helpers::{FailingSessionTokenGenerator, build_create_session_service},
    };

    use super::*;

    #[tokio::test]
    async fn session_is_created_for_user() {
        let (service, repository, hasher) = build_create_session_service();
        let user_id = UserId::new();

        let result = service
            .execute(CreateSessionCommand { user_id })
            .await
            .expect("Session creation should succeed");

        let token_hash = hasher.hash(&result.token);

        let stored_session = repository
            .find_by_token_hash(&token_hash)
            .await
            .expect("Session lookup should succeed")
            .expect("Created session should be stored");

        assert_eq!(stored_session.user_id(), user_id)
    }

    #[tokio::test]
    async fn returned_token_matches_stored_hash() {
        let (service, repository, hasher) = build_create_session_service();
        let user_id = UserId::new();

        let result = service
            .execute(CreateSessionCommand { user_id })
            .await
            .expect("Session creation should succeed");

        let expected_hash = hasher.hash(&result.token);

        let stored_session = repository
            .find_by_token_hash(&expected_hash)
            .await
            .expect("Session lookup should succeed")
            .expect("Created session should be stored");

        assert_eq!(stored_session.token_hash(), &expected_hash);
        assert_eq!(result.token.as_str(), "this-session-token-is-fixed")
    }

    #[tokio::test]
    async fn expiration_matches_configured_lifetime() {
        let (service, _, _) = build_create_session_service();
        let user_id = UserId::new();

        let before = Utc::now();

        let result = service
            .execute(CreateSessionCommand { user_id })
            .await
            .expect("Session creation should succeed");

        let after = Utc::now();

        assert!(result.expires_at >= before + Duration::hours(1));
        assert!(result.expires_at <= after + Duration::hours(1))
    }

    #[tokio::test]
    async fn token_generation_failure_is_reported() {
        let repository = Arc::new(InMemorySessionRepository::new());
        let generator = Arc::new(FailingSessionTokenGenerator);
        let hasher = Arc::new(Sha256TokenHasher::new());

        let service = CreateSessionService::new(
            repository.clone(),
            generator,
            hasher.clone(),
            Duration::hours(1),
        );

        let result = service
            .execute(CreateSessionCommand {
                user_id: UserId::new(),
            })
            .await;

        assert!(matches!(result, Err(InternalError::Failed)));
    }
}
