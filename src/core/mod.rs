// src/core/mod.rs
// ── Internal modules ─────────────────────────────────────────────────
pub(crate) mod cryptography;
mod database;
pub mod procedures;
mod structs;
mod utils;

// ── Direct re-exports for easier access ──────────────────────────────
pub use database::{get_db_manager, init_db_manager, run_migrations};
pub use structs::{File, NuevoFile, NuevoUsuario, Usuario};
pub use utils::{
    Config, check_temp_perms, db_url, http_port, load_config, paseto_keys_path, write_file,
};

// ── Organized sub-modules (if you prefer) ────────────────────────────
pub mod database_ops {
    pub use crate::core::database::operations::{DbManager, get_db_manager, init_db_manager};
}

pub mod schema {
    pub use crate::core::database::schema;
}
