use diesel::prelude::{Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::core::database::schema::{files, usuarios};

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

#[derive(Queryable, Debug)]
pub struct File {
    pub id: String,
    pub mime: String,
    pub hash: String,
    pub owner_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NuevoFile<'a> {
    pub id: &'a str,
    pub mime: &'a str,
    pub hash: &'a str,
    pub owner_id: &'a str,
}

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

#[derive(Serialize, Deserialize)]
pub struct LoginCredentials {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize)]
pub struct AuthResponse {
    pub(crate) sucess: bool,
    pub(crate) message: String,
    pub(crate) token: String,
}
