use diesel::prelude::{Insertable, Queryable};

use crate::core::database::schema::{files, usuarios};

#[derive(Queryable, Debug)]
pub struct Usuario {
    pub id: String,
    pub username: String,
    pub password: String,
    pub b64_pubkey: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = usuarios)]
pub struct NuevoUsuario<'a> {
    pub id: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub b64_pubkey: Option<&'a str>,
}

#[derive(Queryable, Debug)]
pub struct File {
    pub id: String,
    pub mime: String,
    pub hash: String,
    pub owner_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = files)]
pub struct NuevoFile<'a> {
    pub id: &'a str,
    pub mime: &'a str,
    pub hash: &'a str,
    pub owner_id: &'a str,
}
