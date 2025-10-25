use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::core::database::schema::{chunks, files, usuarios};

// ============================================================================
// Database Models
// ============================================================================

#[derive(Queryable, Debug)]
pub struct Usuario {
    pub id: String,
    pub username: String,
    pub password: String,
    pub b64_pubkey: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = usuarios)]
pub struct NuevoUsuario<'a> {
    pub id: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub b64_pubkey: Option<&'a str>,
}

#[derive(Queryable, Debug, Serialize)]
pub struct File {
    pub id: String,
    pub mime: String,
    pub hash: String,
    pub owner_id: String,
    pub status: String,
    pub total_size: Option<i32>,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NuevoFile<'a> {
    pub id: &'a str,
    pub mime: &'a str,
    pub hash: &'a str,
    pub owner_id: &'a str,
    pub status: &'a str,
    pub total_size: Option<i32>,
}

#[derive(Queryable, Debug, Serialize, Clone)]
pub struct Chunk {
    pub id: String,
    pub file_id: String,
    pub chunk_index: i32,
    pub hash: String,
    pub size: i32,
    pub status: String,
    pub created_at: String,
}

#[derive(Insertable)]
#[diesel(table_name = chunks)]
pub struct NuevoChunk<'a> {
    pub id: &'a str,
    pub file_id: &'a str,
    pub chunk_index: i32,
    pub hash: &'a str,
    pub size: i32,
    pub status: &'a str,
}

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Deserialize, Debug)]
pub struct InitUploadRequest {
    pub file_id: String,
    pub total_chunks: i32,
    pub mime: String,
    pub total_size: i32,
}

#[derive(Serialize)]
pub struct InitUploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: String,
}

#[derive(Deserialize, Debug)]
pub struct FinalizeUploadRequest {
    pub file_id: String,
}

#[derive(Serialize)]
pub struct FinalizeUploadResponse {
    pub success: bool,
    pub message: String,
    pub file_id: String,
    pub total_chunks: usize,
    pub hash: String,
}

#[derive(Serialize)]
pub struct ChunkUploadResponse {
    pub success: bool,
    pub message: String,
    pub chunk_index: i32,
    pub hash: String,
}

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
    pub status: String,
    pub total_size: Option<i32>,
    pub created_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: String,
}
