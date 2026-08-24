use std::sync::Arc;

use sqlx::PgPool;

use crate::{
    config::{SessionConfig, SessionCookieConfig},
    modules::accounts::{
        adapters::{Argon2PasswordHasher, PostgresSessionRepository, PostgresUserRepository},
        application::{
            AuthenticateSessionService, CreateSessionService, LoginUserService, RegisterUserService,
        },
    },
    shared::{
        api::AccountsState,
        auth::{SecureTokenGenerator, Sha256TokenHasher},
    },
};

pub(super) fn build_accounts_state(pool: &PgPool, config: &SessionConfig) -> AccountsState {
    let user_repository = Arc::new(PostgresUserRepository::new(pool.clone()));

    let session_repository = Arc::new(PostgresSessionRepository::new(pool.clone()));

    let password_hasher = Arc::new(Argon2PasswordHasher::new());

    let session_token_generator = Arc::new(SecureTokenGenerator);

    let session_token_hasher = Arc::new(Sha256TokenHasher);

    let register_user_service = Arc::new(RegisterUserService::new(
        user_repository.clone(),
        password_hasher.clone(),
    ));

    let login_user_service = Arc::new(LoginUserService::new(user_repository, password_hasher));

    let session_lifetime = chrono::Duration::days(config.lifetime_days);

    let create_session_service = Arc::new(CreateSessionService::new(
        session_repository.clone(),
        session_token_generator,
        session_token_hasher.clone(),
        session_lifetime,
    ));

    let authenticate_session_service = Arc::new(AuthenticateSessionService::new(
        session_repository,
        session_token_hasher,
    ));

    AccountsState {
        register_user: register_user_service,
        login_user: login_user_service,
        create_session: create_session_service,
        authenticate_session: authenticate_session_service,
        session_cookie: SessionCookieConfig {
            name: config.cookie_name.clone(),
            secure: config.cookie_secure,
        },
    }
}
