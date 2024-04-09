mod models;
mod routes;
mod schema;

use actix_web::{App, HttpServer, web};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use routes::config_services;
use models::DbPool;

#[macro_use]
extern crate diesel;



pub fn establish_connection() -> DbPool {
    dotenv::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);
    Pool::builder().build(manager).expect("Failed to create pool.")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let pool = establish_connection();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .configure(config_services)
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
