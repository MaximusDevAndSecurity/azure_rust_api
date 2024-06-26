use actix_web::{web, HttpResponse, get, post, delete};
use crate::models::{DbPool, User, NewUser, UserForInsert};
use diesel::prelude::*;
use bcrypt::DEFAULT_COST;
use crate::auth::create_token;
use crate::diesel::associations::HasTable;
use diesel::insert_into;
use crate::schema::users::dsl::*;
use crate::schema::users::dsl::users;  


pub fn config_auth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(get_user);
    cfg.service(delete_user);
    
}

pub fn config_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(login);
    cfg.service(create_user);
    // other public routes here
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

#[post("/register")]
async fn create_user(
    user_data: web::Json<NewUser>,
    pool: web::Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let conn = pool.get().map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get DB connection from pool"))?;

    let hashed_password = bcrypt::hash(&user_data.password, DEFAULT_COST)
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to hash password"))?;

    let user_for_insert = UserForInsert {
        username: user_data.username.clone(),
        password_hash: hashed_password,
    };

    let result = web::block(move || {
        insert_into(users)  
    .values(&user_for_insert)
    .execute(&conn)
    })
    .await
    .map_err(|_| actix_web::error::ErrorInternalServerError("Error inserting user into database"))?;

    match result {
        Ok(count) if count > 0 => Ok(HttpResponse::Created().json("User created successfully")),
        Ok(_) => Err(actix_web::error::ErrorInternalServerError("No user was created")),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(format!("Database error: {}", e))),
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


#[post("/login")]
async fn login(user_data: web::Json<NewUser>, pool: web::Data<DbPool>) -> HttpResponse {

    let conn = pool.get().expect("couldn't get db connection from pool");

    // Find the user by username
    let user = users
        .filter(username.eq(&user_data.username))
        .first::<User>(&conn);

    match user {
        Ok(u) => {
            // Verify password (ensure passwords are hashed in the database)
            if bcrypt::verify(&user_data.password, &u.password_hash).unwrap_or(false) {
                // Passwords match, proceed to create token
                let token = create_token(&u.username);  // Assuming create_token returns a String
                HttpResponse::Ok().json({"token" ; token})
            } else {
                // Passwords do not match
                HttpResponse::Unauthorized().body("Invalid username or password")
            }
        }
        Err(_) => HttpResponse::Unauthorized().body("Invalid username or password"),
    }
}