use actix_web::{get, delete, post, web, App, HttpResponse, HttpServer, Responder};
use serde::{Serialize, Deserialize};
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::r2d2::{self, ConnectionManager};


#[macro_use]
extern crate diesel;
mod schema;
use crate::schema::users;
use crate::schema::users::dsl::*;




#[derive(Serialize, Deserialize, Queryable)]
struct User {
    id: i32,
    username: String,
    password_hash: String,
}

#[derive(Serialize, Deserialize, Insertable)]
#[table_name = "users"]
struct NewUser {
    username: String,
    password_hash: String,
}


pub type DbPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

#[get("/users/{id}")]
async fn get_user(
    user_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let user_result = web::block(move || {
        users.filter(id.eq(*user_id))
             .first::<User>(&conn)
    })
    .await;

    match user_result {
        Ok(Ok(user)) => Ok(HttpResponse::Ok().json(user)),
        Ok(Err(diesel::result::Error::NotFound)) => Ok(HttpResponse::NotFound().json("User not found")),
        Ok(Err(_)) | Err(_) => Err(actix_web::error::ErrorInternalServerError("Internal Server Error")),
    }
}

#[post("/users")]
async fn create_user(
    user_data: web::Json<NewUser>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let new_user = web::block(move || {
        diesel::insert_into(users)
            .values(&user_data.into_inner())
            .execute(&conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Error inserting user"))?;

    Ok(HttpResponse::Ok().json("User created successfully"))
}

#[delete("/users/{id}")]
async fn delete_user(
    user_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = pool.get().expect("couldn't get db connection from pool");

    let result = web::block(move || {
        diesel::delete(users.filter(id.eq(*user_id)))
            .execute(&conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Error deleting user"))?;

    match result {
        Ok(count) if count > 0 => Ok(HttpResponse::Ok().json("User deleted")),
        Ok(_) => Ok(HttpResponse::NotFound().json("User not found")),
        Err(_) => Err(actix_web::error::ErrorInternalServerError("Internal Server Error")),
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_connection();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(get_user)
            .service(create_user)
            .service(delete_user)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
