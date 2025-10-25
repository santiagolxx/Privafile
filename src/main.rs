use anyhow;
use tokio;
use tracing::info;
use tracing_subscriber::EnvFilter;

use privafile::{
    core::{check_temp_perms, init_db_manager, load_config, run_migrations},
    servers::http::start_server,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info,rocket=off,hyper=off,tokio=off"))
        .init();

    info!("🚀 Iniciando Privafile...");
    info!("📁 Leyendo configuración y ejecutando checks de permisos...");

    load_config().await?;
    check_temp_perms().await?;

    info!("🗄️  Ejecutando migraciones de base de datos...");
    run_migrations();
    init_db_manager();

    info!("🌐 Iniciando servidor HTTP...");
    start_server().launch().await?;

    Ok(())
}
