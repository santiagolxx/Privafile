use rocket::routes;
use rocket_cors::{AllowedOrigins, CorsOptions};
use tracing::info;

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

    let cors = CorsOptions {
        allowed_origins: AllowedOrigins::all(),
        allowed_methods: vec![
            rocket::http::Method::Get,
            rocket::http::Method::Post,
            rocket::http::Method::Put,
            rocket::http::Method::Delete,
        ]
        .into_iter()
        .map(From::from)
        .collect(),
        allowed_headers: rocket_cors::AllowedHeaders::all(),
        allow_credentials: true,
        ..Default::default()
    }
    .to_cors()
    .expect("Error al crear CORS");

    rocket::custom(figment)
        .manage(paseto_manager)
        .mount(
            "/",
            routes![
                routes::init_upload_route,
                routes::upload_chunk_route,
                routes::finalize_upload_route,
                routes::list_files_route,
                routes::download_file_route,
                routes::download_chunk_route,
                routes::delete_file_route,
                routes::login,
                routes::register,
            ],
        )
        .attach(cors)
}
