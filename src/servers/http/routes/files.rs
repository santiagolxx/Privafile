use rocket::data::ToByteUnit;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use rocket::{Data, State, delete, get, http::Status, post, response::status::Custom};
use serde::{Deserialize, Serialize};
use tracing::{Level, error, info, span, warn};

use crate::core::File;
use crate::core::cryptography::authentication::PasetoManager;
use crate::core::init_db_manager;
use crate::core::procedures::{delete_file, download_file, list_user_files, upload_file};

// ============================================================================
// Response Types
// ============================================================================

#[derive(Serialize)]
pub struct UploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: Option<String>,
}

#[derive(Serialize)]
pub struct FileListResponse {
    pub success: bool,
    pub message: String,
    pub files: Vec<FileInfo>,
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub success: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize)]
pub struct FileInfo {
    pub id: String,
    pub mime: String,
    pub hash: String,
}

impl From<File> for FileInfo {
    fn from(file: File) -> Self {
        FileInfo {
            id: file.id,
            mime: file.mime,
            hash: file.hash,
        }
    }
}

// ============================================================================
// Authentication Guard
// ============================================================================

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
            Err(e) => {
                warn!("Token verification failed: {}", e);
                Outcome::Error((Status::Unauthorized, format!("Invalid token: {}", e)))
            }
        }
    }
}

// ============================================================================
// Routes
// ============================================================================

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
    if mime.is_empty() || !mime.contains('/') || mime.len() > 100 {
        error!("mime type inválido: {}", mime);
        return Err(Custom(
            Status::BadRequest,
            Json(UploadResponse {
                success: false,
                message: "El mime type es inválido".to_string(),
                file_id: None,
            }),
        ));
    }

    // Verificar que el usuario exista
    let db = init_db_manager();
    if db.buscar_usuario(&user.user_id).is_err() {
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

    // Leer el archivo (límite de 100 MiB)
    let mut stream = data.open(ToByteUnit::mebibytes(100));
    let mut file_content = Vec::new();

    if let Err(e) = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut file_content).await {
        error!("Error al leer datos del request: {}", e);
        return Err(Custom(
            Status::BadRequest,
            Json(UploadResponse {
                success: false,
                message: format!("Error al leer datos del archivo: {}", e),
                file_id: None,
            }),
        ));
    }

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

    // Usar el procedure para subir el archivo
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

/// Ruta para listar los archivos del usuario autenticado
///
/// Endpoint: GET /api/files/list?mime=<optional>&limit=<optional>
///
/// Headers:
/// ```
/// Authorization: Bearer <paseto-token>
/// ```
///
/// Query params (opcionales):
/// - mime: Filtrar por tipo MIME (ej: "application/pdf")
/// - limit: Límite de resultados (1-1000)
#[get("/api/files/list?<mime>&<limit>")]
pub async fn list_files_route(
    user: AuthenticatedUser,
    mime: Option<String>,
    limit: Option<i64>,
) -> Result<Json<FileListResponse>, Custom<Json<FileListResponse>>> {
    let span = span!(Level::INFO, "list_files_route");
    let _enter = span.enter();

    // Validar límite si se proporciona
    if let Some(lim) = limit {
        if lim <= 0 || lim > 1000 {
            return Err(Custom(
                Status::BadRequest,
                Json(FileListResponse {
                    success: false,
                    message: "El límite debe estar entre 1 y 1000".to_string(),
                    files: vec![],
                }),
            ));
        }
    }

    // Usar el procedure para listar archivos
    match list_user_files(&user.user_id, mime.as_deref(), limit).await {
        Ok(files) => {
            let file_count = files.len();
            let file_infos: Vec<FileInfo> = files.into_iter().map(FileInfo::from).collect();

            Ok(Json(FileListResponse {
                success: true,
                message: format!("Se encontraron {} archivo(s)", file_count),
                files: file_infos,
            }))
        }
        Err(e) => {
            error!("Error al obtener archivos: {}", e);
            Err(Custom(
                Status::InternalServerError,
                Json(FileListResponse {
                    success: false,
                    message: format!("Error al obtener archivos: {}", e),
                    files: vec![],
                }),
            ))
        }
    }
}

/// Ruta para descargar un archivo específico
///
/// Endpoint: GET /api/files/download/<file_id>
///
/// Headers:
/// ```
/// Authorization: Bearer <paseto-token>
/// ```
///
/// Response: El archivo binario con headers apropiados
#[get("/api/files/download/<file_id>")]
pub async fn download_file_route(
    user: AuthenticatedUser,
    file_id: String,
) -> Result<(Status, (rocket::http::ContentType, Vec<u8>)), Custom<String>> {
    let span = span!(Level::INFO, "download_file_route");
    let _enter = span.enter();

    // Usar el procedure para descargar el archivo
    match download_file(&user.user_id, &file_id).await {
        Ok((mime_type, file_content)) => {
            let content_type = rocket::http::ContentType::parse_flexible(&mime_type)
                .unwrap_or(rocket::http::ContentType::Binary);

            Ok((Status::Ok, (content_type, file_content)))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Determinar el status code apropiado
            let status = if error_msg.contains("no encontrado") {
                Status::NotFound
            } else if error_msg.contains("inválido") {
                Status::BadRequest
            } else {
                Status::InternalServerError
            };

            Err(Custom(status, error_msg))
        }
    }
}

/// Ruta para eliminar un archivo
///
/// Endpoint: DELETE /api/files/delete/<file_id>
///
/// Headers:
/// ```
/// Authorization: Bearer <paseto-token>
/// ```
///
/// Response: JSON indicando éxito o error
#[delete("/api/files/delete/<file_id>")]
pub async fn delete_file_route(
    user: AuthenticatedUser,
    file_id: String,
) -> Result<Json<DeleteResponse>, Custom<Json<DeleteResponse>>> {
    let span = span!(Level::INFO, "delete_file_route");
    let _enter = span.enter();

    // Usar el procedure para eliminar el archivo
    match delete_file(&user.user_id, &file_id).await {
        Ok(_) => {
            info!("Archivo {} eliminado exitosamente", file_id);
            Ok(Json(DeleteResponse {
                success: true,
                message: format!("Archivo {} eliminado exitosamente", file_id),
            }))
        }
        Err(e) => {
            let error_msg = e.to_string();

            // Determinar el status code apropiado
            let status = if error_msg.contains("no encontrado") {
                Status::NotFound
            } else if error_msg.contains("inválido") {
                Status::BadRequest
            } else {
                Status::InternalServerError
            };

            error!("Error al eliminar archivo {}: {}", file_id, e);
            Err(Custom(
                status,
                Json(DeleteResponse {
                    success: false,
                    message: error_msg,
                }),
            ))
        }
    }
}
