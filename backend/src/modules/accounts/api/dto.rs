use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub display_name: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterUserResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUserRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginUserResponse {
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct GetUserResponse {
    pub id: String,
    pub display_name: String,
    pub email: String,
}
