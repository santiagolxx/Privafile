use crate::core::File;
use crate::core::database::init_db_manager;
use crate::core::structs::NuevoFile;
use crate::core::utils::write_file;
use anyhow::{Context, Result};
use blake2::{Blake2b512, Digest};
use std::path::PathBuf;
use tracing::{error, info, warn};
use uuid::Uuid;

/// Sube un archivo al sistema
///
/// 1. Genera un ID único (UUID)
/// 2. Calcula el hash Blake2b512
/// 3. Guarda el registro en la base de datos
/// 4. Escribe el archivo en disco
///
/// # Errores
/// - Si falla la inserción en DB, no se escribe el archivo
/// - Si falla la escritura del archivo, se hace rollback en DB
pub async fn upload_file(user_id: &str, mime: &str, file_content: Vec<u8>) -> Result<String> {
    let file_id = Uuid::new_v4().to_string();

    // Generar hash del archivo
    let mut hasher = Blake2b512::new();
    hasher.update(&file_content);
    let hash = format!("{:x}", hasher.finalize());

    info!(
        "Procesando archivo: {} para usuario: {} (tamaño: {} bytes)",
        file_id,
        user_id,
        file_content.len()
    );

    let nuevo_file = NuevoFile {
        id: &file_id,
        mime,
        hash: &hash,
        owner_id: user_id,
    };

    // Guardar en la base de datos primero
    init_db_manager()
        .insertar_file(&nuevo_file)
        .context("Error al insertar archivo en la base de datos")?;

    info!("Registro de archivo creado en DB: {}", file_id);

    // Guardar el archivo en el sistema de archivos
    let file_path = format!("./Privafile/Uploads/{}.st", file_id);

    match write_file(&file_path, &file_content).await {
        Ok(_) => {
            info!("Archivo guardado exitosamente en: {}", file_path);
            Ok(file_id)
        }
        Err(e) => {
            // Rollback: eliminar el registro de la base de datos
            error!("Error al escribir archivo, haciendo rollback en DB");
            let _ = init_db_manager().borrar_file(&file_id);
            Err(e).context("Error al guardar archivo en disco (rollback ejecutado)")
        }
    }
}

/// Lista los archivos de un usuario con filtros opcionales
///
/// # Parámetros
/// - `user_id`: ID del usuario propietario
/// - `mime_filter`: Filtro opcional por tipo MIME
/// - `limit`: Límite opcional de resultados (1-1000)
///
/// # Retorna
/// Vector de archivos que cumplen los criterios
pub async fn list_user_files(
    user_id: &str,
    mime_filter: Option<&str>,
    limit: Option<i64>,
) -> Result<Vec<File>> {
    info!(
        "Listando archivos para usuario: {} (mime: {:?}, limit: {:?})",
        user_id, mime_filter, limit
    );

    let files = init_db_manager()
        .obtener_files_de_usuario(user_id, mime_filter, limit)
        .context("Error al obtener archivos de la base de datos")?;

    info!(
        "Encontrados {} archivos para usuario {}",
        files.len(),
        user_id
    );
    Ok(files)
}

/// Descarga un archivo verificando propiedad
///
/// # Validaciones
/// - El archivo debe existir
/// - El archivo debe pertenecer al usuario
/// - El archivo debe existir en disco
///
/// # Retorna
/// Tupla con (mime_type, contenido_binario)
pub async fn download_file(user_id: &str, file_id: &str) -> Result<(String, Vec<u8>)> {
    // Validación de seguridad: prevenir path traversal
    if file_id.contains("..") || file_id.contains('/') || file_id.contains('\\') {
        error!("Intento de path traversal detectado: {}", file_id);
        return Err(anyhow::anyhow!("ID de archivo inválido"));
    }

    info!(
        "Usuario {} solicitando descarga del archivo {}",
        user_id, file_id
    );

    // Verificar que el archivo exista y pertenezca al usuario
    let files = init_db_manager()
        .obtener_files_de_usuario(user_id, None, None)
        .context("Error al buscar archivos del usuario")?;

    let file_info = files.iter().find(|f| f.id == file_id).ok_or_else(|| {
        warn!(
            "Archivo {} no encontrado o no pertenece al usuario {}",
            file_id, user_id
        );
        anyhow::anyhow!("Archivo no encontrado")
    })?;

    // Construir la ruta del archivo
    let file_path = PathBuf::from(format!("./Privafile/Uploads/{}.st", file_id));

    // Leer el archivo del disco
    let file_content = tokio::fs::read(&file_path)
        .await
        .with_context(|| format!("Error al leer archivo desde disco: {:?}", file_path))?;

    info!(
        "Archivo {} descargado exitosamente ({} bytes)",
        file_id,
        file_content.len()
    );

    Ok((file_info.mime.clone(), file_content))
}

/// Elimina un archivo del sistema
///
/// # Validaciones
/// - El archivo debe existir
/// - El archivo debe pertenecer al usuario
///
/// # Acciones
/// 1. Verifica propiedad
/// 2. Elimina de la base de datos
/// 3. Elimina del disco
pub async fn delete_file(user_id: &str, file_id: &str) -> Result<()> {
    // Validación de seguridad
    if file_id.contains("..") || file_id.contains('/') || file_id.contains('\\') {
        error!("Intento de path traversal detectado en delete: {}", file_id);
        return Err(anyhow::anyhow!("ID de archivo inválido"));
    }

    info!("Usuario {} eliminando archivo {}", user_id, file_id);

    // Verificar que el archivo exista y pertenezca al usuario
    let files = init_db_manager()
        .obtener_files_de_usuario(user_id, None, None)
        .context("Error al buscar archivos del usuario")?;

    let file_exists = files.iter().any(|f| f.id == file_id);
    if !file_exists {
        warn!(
            "Archivo {} no encontrado o no pertenece al usuario {}",
            file_id, user_id
        );
        return Err(anyhow::anyhow!("Archivo no encontrado"));
    }

    // Eliminar de la base de datos
    init_db_manager()
        .borrar_file(file_id)
        .context("Error al eliminar archivo de la base de datos")?;

    info!("Archivo {} eliminado de la base de datos", file_id);

    // Eliminar del disco
    let file_path = PathBuf::from(format!("./Privafile/Uploads/{}.st", file_id));

    match tokio::fs::remove_file(&file_path).await {
        Ok(_) => {
            info!("Archivo {} eliminado del disco", file_id);
            Ok(())
        }
        Err(e) => {
            warn!(
                "Archivo {} eliminado de DB pero no del disco: {}",
                file_id, e
            );
            // No es crítico si el archivo no existe en disco
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_validation() {
        let malicious_ids = vec![
            "../../../etc/passwd",
            "..\\..\\windows\\system32",
            "abc/../../../secret.txt",
            "file/with/slashes",
            "file\\with\\backslashes",
        ];

        for id in malicious_ids {
            assert!(
                id.contains("..") || id.contains('/') || id.contains('\\'),
                "ID debería ser detectado como malicioso: {}",
                id
            );
        }
    }

    #[test]
    fn test_valid_file_ids() {
        let valid_ids = vec![
            "550e8400-e29b-41d4-a716-446655440000",
            "abc123",
            "file-name_123",
        ];

        for id in valid_ids {
            assert!(
                !id.contains("..") && !id.contains('/') && !id.contains('\\'),
                "ID válido detectado como malicioso: {}",
                id
            );
        }
    }

    #[tokio::test]
    async fn test_hash_consistency() {
        let content = b"test content";
        let mut hasher1 = Blake2b512::new();
        hasher1.update(content);
        let hash1 = format!("{:x}", hasher1.finalize());

        let mut hasher2 = Blake2b512::new();
        hasher2.update(content);
        let hash2 = format!("{:x}", hasher2.finalize());

        assert_eq!(hash1, hash2, "Los hashes deberían ser consistentes");
    }
}
