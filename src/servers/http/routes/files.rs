use rocket::data::ToByteUnit;
use rocket::outcome::Outcome;
use rocket::request::{self, FromRequest, Request};
use rocket::serde::json::Json;
use rocket::{Data, State, delete, get, http::Status, post, response::status::Custom};
use serde_json::json;
use tracing::{Level, error, info, span, warn};

use crate::core::cryptography::authentication::PasetoManager;
use crate::core::procedures::{
    delete_file, download_chunk, download_file, finalize_chunked_upload, init_chunked_upload,
    list_user_files, upload_chunk,
};
use crate::core::structs::{
    ChunkUploadResponse, DeleteResponse, File, FileInfo, FileListResponse, FinalizeUploadRequest,
    FinalizeUploadResponse, InitUploadRequest, InitUploadResponse,
};

// ============================================================================
// Authentication Guard
// ============================================================================

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

impl From<File> for FileInfo {
    fn from(file: File) -> Self {
        FileInfo {
            id: file.id,
            mime: file.mime,
            hash: file.hash,
            status: file.status,
            total_size: file.total_size,
            created_at: file.created_at,
        }
    }
}

// ============================================================================
// Chunked Upload Routes
// ============================================================================

/// Inicia un upload fragmentado
#[post("/api/files/upload/init", data = "<request>")]
pub async fn init_upload_route(
    user: AuthenticatedUser,
    request: Json<InitUploadRequest>,
) -> Result<Json<InitUploadResponse>, Custom<String>> {
    let span = span!(Level::INFO, "init_upload");
    let _enter = span.enter();

    let req = request.into_inner();

    // Validaciones
    if req.total_chunks <= 0 {
        return Err(Custom(
            Status::BadRequest,
            "El número de chunks debe ser positivo".to_string(),
        ));
    }

    if req.mime.is_empty() || !req.mime.contains('/') {
        return Err(Custom(Status::BadRequest, "MIME type inválido".to_string()));
    }

    match init_chunked_upload(&user.user_id, &req.file_id, &req.mime, req.total_size).await {
        Ok(_) => {
            info!(
                "Upload iniciado: {} ({} chunks)",
                req.file_id, req.total_chunks
            );
            Ok(Json(InitUploadResponse {
                success: true,
                message: "Upload iniciado exitosamente".to_string(),
                file_id: req.file_id,
            }))
        }
        Err(e) => {
            error!("Error al iniciar upload: {}", e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        }
    }
}

/// Sube un chunk individual
#[post("/api/files/upload/chunk?<file_id>&<chunk_index>", data = "<data>")]
pub async fn upload_chunk_route(
    user: AuthenticatedUser,
    file_id: String,
    chunk_index: i32,
    data: Data<'_>,
) -> Result<Json<ChunkUploadResponse>, Custom<String>> {
    let span = span!(Level::INFO, "upload_chunk");
    let _enter = span.enter();

    // Validar chunk_index
    if chunk_index < 0 {
        return Err(Custom(
            Status::BadRequest,
            "El índice de chunk debe ser positivo".to_string(),
        ));
    }

    // Leer datos del chunk (máximo 10 MiB)
    let mut stream = data.open(ToByteUnit::mebibytes(10));
    let mut chunk_data = Vec::new();

    if let Err(e) = tokio::io::AsyncReadExt::read_to_end(&mut stream, &mut chunk_data).await {
        error!("Error al leer chunk: {}", e);
        return Err(Custom(Status::BadRequest, e.to_string()));
    }

    if chunk_data.is_empty() {
        return Err(Custom(
            Status::BadRequest,
            "El chunk no puede estar vacío".to_string(),
        ));
    }

    match upload_chunk(&user.user_id, &file_id, chunk_index, chunk_data).await {
        Ok(hash) => {
            info!("Chunk {} subido exitosamente", chunk_index);
            Ok(Json(ChunkUploadResponse {
                success: true,
                message: format!("Chunk {} subido exitosamente", chunk_index),
                chunk_index,
                hash,
            }))
        }
        Err(e) => {
            error!("Error al subir chunk {}: {}", chunk_index, e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        }
    }
}

/// Finaliza el upload
#[post("/api/files/upload/finalize", data = "<request>")]
pub async fn finalize_upload_route(
    user: AuthenticatedUser,
    request: Json<FinalizeUploadRequest>,
) -> Result<Json<FinalizeUploadResponse>, Custom<String>> {
    let span = span!(Level::INFO, "finalize_upload");
    let _enter = span.enter();

    let file_id = &request.file_id;

    match finalize_chunked_upload(&user.user_id, file_id).await {
        Ok((hash, total_chunks)) => {
            info!("Upload finalizado: {} ({} chunks)", file_id, total_chunks);
            Ok(Json(FinalizeUploadResponse {
                success: true,
                message: "Upload completado exitosamente".to_string(),
                file_id: file_id.clone(),
                total_chunks,
                hash,
            }))
        }
        Err(e) => {
            error!("Error al finalizar upload: {}", e);
            Err(Custom(Status::InternalServerError, e.to_string()))
        }
    }
}

// ============================================================================
// Download Routes
// ============================================================================

/// Lista archivos del usuario
#[get("/api/files/list?<mime>&<limit>")]
pub async fn list_files_route(
    user: AuthenticatedUser,
    mime: Option<String>,
    limit: Option<i64>,
) -> Result<Json<FileListResponse>, Custom<Json<FileListResponse>>> {
    let span = span!(Level::INFO, "list_files");
    let _enter = span.enter();

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
            error!("Error al listar archivos: {}", e);
            Err(Custom(
                Status::InternalServerError,
                Json(FileListResponse {
                    success: false,
                    message: e.to_string(),
                    files: vec![],
                }),
            ))
        }
    }
}

/// Descarga un archivo completo
#[get("/api/files/download/<file_id>")]
pub async fn download_file_route(
    user: AuthenticatedUser,
    file_id: String,
) -> Result<(Status, (rocket::http::ContentType, Vec<u8>)), Custom<String>> {
    let span = span!(Level::INFO, "download_file");
    let _enter = span.enter();

    match download_file(&user.user_id, &file_id).await {
        Ok((mime_type, file_content)) => {
            let content_type = rocket::http::ContentType::parse_flexible(&mime_type)
                .unwrap_or(rocket::http::ContentType::Binary);

            Ok((Status::Ok, (content_type, file_content)))
        }
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("no encontrado") {
                Status::NotFound
            } else if error_msg.contains("inválido") || error_msg.contains("No autorizado") {
                Status::Forbidden
            } else {
                Status::InternalServerError
            };

            Err(Custom(status, error_msg))
        }
    }
}

/// Descarga un chunk individual
#[get("/api/files/download/<file_id>/chunk/<chunk_index>")]
pub async fn download_chunk_route(
    user: AuthenticatedUser,
    file_id: String,
    chunk_index: i32,
) -> Result<(Status, Vec<u8>), Custom<String>> {
    let span = span!(Level::INFO, "download_chunk");
    let _enter = span.enter();

    match download_chunk(&user.user_id, &file_id, chunk_index).await {
        Ok((chunk_data, _hash)) => Ok((Status::Ok, chunk_data)),
        Err(e) => {
            let error_msg = e.to_string();
            let status = if error_msg.contains("no encontrado") {
                Status::NotFound
            } else {
                Status::InternalServerError
            };

            Err(Custom(status, error_msg))
        }
    }
}

// ============================================================================
// Delete Route
// ============================================================================

#[delete("/api/files/delete/<file_id>")]
pub async fn delete_file_route(
    user: AuthenticatedUser,
    file_id: String,
) -> Result<Json<DeleteResponse>, Custom<Json<DeleteResponse>>> {
    let span = span!(Level::INFO, "delete_file");
    let _enter = span.enter();

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
