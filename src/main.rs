mod models;
mod routes;
mod schema;
mod auth;
mod middleware;


use actix_web::{App, HttpServer, web};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use routes::{config_auth_routes, config_public_routes};
use models::DbPool;
use crate::middleware::JwtValidator;
use actix_web::middleware::Logger;


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
    env_logger::init();
    println!("Starting server...");
    let pool = establish_connection();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(pool.clone())) // Provide pool to all routes
            .configure(config_public_routes) // Public routes (no JWT validation)
            .service(
                web::scope("/api") // Routes requiring authentication
                    .wrap(JwtValidator)
                    .configure(config_auth_routes)
            )
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}
