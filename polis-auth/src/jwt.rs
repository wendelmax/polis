use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};

pub struct JwtManager {
    secret: String,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String, // Subject (user ID)
    pub username: String,
    pub permissions: Vec<String>,
    pub exp: usize,         // Expiration time
    pub iat: Option<usize>, // Issued at
}

impl JwtManager {
    pub fn new(secret: String) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());

        Self {
            secret,
            encoding_key,
            decoding_key,
        }
    }

    pub fn generate_token(&self, claims: &JwtClaims) -> Result<String> {
        let header = Header::new(Algorithm::HS256);
        let mut claims_with_iat = claims.clone();
        claims_with_iat.iat = Some(chrono::Utc::now().timestamp() as usize);

        encode(&header, &claims_with_iat, &self.encoding_key)
            .map_err(|e| PolisError::Auth(format!("Erro ao gerar token JWT: {}", e)))
    }

    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        let validation = Validation::new(Algorithm::HS256);

        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &validation)
            .map_err(|e| PolisError::Auth(format!("Erro ao validar token JWT: {}", e)))?;

        // Verificar se o token não expirou
        let now = chrono::Utc::now().timestamp() as usize;
        if token_data.claims.exp < now {
            return Err(PolisError::Auth("Token expirado".to_string()));
        }

        Ok(token_data.claims)
    }

    pub fn extract_claims(&self, token: &str) -> Result<JwtClaims> {
        self.validate_token(token)
    }

    pub fn is_token_expired(&self, token: &str) -> bool {
        match self.extract_claims(token) {
            Ok(claims) => {
                let now = chrono::Utc::now().timestamp() as usize;
                claims.exp < now
            }
            Err(_) => true,
        }
    }

    pub fn get_token_ttl(&self, token: &str) -> Result<i64> {
        let claims = self.extract_claims(token)?;
        let now = chrono::Utc::now().timestamp();
        Ok(claims.exp as i64 - now)
    }
}
