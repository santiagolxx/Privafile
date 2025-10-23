use rocket::serde::json::Json;
use rocket::{http::Status, post, response::status::Custom};
use serde::{Deserialize, Serialize};
use std::io::Read;
use tracing::{Level, error, info, span};

use crate::core::init_db_manager;
use crate::core::procedures::upload_file;

#[derive(Deserialize)]
pub struct UploadRequest {
    pub user_id: String,
    pub mime: String,
}

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: Option<String>,
}

/// Ruta para subir archivos con validación básica
///
/// Endpoint: POST /api/files/upload
///
/// Headers:
/// ```
/// Content-Type: application/octet-stream
/// ```
///
/// Query params:
/// ```
/// user_id=usuario-id&mime=application/pdf
/// ```
///
/// El contenido del archivo va en el body (raw binary)
#[post("/api/files/upload?<user_id>&<mime>", data = "<data>")]
pub async fn upload_file_route(
    user_id: String,
    mime: String,
    data: rocket::Data<'_>,
) -> Result<Json<UploadResponse>, Custom<Json<UploadResponse>>> {
    let span = span!(Level::INFO, "upload_file_route");
    let _enter = span.enter();

    // Validar user_id
    if user_id.is_empty() {
        error!("user_id vacío");
        return Err(Custom(
            Status::BadRequest,
            Json(UploadResponse {
                success: false,
                message: "El user_id no puede estar vacío".to_string(),
                file_id: None,
            }),
        ));
    }

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
    match db.buscar_usuario(&user_id) {
        Ok(_) => {
            info!("Usuario encontrado: {}", user_id);
        }
        Err(_) => {
            error!("Usuario no encontrado: {}", user_id);
            return Err(Custom(
                Status::NotFound,
                Json(UploadResponse {
                    success: false,
                    message: format!("Usuario '{}' no encontrado", user_id),
                    file_id: None,
                }),
            ));
        }
    }

    // Leer el contenido del archivo desde el data stream
    let mut stream = data.open(rocket::data::ToByteUnit::mebibytes(100)); // Límite de 100 MiB
    let mut file_content = Vec::new();

    match tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut file_content).await {
        Ok(_) => {
            info!(
                "Subiendo archivo para usuario: {}, tamaño: {} bytes, mime: {}",
                user_id,
                file_content.len(),
                mime
            );

            // Validar que el archivo no esté vacío
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

            // Llamar a la función de procedimiento
            match upload_file(&user_id, &mime, file_content).await {
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
