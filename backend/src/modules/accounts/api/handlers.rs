use axum::{Json, extract::State, http::StatusCode};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};

use crate::{
    modules::accounts::{
        api::{
            CurrentUser, LoginUserRequest, LoginUserResponse, RegisterUserRequest,
            RegisterUserResponse, dto::GetUserResponse,
        },
        application::{
            CreateSessionCommand, GetUserCommand, LoginUserCommand, LogoutUserCommand,
            RegisterUserCommand,
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<RegisterUserRequest>,
) -> Result<(StatusCode, Json<RegisterUserResponse>), ApiError> {
    let command = RegisterUserCommand {
        email: request.email,
        display_name: request.display_name,
        password: request.password,
    };

    let user_id = state
        .accounts
        .register_user
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = RegisterUserResponse {
        id: user_id.to_string(),
    };

    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn login_user(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginUserRequest>,
) -> Result<(StatusCode, CookieJar, Json<LoginUserResponse>), ApiError> {
    let command = LoginUserCommand {
        email: request.email,
        password: request.password,
    };

    let user_id = state.accounts.login_user.execute(command).await?;

    let session = state
        .accounts
        .create_session
        .execute(CreateSessionCommand { user_id })
        .await?;

    let cookie = Cookie::build((
        state.accounts.session_cookie.name.clone(),
        session.token.into_string(),
    ))
    .path("/")
    .http_only(true)
    .same_site(SameSite::Lax)
    .secure(state.accounts.session_cookie.secure)
    .build();

    let jar = jar.add(cookie);

    Ok((
        StatusCode::OK,
        jar,
        Json(LoginUserResponse {
            id: user_id.to_string(),
        }),
    ))
}

pub async fn get_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<GetUserResponse>, ApiError> {
    let command = GetUserCommand {
        user_id: current_user.user_id(),
    };

    let user = state
        .accounts
        .get_user
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(GetUserResponse {
        id: user.id().to_string(),
        display_name: user.display_name().as_str().to_owned(),
        email: user.email().to_string(),
    }))
}

pub async fn logout_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
    jar: CookieJar,
) -> Result<(CookieJar, StatusCode), ApiError> {
    let command = LogoutUserCommand {
        user_id: current_user.user_id(),
        token: current_user.token().clone(),
    };

    state
        .accounts
        .logout_user
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let jar = jar.remove(state.accounts.session_cookie.name.clone());

    Ok((jar, StatusCode::NO_CONTENT))
}
