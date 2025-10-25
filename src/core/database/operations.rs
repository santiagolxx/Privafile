use crate::core::database::schema::{chunks, files, usuarios};
use crate::core::db_url;
use crate::core::structs::{Chunk, File, NuevoChunk, NuevoFile, NuevoUsuario, Usuario};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sqlite::SqliteConnection;
use once_cell::sync::OnceCell;

pub struct DbManager {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl DbManager {
    pub fn new(database_url: &str) -> Self {
        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder().build(manager).expect("Error creando pool");
        DbManager { pool }
    }

    fn get_conn(&self) -> diesel::r2d2::PooledConnection<ConnectionManager<SqliteConnection>> {
        self.pool.get().unwrap()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Usuarios CRUD
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insertar_usuario(&self, nuevo: &NuevoUsuario) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::insert_into(usuarios::table)
            .values(nuevo)
            .execute(&mut conn)
    }

    pub fn buscar_usuario(&self, user_id: &str) -> Result<Usuario, diesel::result::Error> {
        let mut conn = self.get_conn();
        usuarios::table.find(user_id).first(&mut conn)
    }

    pub fn buscar_usuario_por_username(
        &self,
        user_name: &str,
    ) -> Result<Usuario, diesel::result::Error> {
        let mut conn = self.get_conn();
        usuarios::table
            .filter(usuarios::username.eq(user_name))
            .first(&mut conn)
    }

    pub fn borrar_usuario(&self, user_id: &str) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::delete(usuarios::table.find(user_id)).execute(&mut conn)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Files CRUD
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insertar_file(&self, nuevo: &NuevoFile) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::insert_into(files::table)
            .values(nuevo)
            .execute(&mut conn)
    }

    pub fn buscar_file(&self, file_id: &str) -> Result<File, diesel::result::Error> {
        let mut conn = self.get_conn();
        files::table.find(file_id).first(&mut conn)
    }

    pub fn borrar_file(&self, file_id: &str) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::delete(files::table.find(file_id)).execute(&mut conn)
    }

    pub fn actualizar_file_hash(
        &self,
        file_id: &str,
        hash: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::update(files::table.find(file_id))
            .set(files::hash.eq(hash))
            .execute(&mut conn)
    }

    pub fn actualizar_file_status(
        &self,
        file_id: &str,
        status: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::update(files::table.find(file_id))
            .set(files::status.eq(status))
            .execute(&mut conn)
    }

    pub fn obtener_files_de_usuario(
        &self,
        user_id: &str,
        mime_filtro: Option<&str>,
        limite: Option<i64>,
    ) -> Result<Vec<File>, diesel::result::Error> {
        let mut conn = self.get_conn();
        let mut query = files::table
            .filter(files::owner_id.eq(user_id))
            .into_boxed();

        if let Some(mime) = mime_filtro {
            query = query.filter(files::mime.eq(mime));
        }

        if let Some(lim) = limite {
            query = query.limit(lim);
        }

        query.load::<File>(&mut conn)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Chunks CRUD
    // ═══════════════════════════════════════════════════════════════════════

    pub fn insertar_chunk(&self, nuevo: &NuevoChunk) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::insert_into(chunks::table)
            .values(nuevo)
            .execute(&mut conn)
    }

    pub fn buscar_chunk(&self, chunk_id: &str) -> Result<Chunk, diesel::result::Error> {
        let mut conn = self.get_conn();
        chunks::table.find(chunk_id).first(&mut conn)
    }

    pub fn obtener_chunks_de_file(
        &self,
        file_id: &str,
    ) -> Result<Vec<Chunk>, diesel::result::Error> {
        let mut conn = self.get_conn();
        chunks::table
            .filter(chunks::file_id.eq(file_id))
            .order(chunks::chunk_index.asc())
            .load::<Chunk>(&mut conn)
    }

    pub fn contar_chunks_de_file(&self, file_id: &str) -> Result<i64, diesel::result::Error> {
        use diesel::dsl::count;
        let mut conn = self.get_conn();
        chunks::table
            .filter(chunks::file_id.eq(file_id))
            .select(count(chunks::id))
            .first(&mut conn)
    }

    pub fn actualizar_chunk_status(
        &self,
        chunk_id: &str,
        status: &str,
    ) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::update(chunks::table.find(chunk_id))
            .set(chunks::status.eq(status))
            .execute(&mut conn)
    }

    pub fn borrar_chunks_de_file(&self, file_id: &str) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::delete(chunks::table.filter(chunks::file_id.eq(file_id))).execute(&mut conn)
    }
}

pub static DB_MANAGER: OnceCell<DbManager> = OnceCell::new();

pub fn init_db_manager() -> &'static DbManager {
    DB_MANAGER.get_or_init(|| DbManager::new(db_url().as_str()))
}

pub fn get_db_manager() -> &'static DbManager {
    DB_MANAGER.get().expect("DbManager no inicializado")
}
