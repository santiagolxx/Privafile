use crate::core::database::init_db_manager;
use crate::core::structs::{Chunk, File, NuevoChunk, NuevoFile, NuevoUsuario};
use crate::core::utils::write_file;
use anyhow::{Context, Result, anyhow};
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use blake2::{Blake2b512, Digest};
use std::path::PathBuf;
use tracing::{error, info, warn};
use uuid::Uuid;

// ═══════════════════════════════════════════════════════════════════════════
// Upload con Fragmentación
// ═══════════════════════════════════════════════════════════════════════════

/// Inicia un upload fragmentado
pub async fn init_chunked_upload(
    user_id: &str,
    file_id: &str,
    mime: &str,
    total_size: i32,
) -> Result<()> {
    info!(
        "Iniciando upload fragmentado: {} para usuario: {} ({} bytes)",
        file_id, user_id, total_size
    );

    let nuevo_file = NuevoFile {
        id: file_id,
        mime,
        hash: "", // Se calculará al finalizar
        owner_id: user_id,
        status: "uploading",
        total_size: Some(total_size),
    };

    init_db_manager()
        .insertar_file(&nuevo_file)
        .context("Error al crear registro de archivo")?;

    // Crear directorio para chunks
    let chunks_dir = format!("./Privafile/Chunks/{}", file_id);
    tokio::fs::create_dir_all(&chunks_dir)
        .await
        .context("Error al crear directorio de chunks")?;

    info!("Upload iniciado exitosamente: {}", file_id);
    Ok(())
}

/// Sube un chunk individual
pub async fn upload_chunk(
    user_id: &str,
    file_id: &str,
    chunk_index: i32,
    chunk_data: Vec<u8>,
) -> Result<String> {
    let db = init_db_manager();

    // Verificar que el archivo exista y pertenezca al usuario
    let file = db.buscar_file(file_id).context("Archivo no encontrado")?;

    if file.owner_id != user_id {
        return Err(anyhow!("No autorizado"));
    }

    if file.status != "uploading" {
        return Err(anyhow!("El archivo no está en estado de subida"));
    }

    // Calcular hash del chunk
    let mut hasher = Blake2b512::new();
    hasher.update(&chunk_data);
    let chunk_hash = format!("{:x}", hasher.finalize());

    // Generar chunk_id
    let chunk_id = format!("{}_{}", file_id, chunk_index);

    // Guardar chunk en disco
    let chunk_path = format!("./Privafile/Chunks/{}/{}.stchunk", file_id, chunk_index);
    write_file(&chunk_path, &chunk_data)
        .await
        .context("Error al guardar chunk en disco")?;

    // Guardar en DB
    let nuevo_chunk = NuevoChunk {
        id: &chunk_id,
        file_id,
        chunk_index,
        hash: &chunk_hash,
        size: chunk_data.len() as i32,
        status: "uploaded",
    };

    db.insertar_chunk(&nuevo_chunk)
        .context("Error al registrar chunk en DB")?;

    info!(
        "Chunk {} subido exitosamente: {} bytes (hash: {})",
        chunk_index,
        chunk_data.len(),
        &chunk_hash[..16]
    );

    Ok(chunk_hash)
}

/// Finaliza el upload y verifica integridad
pub async fn finalize_chunked_upload(user_id: &str, file_id: &str) -> Result<(String, usize)> {
    let db = init_db_manager();

    // Verificar que el archivo exista y pertenezca al usuario
    let file = db.buscar_file(file_id).context("Archivo no encontrado")?;

    if file.owner_id != user_id {
        return Err(anyhow!("No autorizado"));
    }

    // Obtener todos los chunks
    let chunks = db
        .obtener_chunks_de_file(file_id)
        .context("Error al obtener chunks")?;

    if chunks.is_empty() {
        return Err(anyhow!("No hay chunks para este archivo"));
    }

    // Verificar que los chunks estén en orden consecutivo
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.chunk_index != i as i32 {
            error!("Falta chunk {} para el archivo {}", i, file_id);
            return Err(anyhow!("Faltan chunks o están desordenados"));
        }
    }

    // Calcular hash final (concatenando hashes de chunks)
    let mut final_hasher = Blake2b512::new();
    for chunk in &chunks {
        final_hasher.update(chunk.hash.as_bytes());
    }
    let final_hash = format!("{:x}", final_hasher.finalize());

    // Actualizar archivo con hash final y status
    db.actualizar_file_hash(file_id, &final_hash)
        .context("Error al actualizar hash del archivo")?;

    db.actualizar_file_status(file_id, "complete")
        .context("Error al actualizar status del archivo")?;

    info!(
        "Upload finalizado: {} ({} chunks, hash: {})",
        file_id,
        chunks.len(),
        &final_hash[..16]
    );

    Ok((final_hash, chunks.len()))
}

/// Descarga un archivo completo (ensamblando chunks si es necesario)
pub async fn download_file(user_id: &str, file_id: &str) -> Result<(String, Vec<u8>)> {
    // Validación de seguridad
    if file_id.contains("..") || file_id.contains('/') || file_id.contains('\\') {
        error!("Intento de path traversal detectado: {}", file_id);
        return Err(anyhow!("ID de archivo inválido"));
    }

    let db = init_db_manager();

    // Verificar que el archivo exista y pertenezca al usuario
    let file = db.buscar_file(file_id).context("Archivo no encontrado")?;

    if file.owner_id != user_id {
        warn!(
            "Usuario {} intentó acceder al archivo {} de otro usuario",
            user_id, file_id
        );
        return Err(anyhow!("No autorizado"));
    }

    // Verificar si tiene chunks
    let chunks = db.obtener_chunks_de_file(file_id)?;

    let file_data = if chunks.is_empty() {
        // Archivo sin chunks (legacy o pequeño)
        let file_path = PathBuf::from(format!("./Privafile/Uploads/{}.st", file_id));
        tokio::fs::read(&file_path)
            .await
            .context("Error al leer archivo del disco")?
    } else {
        // Ensamblar chunks
        info!(
            "Ensamblando {} chunks para el archivo {}",
            chunks.len(),
            file_id
        );

        let mut assembled_data = Vec::new();

        for chunk in chunks.iter() {
            let chunk_path = PathBuf::from(format!(
                "./Privafile/Chunks/{}/{}.stchunk",
                file_id, chunk.chunk_index
            ));

            let chunk_data = tokio::fs::read(&chunk_path)
                .await
                .with_context(|| format!("Error al leer chunk {} del disco", chunk.chunk_index))?;

            assembled_data.extend_from_slice(&chunk_data);
        }

        info!(
            "Archivo {} ensamblado exitosamente: {} bytes",
            file_id,
            assembled_data.len()
        );

        assembled_data
    };

    Ok((file.mime.clone(), file_data))
}

/// Descarga un chunk individual
pub async fn download_chunk(
    user_id: &str,
    file_id: &str,
    chunk_index: i32,
) -> Result<(Vec<u8>, String)> {
    let db = init_db_manager();

    // Verificar propiedad del archivo
    let file = db.buscar_file(file_id).context("Archivo no encontrado")?;

    if file.owner_id != user_id {
        return Err(anyhow!("No autorizado"));
    }

    // Buscar el chunk
    let chunk_id = format!("{}_{}", file_id, chunk_index);
    let chunk = db.buscar_chunk(&chunk_id).context("Chunk no encontrado")?;

    // Leer del disco
    let chunk_path = PathBuf::from(format!(
        "./Privafile/Chunks/{}/{}.stchunk",
        file_id, chunk_index
    ));

    let chunk_data = tokio::fs::read(&chunk_path)
        .await
        .context("Error al leer chunk del disco")?;

    Ok((chunk_data, chunk.hash))
}

/// Lista los archivos de un usuario
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

/// Elimina un archivo y todos sus chunks
pub async fn delete_file(user_id: &str, file_id: &str) -> Result<()> {
    if file_id.contains("..") || file_id.contains('/') || file_id.contains('\\') {
        error!("Intento de path traversal detectado en delete: {}", file_id);
        return Err(anyhow!("ID de archivo inválido"));
    }

    let db = init_db_manager();

    // Verificar propiedad
    let file = db.buscar_file(file_id).context("Archivo no encontrado")?;

    if file.owner_id != user_id {
        return Err(anyhow!("No autorizado"));
    }

    // Obtener chunks
    let chunks = db.obtener_chunks_de_file(file_id)?;

    // Eliminar chunks del disco
    for chunk in chunks.iter() {
        let chunk_path = PathBuf::from(format!(
            "./Privafile/Chunks/{}/{}.stchunk",
            file_id, chunk.chunk_index
        ));

        if let Err(e) = tokio::fs::remove_file(&chunk_path).await {
            warn!("Error al eliminar chunk del disco: {}", e);
        }
    }

    // Eliminar directorio de chunks
    let chunks_dir = PathBuf::from(format!("./Privafile/Chunks/{}", file_id));
    if let Err(e) = tokio::fs::remove_dir_all(&chunks_dir).await {
        warn!("Error al eliminar directorio de chunks: {}", e);
    }

    // Eliminar chunks de la DB
    db.borrar_chunks_de_file(file_id)
        .context("Error al eliminar chunks de DB")?;

    // Eliminar archivo de la DB
    db.borrar_file(file_id)
        .context("Error al eliminar archivo de DB")?;

    info!("Archivo {} eliminado exitosamente", file_id);
    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// Autenticación
// ═══════════════════════════════════════════════════════════════════════════

pub async fn register_user(username: &str, password: &str) -> Result<String> {
    if username.len() < 3 || username.len() > 50 {
        return Err(anyhow!("El username debe tener entre 3 y 50 caracteres"));
    }

    if password.len() < 8 {
        return Err(anyhow!("La contraseña debe tener al menos 8 caracteres"));
    }

    if !username
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(anyhow!(
            "El username solo puede contener letras, números, guiones y guiones bajos"
        ));
    }

    let db = init_db_manager();

    // Verificar que el username no exista
    if db.buscar_usuario_por_username(username).is_ok() {
        warn!("Intento de registro con username existente: {}", username);
        return Err(anyhow!("El username '{}' ya está en uso", username));
    }

    // Hashear la contraseña
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), salt.as_salt())
        .map_err(|e| anyhow!("Error al hashear la contraseña: {}", e))?
        .to_string();

    // Crear el usuario
    let user_id = Uuid::new_v4().to_string();
    let nuevo_usuario = NuevoUsuario {
        id: &user_id,
        username,
        password: &password_hash,
        b64_pubkey: None,
    };

    db.insertar_usuario(&nuevo_usuario)
        .context("Error al insertar usuario en la base de datos")?;

    info!(
        "Usuario registrado exitosamente: {} (ID: {})",
        username, user_id
    );
    Ok(user_id)
}

pub async fn authenticate_user(username: &str, password: &str) -> Result<String> {
    let db = init_db_manager();

    // Buscar usuario por username
    let usuario = db
        .buscar_usuario_por_username(username)
        .context("Usuario no encontrado")?;

    // Verificar contraseña
    let parsed_hash = PasswordHash::new(&usuario.password)
        .map_err(|e| anyhow!("Error al parsear hash de contraseña: {}", e))?;

    match Argon2::default().verify_password(password.as_bytes(), &parsed_hash) {
        Ok(_) => {
            info!(
                "Login exitoso para usuario: {} (ID: {})",
                username, usuario.id
            );
            Ok(usuario.id)
        }
        Err(_) => {
            warn!("Intento de login fallido para usuario: {}", username);
            Err(anyhow!("Credenciales inválidas"))
        }
    }
}

pub async fn user_exists(user_id: &str) -> Result<bool> {
    let db = init_db_manager();
    Ok(db.buscar_usuario(user_id).is_ok())
}
