use crate::admin::models::JwtClaims;
use crate::errors::AuthError;
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

#[derive(Clone)]
pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    pub expiration_hours: u64,
}

impl JwtManager {
    pub fn new(secret: &str, expiration_hours: u64) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
            expiration_hours,
        }
    }

    pub fn generate_token(
        &self,
        user_id: &str,
        username: &str,
        is_super_admin: bool,
    ) -> Result<String, AuthError> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiration_hours as i64);

        let claims = JwtClaims {
            sub: user_id.to_string(),
            username: username.to_string(),
            is_super_admin,
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(AuthError::from)
    }

    pub fn validate_token(&self, token: &str) -> Result<JwtClaims, AuthError> {
        let validation = Validation::new(Algorithm::HS256);

        match decode::<JwtClaims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let now = Utc::now().timestamp() as usize;
                if token_data.claims.exp < now {
                    Err(AuthError::TokenExpired)
                } else {
                    Ok(token_data.claims)
                }
            }
            Err(_) => Err(AuthError::InvalidToken),
        }
    }
}
