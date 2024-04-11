use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};
use chrono::{Utc, Duration};
use crate::models::Claims;


pub fn create_token(username: &str) -> String {
    dotenv::dotenv().ok();
    let secret_key = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
    let secret_key_bytes = secret_key.as_bytes(); // Convert String to byte slice

    let expiration = Utc::now()
        .checked_add_signed(Duration::minutes(60))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims { sub: username.to_owned(), exp: expiration };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret_key_bytes)).unwrap() // Use byte slice here: reminder
}


pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims, ()> {
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &Validation::default())
        .map(|data| data.claims)
        .map_err(|_| ())
}
