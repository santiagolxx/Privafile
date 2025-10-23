use crate::core::getters::http_port;
use rocket::{get, routes};
use tracing::{Level, error, info, span};
#[get("/usuario/<id>")]
fn obtener_usuario(id: u32) -> String {
    let span = span!(Level::INFO, "my_span");

    if id == 0 {
        error!("ID inválido: {}", id);
        return "Error: ID inválido".to_string();
    }
    let _enter = span.enter();
    format!("Usuario {}", id)
}

pub fn start_server() -> rocket::Rocket<rocket::Build> {
    info!("Iniciando servidor Privafile en {}", http_port());

    let figment = rocket::Config::figment()
        .merge(("port", http_port()))
        .merge(("log_level", rocket::config::LogLevel::Critical));

    rocket::custom(figment).mount("/", routes![obtener_usuario])
}
