use crate::{
    modules::accounts::application::{
        AuthenticateSessionError, GetUserError, LoginUserError, LogoutUserError, RegisterUserError,
    },
    shared::{api::error::ApiError, application::InternalError},
};

impl From<RegisterUserError> for ApiError {
    fn from(error: RegisterUserError) -> Self {
        match error {
            RegisterUserError::EmailAlreadyExists => Self::conflict(
                "email_already_exists",
                "A user with this email already exists",
            ),
            RegisterUserError::InvalidEmail => {
                Self::bad_request("invalid_email", "The email address is invalid")
            }
            RegisterUserError::InvalidDisplayName => {
                Self::bad_request("invalid_display_name", "The display name is invalid")
            }
            RegisterUserError::Internal(_) => Self::internal_error(),
        }
    }
}

impl From<LoginUserError> for ApiError {
    fn from(error: LoginUserError) -> Self {
        match error {
            LoginUserError::InvalidCredentials => Self::unauthorized(
                "invalid_credentials",
                "The supplied credentials are invalid",
            ),
            LoginUserError::Internal(_) => Self::internal_error(),
        }
    }
}

impl From<AuthenticateSessionError> for ApiError {
    fn from(error: AuthenticateSessionError) -> Self {
        match error {
            AuthenticateSessionError::InvalidSession | AuthenticateSessionError::SessionExpired => {
                Self::unauthorized("authentication_required", "Authentication is required")
            }
            AuthenticateSessionError::Internal(_) => Self::internal_error(),
        }
    }
}

impl From<GetUserError> for ApiError {
    fn from(value: GetUserError) -> Self {
        match value {
            GetUserError::UserNotFound => {
                ApiError::not_found("user_not_found", "The user was not found")
            }
            GetUserError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<LogoutUserError> for ApiError {
    fn from(value: LogoutUserError) -> Self {
        match value {
            LogoutUserError::Internal(_) => ApiError::internal_error(),
        }
    }
}
impl From<InternalError> for ApiError {
    fn from(_error: InternalError) -> Self {
        Self::internal_error()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use super::*;

    #[test]
    fn invalid_email_maps_to_bad_request() {
        let error = ApiError::from(RegisterUserError::InvalidEmail);

        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.code(), "invalid_email")
    }

    #[test]
    fn invalid_display_name_maps_to_bad_request() {
        let error = ApiError::from(RegisterUserError::InvalidDisplayName);

        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.code(), "invalid_display_name")
    }

    #[test]
    fn existing_email_maps_to_conflict() {
        let error = ApiError::from(RegisterUserError::EmailAlreadyExists);

        assert_eq!(error.status(), StatusCode::CONFLICT);
        assert_eq!(error.code(), "email_already_exists")
    }

    #[test]
    fn invalid_credentials_maps_to_unauthorized() {
        let error = ApiError::from(LoginUserError::InvalidCredentials);

        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.code(), "invalid_credentials")
    }

    #[test]
    fn invalid_session_maps_to_authentication_required() {
        let error = ApiError::from(AuthenticateSessionError::InvalidSession);

        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.code(), "authentication_required")
    }

    #[test]
    fn expired_session_maps_to_authentication_required() {
        let error = ApiError::from(AuthenticateSessionError::SessionExpired);

        assert_eq!(error.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(error.code(), "authentication_required")
    }

    #[test]
    fn internal_account_error_maps_to_internal_server_error() {
        let error = ApiError::from(RegisterUserError::Internal(InternalError::Failed));

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.code(), "internal_error")
    }
}
