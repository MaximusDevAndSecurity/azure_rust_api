use serde::{Serialize, Deserialize};
use diesel::r2d2::{Pool, ConnectionManager};
use diesel::{MysqlConnection, Insertable, Queryable};
use crate::schema::users;


#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password: String, // Plaintext password
}

#[derive(Insertable)]
#[table_name = "users"]
pub struct UserForInsert {
    pub username: String,
    pub password_hash: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
  pub  sub: String,
  pub exp: usize, // Expiry timestamp
}


pub type DbPool = Pool<ConnectionManager<MysqlConnection>>;

