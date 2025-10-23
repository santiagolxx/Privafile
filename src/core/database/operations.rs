use crate::core::database::schema::{files, usuarios};
use crate::core::getters::db_url;
use crate::core::structs::{File, NuevoFile, NuevoUsuario, Usuario};
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

    // -------------------
    // Usuarios CRUD
    // -------------------
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

    pub fn borrar_usuario(&self, user_id: &str) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::delete(usuarios::table.find(user_id)).execute(&mut conn)
    }

    // -------------------
    // Files CRUD
    // -------------------
    pub fn insertar_file(&self, nuevo: &NuevoFile) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::insert_into(files::table)
            .values(nuevo)
            .execute(&mut conn)
    }

    pub fn borrar_file(&self, file_id: &str) -> Result<usize, diesel::result::Error> {
        let mut conn = self.get_conn();
        diesel::delete(files::table.find(file_id)).execute(&mut conn)
    }

    // -------------------
    // Obtener archivos de un usuario
    // -------------------
    /// Obtiene los archivos de un usuario.
    /// `mime_filtro` permite filtrar por tipo MIME opcional.
    /// `limite` limita la cantidad de resultados opcionalmente.
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
}

pub static DB_MANAGER: OnceCell<DbManager> = OnceCell::new();

pub fn init_db_manager() -> &'static DbManager {
    DB_MANAGER.get_or_init(|| DbManager::new(db_url().as_str()))
}

/// Devuelve la referencia al DbManager global.
pub fn get_db_manager() -> &'static DbManager {
    DB_MANAGER.get().expect("DbManager no inicializado")
}
