use std::sync::Arc;

use crate::{
    modules::accounts::{
        domain::{SessionToken, SessionTokenHash, UserId},
        ports::SessionRepository,
    },
    shared::{application::InternalError, auth::TokenHasher},
};

pub struct LogoutUserCommand {
    pub user_id: UserId,
    pub token: SessionToken,
}

pub struct LogoutUserService {
    session_repository: Arc<dyn SessionRepository>,
    hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
}

impl LogoutUserService {
    pub fn new(
        session_repository: Arc<dyn SessionRepository>,
        hasher: Arc<dyn TokenHasher<SessionToken, SessionTokenHash>>,
    ) -> Self {
        Self {
            session_repository,
            hasher,
        }
    }

    pub async fn execute(&self, command: LogoutUserCommand) -> Result<(), LogoutUserError> {
        let token_hash = self.hasher.hash(&command.token);

        self.session_repository
            .delete_by_token_hash(&token_hash)
            .await
            .map_err(|error| {
                tracing::error!(
                    error = ?error,
                    user_id = %command.user_id,
                    "Failed to delete session"
                );
                LogoutUserError::Internal(InternalError::Failed)
            })?;

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LogoutUserError {
    #[error(transparent)]
    Internal(#[from] InternalError),
}
