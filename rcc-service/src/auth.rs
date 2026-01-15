use jsonwebtoken::{encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey};
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,     // Executor ID
    pub exp: usize,      // Expiration
}

pub fn create_jwt(executor_id: &str, secret: &str) -> String {
    let expiration = 10000000000; // Far future for MVP
    let claims = Claims { sub: executor_id.to_owned(), exp: expiration };
    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())).unwrap()
}
