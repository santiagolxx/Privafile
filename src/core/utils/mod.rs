use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{error, info};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub uploads_path: String,
    pub http_port: u16,
    pub database_url: String,
}

pub static CONFIG: OnceCell<Config> = OnceCell::new();

/// Lee ./Privafile/Privafile.toml o crea uno por defecto si no existe.
/// También asegura que la carpeta ./Privafile exista.
pub async fn load_config() -> anyhow::Result<()> {
    let base_dir = Path::new("./Privafile");
    let config_path = base_dir.join("Privafile.toml");

    // Crear la carpeta base si no existe
    if !base_dir.exists() {
        fs::create_dir_all(base_dir)
            .await
            .expect("No se pudo crear el directorio ./Privafile");
        info!("Directorio base ./Privafile creado");
    }

    // Si no existe el archivo de configuración, crear uno por defecto
    if !config_path.exists() {
        let default_config = Config {
            uploads_path: "./Privafile/Uploads".to_string(),
            http_port: 5830,
            database_url: "./Privafile/Privafile.db".to_string(),
        };
        let toml_string = toml::to_string_pretty(&default_config)?;
        fs::write(&config_path, toml_string)
            .await
            .expect("No se pudo crear el archivo de configuración por defecto");
        info!(
            "Archivo de configuración por defecto creado en {:?}",
            config_path
        );
    }

    // Leer el archivo (sea nuevo o existente)
    let content = fs::read_to_string(&config_path)
        .await
        .expect("No se pudo leer Privafile.toml");
    let config: Config =
        toml::from_str(&content).expect("Error al parsear el archivo de configuración TOML");

    CONFIG
        .set(config)
        .map_err(|_| anyhow::anyhow!("La configuración ya fue inicializada"))?;
    info!("Configuración cargada desde {:?}", config_path);

    Ok(())
}

/// Chequea permisos en el directorio temporal y hace panic ante errores graves.
/// Si el directorio no existe, lo crea automáticamente.
pub async fn check_temp_perms() -> anyhow::Result<()> {
    let config = CONFIG
        .get()
        .expect("Configuración no inicializada. Llama a load_config() primero.");

    let path = PathBuf::from(&config.uploads_path);

    // Crear el directorio si no existe
    if !path.exists() {
        fs::create_dir_all(&path).await.unwrap_or_else(|e| {
            panic!(
                "No se pudo crear el directorio temporal {:?}: {:?}",
                path, e
            )
        });
        info!("Directorio temporal creado en {:?}", path);
    }

    // Verificar permisos
    match fs::metadata(&path).await {
        Ok(metadata) => {
            let permissions = metadata.permissions();
            info!("Permisos de {:?}: {:?}", path, permissions);

            if permissions.readonly() {
                panic!("No tenemos permiso para escribir en {:?}", path);
            } else {
                info!("Tenemos permisos de escritura en {:?}", path);
            }
        }
        Err(e) => panic!("No se pudo acceder al path {:?}: {:?}", path, e),
    }

    Ok(())
}

pub fn http_port() -> u16 {
    CONFIG
        .get()
        .map(|c| c.http_port)
        .unwrap_or_else(|| {
            error!("Se intentó obtener el puerto del servidor, pero CONFIG no está inicializado. Usando default (5830)");
            5830
        })
}

pub fn db_url() -> String {
    CONFIG
        .get()
        .map(|c| c.database_url.clone())
        .unwrap_or_else(|| {
            error!("Se intentó obtener la url de la base de datos del servidor, pero CONFIG no está inicializado. Usando default (5830)");
            "./Privafile/Privafile.sql".to_string()
        })
}
