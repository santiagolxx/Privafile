use crate::core::File;
use crate::core::database::init_db_manager;
use crate::core::structs::NuevoFile;
use crate::core::structs::NuevoUsuario;
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

/// Registra un nuevo usuario en el sistema
///
/// # Validaciones
/// - Username único (no puede existir)
/// - Username: 3-50 caracteres alfanuméricos
/// - Password: mínimo 8 caracteres
///
/// # Seguridad
/// - Password hasheado con Argon2id
/// - Salt aleatorio por usuario
pub async fn register_user(username: &str, password: &str) -> Result<String> {
    // Validaciones de entrada
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
    // Nota: Diesel no tiene un método directo para buscar por username,
    // así que haremos una query custom
    let existing = db.buscar_usuario_por_username(username);
    if existing.is_ok() {
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
        b64_pubkey: None, // Para futuras implementaciones de E2E encryption
    };

    db.insertar_usuario(&nuevo_usuario)
        .context("Error al insertar usuario en la base de datos")?;

    info!(
        "Usuario registrado exitosamente: {} (ID: {})",
        username, user_id
    );
    Ok(user_id)
}

/// Autentica un usuario verificando sus credenciales
///
/// # Validaciones
/// - Usuario debe existir
/// - Password debe coincidir con el hash almacenado
///
/// # Retorna
/// ID del usuario si las credenciales son correctas
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

/// Verifica si un usuario existe por ID
pub async fn user_exists(user_id: &str) -> Result<bool> {
    let db = init_db_manager();
    Ok(db.buscar_usuario(user_id).is_ok())
}
