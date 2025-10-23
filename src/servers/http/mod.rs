use rocket::routes;
use tracing::info;

// Internal crates
use crate::core::http_port;

mod routes;

pub fn start_server() -> rocket::Rocket<rocket::Build> {
    info!("Iniciando servidor Privafile en el puerto {}", http_port());

    let figment = rocket::Config::figment()
        .merge(("port", http_port()))
        .merge(("log_level", rocket::config::LogLevel::Critical));

    rocket::custom(figment).mount("/", routes![routes::upload_file_route])
}
