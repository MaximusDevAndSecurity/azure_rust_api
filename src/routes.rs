use actix_web::{web, HttpResponse, get, post, delete};
use crate::models::{DbPool, User, NewUser, UserForInsert};
use diesel::prelude::*;
use bcrypt::DEFAULT_COST;
use crate::schema::users::dsl::*;


pub fn config_services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(create_user);
    cfg.service(delete_user);
    // other routes here
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

    // Hash the plaintext password
    let hashed_password = bcrypt::hash(&user_data.password, DEFAULT_COST)
        .expect("Failed to hash password");

    // Create an instance for insertion, with the hashed password
    let user_for_insert = UserForInsert {
        username: user_data.username.clone(),
        password_hash: hashed_password,
    };

    // Insert the user into the database
    let new_user_result = web::block(move || {
        diesel::insert_into(users)
            .values(&user_for_insert)
            .execute(&conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Error inserting user"));

    match new_user_result {
        Ok(_) => Ok(HttpResponse::Ok().json("User created successfully")),
        Err(e) => Err(e),
    }
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