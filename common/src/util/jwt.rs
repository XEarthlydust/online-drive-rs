use crate::module::error::AppError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

lazy_static! {
    static ref DECODE_KEY: DecodingKey =
        DecodingKey::from_rsa_pem(include_bytes!("../../../keys/public.key"))
            .expect("public keys parse failed");
    static ref ENCODE_KEY: EncodingKey =
        EncodingKey::from_rsa_pem(include_bytes!("../../../keys/private.key"))
            .expect("private keys parse failed");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub uid: Uuid,
    pub user_role: String,
    exp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Operation {
    ItemCreate = 1,
    FromFileStartUpload = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub uid: Uuid,
    pub id: String,
    pub data: Option<String>,
    pub operation: Operation,
    exp: u64,
}

// 创建JWT
pub fn create_jwt(user_id: Uuid, user_role: String, exp_min: u64) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let claims = Claims {
        uid: user_id,
        user_role,
        exp: now + exp_min * 60, // 设置JWT有效期
    };

    encode(&Header::new(Algorithm::RS256), &claims, &ENCODE_KEY)
        .map_err(|_e| AppError::TokenCreateError)
}

// 验证JWT
pub fn validate_jwt(token: &str) -> Result<Claims, AppError> {
    let validation = Validation::new(Algorithm::RS256);
    let token_data =
        decode::<Claims>(token, &DECODE_KEY, &validation).map_err(|_e| AppError::TokenInvalid)?;
    Ok(token_data.claims)
}

pub fn create_payload(
    uid: Uuid,
    id: String,
    data: Option<String>,
    operation: Operation,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let payload = Payload {
        uid,
        id,
        data,
        operation,
        exp: now + 60 * 60 * 2,
    };
    encode(&Header::new(Algorithm::RS256), &payload, &ENCODE_KEY)
        .map_err(|_e| AppError::PayloadCreateError)
}

pub fn validate_payload(token: &str) -> Result<Payload, AppError> {
    let validation = Validation::new(Algorithm::RS256);
    let token_data = decode::<Payload>(token, &DECODE_KEY, &validation)
        .map_err(|_e| AppError::PayloadInvalid)?;
    Ok(token_data.claims)
}
