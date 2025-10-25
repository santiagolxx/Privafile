use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use reqwest::{
    Client,
    header::{AUTHORIZATION, CONTENT_RANGE, HeaderMap, HeaderValue},
};
use ring::{digest, pbkdf2};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const CHUNK_SIZE: usize = 10 * 1024 * 1024; // 10 MB
const SALT: &[u8] = b"privafile-salt";
const TOKEN_TTL_MINUTES: i64 = 30;

#[derive(Parser)]
#[command(name = "privafile-cli")]
#[command(about = "Secure file storage CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Upload a file
    Upload {
        #[arg(short, long)]
        file: PathBuf,

        #[arg(short, long)]
        password: Option<String>,
    },

    /// Download a file
    Download {
        #[arg(short, long)]
        file_id: String,

        #[arg(short, long)]
        password: Option<String>,

        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Login and store encrypted token
    Login {
        #[arg(short, long)]
        username: String,

        #[arg(short, long)]
        password: String,

        #[arg(long)]
        server_url: Option<String>,
    },

    /// Configure CLI settings
    Config {
        #[arg(short, long)]
        server_url: Option<String>,

        #[arg(long)]
        list: bool,
    },

    /// Check authentication status
    Status,
}

#[derive(Serialize, Deserialize, Clone)]
struct Config {
    server_url: String,
}

#[derive(Serialize, Deserialize)]
struct InitUploadRequest {
    file_id: String,
    total_chunks: usize,
    total_size: usize,
    mime: String,
    file_name: String,
}

#[derive(Serialize, Deserialize)]
struct UploadChunkResponse {
    success: bool,
    chunk_index: usize,
}

#[derive(Serialize, Deserialize)]
struct FinalizeUploadRequest {
    file_id: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize, Deserialize)]
struct LoginResponse {
    token: String,
}

#[derive(Serialize, Deserialize)]
struct StoredToken {
    encrypted_token: Vec<u8>,
    created_at: i64,
}

struct PrivafileClient {
    client: Client,
    config: Config,
}

impl PrivafileClient {
    fn new(config: Config) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }

    fn get_workspace_dir() -> PathBuf {
        Path::new("/tmp/privafile-cli").to_path_buf()
    }

    fn get_config_path() -> PathBuf {
        Self::get_workspace_dir().join("config.json")
    }

    fn get_tokens_dir() -> PathBuf {
        Self::get_workspace_dir().join("tokens")
    }

    fn get_downloads_dir() -> PathBuf {
        Self::get_workspace_dir().join("downloads")
    }

    fn ensure_directories() -> Result<(), Box<dyn std::error::Error>> {
        let dirs = [
            Self::get_workspace_dir(),
            Self::get_tokens_dir(),
            Self::get_downloads_dir(),
        ];

        for dir in dirs {
            fs::create_dir_all(&dir)?;
        }
        Ok(())
    }

    fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        if config_path.exists() {
            let config_data = fs::read_to_string(config_path)?;
            Ok(serde_json::from_str(&config_data)?)
        } else {
            // Configuración por defecto
            let config = Config {
                server_url: "http://localhost:3000".to_string(),
            };
            Self::save_config(&config)?;
            Ok(config)
        }
    }

    fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::get_config_path();
        let config_data = serde_json::to_string_pretty(config)?;
        fs::write(config_path, config_data)?;
        Ok(())
    }

    fn derive_master_key(password: &str) -> Aes256Gcm {
        let mut key = [0u8; 32];
        let iterations = NonZeroU32::new(100000).unwrap();

        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            iterations,
            SALT,
            password.as_bytes(),
            &mut key,
        );

        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key))
    }

    fn encrypt_data(
        data: &[u8],
        cipher: &Aes256Gcm,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let ciphertext = cipher
            .encrypt(nonce, data)
            .map_err(|e| format!("Encryption failed: {}", e))?;
        Ok(ciphertext)
    }

    fn decrypt_data(
        encrypted_data: &[u8],
        cipher: &Aes256Gcm,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let nonce = Nonce::from_slice(&[0u8; 12]);
        let plaintext = cipher
            .decrypt(nonce, encrypted_data)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        Ok(plaintext)
    }

    fn save_encrypted_token(token: &str, password: &str) -> Result<(), Box<dyn std::error::Error>> {
        let master_key = Self::derive_master_key(password);
        let encrypted_token = Self::encrypt_data(token.as_bytes(), &master_key)?;

        let stored_token = StoredToken {
            encrypted_token,
            created_at: Utc::now().timestamp(),
        };

        let token_path = Self::get_tokens_dir().join("token.enc");
        let token_data = serde_json::to_vec(&stored_token)?;
        fs::write(token_path, token_data)?;

        // También guardar el token desencriptado temporalmente
        let decrypted_token_path = Self::get_tokens_dir().join("token.dec");
        fs::write(decrypted_token_path, token)?;

        Ok(())
    }

    fn load_token(password: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
        // Primero intentar cargar el token desencriptado si es reciente
        let decrypted_token_path = Self::get_tokens_dir().join("token.dec");
        if decrypted_token_path.exists() {
            if let Ok(metadata) = fs::metadata(&decrypted_token_path) {
                if let Ok(modified) = metadata.modified() {
                    let duration = modified.elapsed().unwrap_or_default();
                    if duration.as_secs() < (TOKEN_TTL_MINUTES * 60) as u64 {
                        let token = fs::read_to_string(decrypted_token_path)?;
                        return Ok(token);
                    }
                }
            }
        }

        // Si el token desencriptado expiró o no existe, desencriptar
        let password = password.ok_or("Password required to decrypt token")?;
        let token_path = Self::get_tokens_dir().join("token.enc");
        let token_data = fs::read(token_path)?;
        let stored_token: StoredToken = serde_json::from_slice(&token_data)?;

        let master_key = Self::derive_master_key(password);
        let decrypted_token_bytes = Self::decrypt_data(&stored_token.encrypted_token, &master_key)?;
        let token = String::from_utf8(decrypted_token_bytes)?;

        // Guardar temporalmente el token desencriptado
        fs::write(decrypted_token_path, &token)?;

        Ok(token)
    }

    async fn login(
        &self,
        username: &str,
        password: &str,
        server_url: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let actual_server_url = server_url
            .clone()
            .unwrap_or_else(|| self.config.server_url.clone());

        let login_url = format!("{}/api/auth/login", actual_server_url);

        let request = LoginRequest {
            username: username.to_string(),
            password: password.to_string(),
        };

        let response = self.client.post(&login_url).json(&request).send().await?;

        if !response.status().is_success() {
            return Err("Login failed".into());
        }

        let login_response: LoginResponse = response.json().await?;

        // Guardar token encriptado
        Self::save_encrypted_token(&login_response.token, password)?;

        // Actualizar config si se proporcionó nueva URL
        if let Some(url) = server_url {
            let mut new_config = self.config.clone();
            new_config.server_url = url;
            Self::save_config(&new_config)?;
        }

        println!("✅ Login successful! Token stored securely.");
        Ok(())
    }

    async fn upload_file(
        &self,
        file_path: PathBuf,
        password: Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let token = Self::load_token(password.as_deref())?;

        println!("📁 Reading file: {:?}", file_path);

        // Obtener metadata del archivo
        let metadata = fs::metadata(&file_path)?;
        let file_size = metadata.len() as usize;
        let total_chunks = (file_size + CHUNK_SIZE - 1) / CHUNK_SIZE;
        let file_name = file_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        println!(
            "📦 File size: {} bytes, Chunks: {}",
            file_size, total_chunks
        );

        // 1. Generar file_id único
        let file_id = Uuid::new_v4().to_string();

        // 2. Derivar master key del password
        let master_key_password = password.ok_or("Password required for encryption")?;
        let master_key = Self::derive_master_key(&master_key_password);

        // 3. Iniciar upload con el campo total_size requerido
        let init_url = format!("{}/api/files/upload/init", self.config.server_url);
        let init_request = InitUploadRequest {
            file_id: file_id.clone(),
            total_chunks,
            total_size: file_size, // <- Campo requerido
            mime: "application/octet-stream".to_string(),
            file_name, // <- Nombre del archivo
        };

        let response = self
            .client
            .post(&init_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(&init_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Server error: {}", error_text);
            return Err("Failed to init upload".into());
        }

        println!("🚀 Upload initialized: {}", file_id);

        // 4. Subir chunks leyendo directamente del archivo
        let mut file = File::open(&file_path)?;

        for chunk_index in 0..total_chunks {
            let start = chunk_index * CHUNK_SIZE;
            let end = std::cmp::min(start + CHUNK_SIZE, file_size);
            let chunk_size = end - start;

            // Leer solo el chunk actual
            let mut chunk_data = vec![0u8; chunk_size];
            file.read_exact(&mut chunk_data)?;

            // Encriptar chunk
            let encrypted_chunk = Self::encrypt_data(&chunk_data, &master_key)?;

            // Subir chunk - CORREGIDO: usar query parameters como espera el servidor
            let chunk_url = format!(
                "{}/api/files/upload/chunk?file_id={}&chunk_index={}",
                self.config.server_url, file_id, chunk_index
            );

            let response = self
                .client
                .post(&chunk_url)
                .header(AUTHORIZATION, format!("Bearer {}", token))
                .body(encrypted_chunk)
                .send()
                .await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                eprintln!("❌ Chunk upload error: {}", error_text);
                return Err(format!("Failed to upload chunk {}", chunk_index).into());
            }

            let upload_response: UploadChunkResponse = response.json().await?;
            println!("✅ Uploaded chunk {}/{}", chunk_index + 1, total_chunks);
        }

        // 5. Finalizar upload
        let finalize_url = format!("{}/api/files/upload/finalize", self.config.server_url);
        let finalize_request = FinalizeUploadRequest {
            file_id: file_id.clone(),
        };

        let response = self
            .client
            .post(&finalize_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .json(&finalize_request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Finalize error: {}", error_text);
            return Err("Failed to finalize upload".into());
        }

        println!("🎉 Upload completed! File ID: {}", file_id);
        Ok(())
    }

    async fn download_file(
        &self,
        file_id: String,
        password: Option<String>,
        output: Option<PathBuf>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let token = Self::load_token(password.as_deref())?;
        let master_key_password = password.ok_or("Password required for decryption")?;
        let master_key = Self::derive_master_key(&master_key_password);

        println!("📥 Downloading file: {}", file_id);

        let download_url = format!("{}/api/files/download/{}", self.config.server_url, file_id);

        let response = self
            .client
            .get(&download_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            eprintln!("❌ Download error: {}", error_text);
            return Err("Download failed".into());
        }

        let encrypted_data = response.bytes().await?;

        // Guardar archivo encriptado en /tmp/privafile-cli/downloads/
        let encrypted_filename = format!("{}.encrypted", file_id);
        let encrypted_path = Self::get_downloads_dir().join(&encrypted_filename);
        fs::write(&encrypted_path, &encrypted_data)?;
        println!("💾 Encrypted file saved to: {:?}", encrypted_path);

        // Desencriptar
        let decrypted_data = Self::decrypt_data(&encrypted_data, &master_key)?;

        // Guardar archivo desencriptado
        let output_path = output
            .unwrap_or_else(|| Self::get_downloads_dir().join(format!("{}.decrypted", file_id)));
        let mut output_file = File::create(&output_path)?;
        output_file.write_all(&decrypted_data)?;

        println!("🔓 Decrypted file saved to: {:?}", output_path);
        Ok(())
    }

    fn check_status(&self) -> Result<(), Box<dyn std::error::Error>> {
        let decrypted_token_path = Self::get_tokens_dir().join("token.dec");

        if decrypted_token_path.exists() {
            if let Ok(metadata) = fs::metadata(&decrypted_token_path) {
                if let Ok(modified) = metadata.modified() {
                    let duration = modified.elapsed().unwrap_or_default();
                    let remaining = (TOKEN_TTL_MINUTES * 60) as u64 - duration.as_secs();

                    if remaining > 0 {
                        let minutes = remaining / 60;
                        let seconds = remaining % 60;
                        println!(
                            "✅ Authenticated - Token valid for {}m {}s",
                            minutes, seconds
                        );
                        return Ok(());
                    }
                }
            }
        }

        println!("❌ Not authenticated or token expired");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Crear directorios necesarios
    PrivafileClient::ensure_directories()?;

    let cli = Cli::parse();
    let config = PrivafileClient::load_config()?;
    let client = PrivafileClient::new(config);

    match cli.command {
        Commands::Login {
            username,
            password,
            server_url,
        } => {
            client.login(&username, &password, server_url).await?;
        }

        Commands::Upload { file, password } => {
            client.upload_file(file, password).await?;
        }

        Commands::Download {
            file_id,
            password,
            output,
        } => {
            client.download_file(file_id, password, output).await?;
        }

        Commands::Config { server_url, list } => {
            if list {
                let config = PrivafileClient::load_config()?;
                println!("Server URL: {}", config.server_url);
            } else if let Some(url) = server_url {
                let new_config = Config { server_url: url };
                PrivafileClient::save_config(&new_config)?;
                println!("✅ Configuration updated");
            }
        }

        Commands::Status => {
            client.check_status()?;
        }
    }

    Ok(())
}
