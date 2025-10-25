use crate::core::cryptography::authentication::PasetoManager;
use crate::core::paseto_keys_path;
use crate::core::procedures::{authenticate_user, register_user};
use crate::core::structs::{AuthResponse, LoginCredentials};
use rocket::response::status;
use rocket::{post, serde::json::Json};

#[post("/api/auth/register", data = "<credentials>")]
pub async fn register(
    credentials: Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, status::Custom<String>> {
    let creds = credentials.into_inner();

    let user_id = register_user(&creds.username, &creds.password)
        .await
        .map_err(|e| status::Custom(rocket::http::Status::BadRequest, e.to_string()))?;

    let paseto_manager = PasetoManager::from_file(paseto_keys_path())
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    let token = paseto_manager
        .create_token(&user_id, 72)
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    Ok(Json(AuthResponse {
        success: true,
        message: "Usuario registrado exitosamente".to_string(),
        token,
    }))
}

#[post("/api/auth/login", data = "<credentials>")]
pub async fn login(
    credentials: Json<LoginCredentials>,
) -> Result<Json<AuthResponse>, status::Custom<String>> {
    let creds = credentials.into_inner();

    let user_id = authenticate_user(&creds.username, &creds.password)
        .await
        .map_err(|e| status::Custom(rocket::http::Status::Unauthorized, e.to_string()))?;

    let paseto_manager = PasetoManager::from_file(paseto_keys_path())
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    let token = paseto_manager
        .create_token(&user_id, 72)
        .map_err(|e| status::Custom(rocket::http::Status::InternalServerError, e.to_string()))?;

    Ok(Json(AuthResponse {
        success: true,
        message: "Bienvenido de vuelta!".to_string(),
        token,
    }))
}
