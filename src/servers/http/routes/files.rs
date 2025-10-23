use rocket::{get, post};
use tracing::{Level, error, span};

#[get("/usuario/<id>")]
pub fn obtener_usuario(id: u32) -> String {
    let span = span!(Level::INFO, "my_span");

    if id == 0 {
        error!("ID inválido: {}", id);
        return "Error: ID inválido".to_string();
    }
    let _enter = span.enter();
    format!("Usuario {}", id)
}
