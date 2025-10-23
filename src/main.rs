use anyhow;

// External crates
use tokio;
use tracing::info;
use tracing_subscriber::EnvFilter;

// Internal crates
use privafile::{
    core::utilities::{check_temp_perms, load_config, run_migrations},
    servers::http::start_server,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info,rocket=off,hyper=off,tokio=off"))
        .init();

    info!("Leyendo configuración y ejecutando checks de permisos...");

    load_config().await?;
    check_temp_perms().await?;
    run_migrations();
    info!("Iniciando servidor...");
    start_server().launch().await?;

    Ok(())
}
