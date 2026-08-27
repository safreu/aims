use std::sync::Arc;

use crate::{
    modules::accounts::{
        domain::{User, UserId},
        ports::UserRepository,
    },
    shared::application::InternalError,
};

pub struct GetUserCommand {
    pub user_id: UserId,
}

pub struct GetUserService {
    user_repository: Arc<dyn UserRepository>,
}

impl GetUserService {
    pub fn new(user_repository: Arc<dyn UserRepository>) -> Self {
        Self { user_repository }
    }

    pub async fn execute(&self, command: GetUserCommand) -> Result<User, GetUserError> {
        let user = self
            .user_repository
            .find_by_id(&command.user_id)
            .await
            .map_err(|error| {
                tracing::error!(error = ?error, "Failed to load user");
                GetUserError::Internal(InternalError::Failed)
            })?
            .ok_or(GetUserError::UserNotFound)?;

        Ok(user)
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum GetUserError {
    #[error("The user was not found")]
    UserNotFound,

    #[error(transparent)]
    Internal(#[from] InternalError),
}
