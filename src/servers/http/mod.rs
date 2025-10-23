use rocket::routes;
use tracing::info;

// Internal crates
use crate::core::{cryptography::authentication::PasetoManager, http_port, paseto_keys_path};
mod routes;

pub fn start_server() -> rocket::Rocket<rocket::Build> {
    let paseto_manager =
        PasetoManager::from_file(paseto_keys_path()).expect("Failed to initialize PasetoManager");

    info!("PasetoManager inicializado correctamente");
    info!("Iniciando servidor Privafile en el puerto {}", http_port());

    let figment = rocket::Config::figment()
        .merge(("port", http_port()))
        .merge(("log_level", rocket::config::LogLevel::Critical));

    rocket::custom(figment).manage(paseto_manager).mount(
        "/",
        routes![
            routes::upload_file_route,
            routes::list_files_route,
            routes::download_file_route,
            routes::delete_file_route
        ],
    )
}
