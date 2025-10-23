use crate::core::database::init_db_manager;
use crate::core::structs::NuevoFile;
use crate::core::utils::write_file;
use anyhow::Result;
use blake2::{Blake2b512, Digest};
use tracing::info;
use uuid::Uuid;

pub async fn upload_file(user_id: &str, mime: &str, file_content: Vec<u8>) -> Result<String> {
    let file_id = Uuid::new_v4().to_string();

    // Generar hash del archivo
    let mut hasher = Blake2b512::new();
    hasher.update(&file_content);
    let hash = format!("{:x}", hasher.finalize());

    info!("Procesando archivo: {} para usuario: {}", file_id, user_id);

    let nuevo_file = NuevoFile {
        id: &file_id,
        mime,
        hash: &hash,
        owner_id: user_id,
    };

    // Guardar en la base de datos
    init_db_manager().insertar_file(&nuevo_file)?;
    info!("Registro de archivo creado en DB: {}", file_id);

    // Guardar el archivo en el sistema de archivos
    let file_path = format!("./Privafile/Uploads/{}.st", file_id);
    write_file(&file_path, &file_content).await?;
    info!("Archivo guardado en: {}", file_path);

    Ok(file_id)
}
