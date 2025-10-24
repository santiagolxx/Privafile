use crate::core::paseto_keys_path;
use crate::core::procedures::{authenticate_user, register_user};
use crate::core::{
    cryptography::authentication::PasetoManager,
    structs::{AuthResponse, LoginCredentials},
};
use anyhow::Result;
use rocket::response::status;
use rocket::{post, serde::json::Json};

#[post("/api/auth/register", data = "<credentials>")]
pub async fn register(
    credentials: Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, status::Custom<String>> {
    let creds = credentials.into_inner();
    let username = creds.username;
    let password = creds.password;

    let auth_response = register_user(&username, &password)
        .await
        .map_err(|e| status::Custom(rocket::http::Status::BadRequest, e.to_string()))?;

    let paseto_manager = PasetoManager::from_file(paseto_keys_path())
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    let token = paseto_manager
        .create_token(&auth_response, 72)
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    Ok(Json(AuthResponse {
        sucess: true,
        message: "User registered successfully".to_string(),
        token,
    }))
}

#[post("/api/auth/login", data = "<credentials>")]
pub async fn login(
    credentials: Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, status::Custom<String>> {
    let creds = credentials.into_inner();
    let username = creds.username;
    let password = creds.password;

    let auth_response = authenticate_user(&username, &password)
        .await
        .map_err(|e| status::Custom(rocket::http::Status::Unauthorized, e.to_string()))?;

    let paseto_manager = PasetoManager::from_file(paseto_keys_path())
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    let token = paseto_manager
        .create_token(&auth_response, 72)
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    Ok(Json(AuthResponse {
        sucess: true,
        message: "Welcome back!".to_string(),
        token,
    }))
}
