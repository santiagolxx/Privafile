mod auth;
mod files;

pub use auth::{login, register};
pub use files::{
    delete_file_route, download_chunk_route, download_file_route, finalize_upload_route,
    init_upload_route, list_files_route, upload_chunk_route,
};
