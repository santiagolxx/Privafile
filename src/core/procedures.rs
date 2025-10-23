use crate::core::database::init_db_manager;
use crate::core::database::schema::usuarios::id;
use crate::core::structs::{File, NuevoFile};
use crate::core::utils::write_file;
use anyhow::Result;
use blake2::{Blake2b512, Digest};
use uuid::Uuid;

pub async fn upload_file(user_id: &str, mime: &str, file_content: Vec<u8>) -> Result<()> {
    let file_id = Uuid::new_v4().to_string();

    let mut hasher = Blake2b512::new();
    hasher.update(&file_content);
    let hash = format!("{:x}", hasher.finalize());

    let nuevo_file = NuevoFile {
        id: &file_id,
        mime,
        hash: &hash,
        owner_id: user_id,
    };

    init_db_manager().insertar_file(&nuevo_file)?;
    write_file(format!("./Privafile/Uploads/{}.st", file_id), &file_content).await?;
    Ok(())
}
