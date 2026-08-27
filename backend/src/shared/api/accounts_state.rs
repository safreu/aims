use std::sync::Arc;

use crate::{
    config::SessionCookieConfig,
    modules::accounts::application::{
        AuthenticateSessionService, CreateSessionService, GetUserService, LoginUserService,
        RegisterUserService,
    },
};

#[derive(Clone)]
pub struct AccountsState {
    pub register_user: Arc<RegisterUserService>,
    pub login_user: Arc<LoginUserService>,
    pub create_session: Arc<CreateSessionService>,
    pub authenticate_session: Arc<AuthenticateSessionService>,
    pub session_cookie: SessionCookieConfig,
    pub get_user: Arc<GetUserService>,
}
