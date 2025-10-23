use rocket::data::{Limits, ToByteUnit};
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use rocket::{Data, State, http::Status, post, response::status::Custom};
use serde::Serialize;
use tracing::{Level, error, info, span};

use crate::core::cryptography::authentication::PasetoManager;
use crate::core::init_db_manager;
use crate::core::procedures::upload_file;

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: Option<String>,
}

/// Guard para extraer y validar el token PASETO del header Authorization
pub struct AuthenticatedUser {
    pub user_id: String,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for AuthenticatedUser {
    type Error = String;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let auth_header = match request.headers().get_one("Authorization") {
            Some(h) => h,
            None => {
                return Outcome::Error((
                    Status::Unauthorized,
                    "Missing Authorization header".to_string(),
                ));
            }
        };

        let token = if auth_header.starts_with("Bearer ") {
            &auth_header[7..]
        } else {
            return Outcome::Error((
                Status::Unauthorized,
                "Invalid Authorization format. Use: Bearer <token>".to_string(),
            ));
        };

        let paseto_manager = match request.guard::<&State<PasetoManager>>().await {
            Outcome::Success(manager) => manager,
            _ => {
                return Outcome::Error((
                    Status::InternalServerError,
                    "PasetoManager not available".to_string(),
                ));
            }
        };

        match paseto_manager.verify_token(token) {
            Ok(claims) => Outcome::Success(AuthenticatedUser {
                user_id: claims.sub,
            }),
            Err(e) => Outcome::Error((Status::Unauthorized, format!("Invalid token: {}", e))),
        }
    }
}

/// Ruta para subir archivos con autenticación PASETO
///
/// Endpoint: POST /api/files/upload?mime=application/pdf
///
/// Headers:
/// ```
/// Content-Type: application/octet-stream
/// Authorization: Bearer <paseto-token>
/// ```
///
/// Body: archivo binario raw
#[post("/api/files/upload?<mime>", data = "<data>")]
pub async fn upload_file_route(
    user: AuthenticatedUser,
    mime: String,
    data: Data<'_>,
) -> Result<Json<UploadResponse>, Custom<Json<UploadResponse>>> {
    let span = span!(Level::INFO, "upload_file_route");
    let _enter = span.enter();

    // Validar mime type
    if mime.is_empty() {
        error!("mime type vacío");
        return Err(Custom(
            Status::BadRequest,
            Json(UploadResponse {
                success: false,
                message: "El mime type no puede estar vacío".to_string(),
                file_id: None,
            }),
        ));
    }

    // Verificar que el usuario exista
    let db = init_db_manager();
    match db.buscar_usuario(&user.user_id) {
        Ok(_) => {
            info!("Usuario autenticado: {}", user.user_id);
        }
        Err(_) => {
            error!("Usuario no encontrado: {}", user.user_id);
            return Err(Custom(
                Status::NotFound,
                Json(UploadResponse {
                    success: false,
                    message: format!("Usuario '{}' no encontrado", user.user_id),
                    file_id: None,
                }),
            ));
        }
    }

    // Leer el archivo (límite de 100 MiB)
    let mut stream = data.open(ToByteUnit::mebibytes(100));
    let mut file_content = Vec::new();

    match tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut file_content).await {
        Ok(_) => {
            if file_content.is_empty() {
                error!("Archivo vacío");
                return Err(Custom(
                    Status::BadRequest,
                    Json(UploadResponse {
                        success: false,
                        message: "El archivo no puede estar vacío".to_string(),
                        file_id: None,
                    }),
                ));
            }

            info!(
                "Subiendo archivo para usuario: {}, tamaño: {} bytes, mime: {}",
                user.user_id,
                file_content.len(),
                mime
            );

            match upload_file(&user.user_id, &mime, file_content).await {
                Ok(file_id) => {
                    info!("Archivo subido exitosamente: {}", file_id);
                    Ok(Json(UploadResponse {
                        success: true,
                        message: "Archivo subido exitosamente".to_string(),
                        file_id: Some(file_id),
                    }))
                }
                Err(e) => {
                    error!("Error al subir archivo: {}", e);
                    Err(Custom(
                        Status::InternalServerError,
                        Json(UploadResponse {
                            success: false,
                            message: format!("Error al subir archivo: {}", e),
                            file_id: None,
                        }),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Error al leer datos del request: {}", e);
            Err(Custom(
                Status::BadRequest,
                Json(UploadResponse {
                    success: false,
                    message: format!("Error al leer datos del archivo: {}", e),
                    file_id: None,
                }),
            ))
        }
    }
}
