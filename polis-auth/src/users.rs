use argon2::password_hash::SaltString;
use getrandom::getrandom;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::{DateTime, Utc};
use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UserManager {
    users: std::collections::HashMap<Uuid, User>,
    username_index: std::collections::HashMap<String, Uuid>,
    email_index: std::collections::HashMap<String, Uuid>,
}

impl UserManager {
    pub fn new() -> Self {
        let mut manager = Self {
            users: std::collections::HashMap::new(),
            username_index: std::collections::HashMap::new(),
            email_index: std::collections::HashMap::new(),
        };

        // Criar usuário admin padrão
        manager.create_default_admin();
        manager
    }

    fn create_default_admin(&mut self) {
        let admin_id = Uuid::new_v4();
        let default_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());
        let password_hash = Self::hash_password(&default_password).unwrap();

        let admin_user = User {
            id: admin_id,
            username: "admin".to_string(),
            email: "admin@polis.local".to_string(),
            password_hash,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.users.insert(admin_id, admin_user);
        self.username_index.insert("admin".to_string(), admin_id);
        self.email_index
            .insert("admin@polis.local".to_string(), admin_id);
    }

    pub async fn create_user(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<User> {
        // Verificar se o usuário já existe
        if self.username_index.contains_key(&username) {
            return Err(PolisError::Auth("Nome de usuário já existe".to_string()));
        }

        if self.email_index.contains_key(&email) {
            return Err(PolisError::Auth("Email já existe".to_string()));
        }

        let user_id = Uuid::new_v4();
        let password_hash = Self::hash_password(&password)?;

        let user = User {
            id: user_id,
            username: username.clone(),
            email: email.clone(),
            password_hash,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.users.insert(user_id, user.clone());
        self.username_index.insert(username, user_id);
        self.email_index.insert(email, user_id);

        Ok(user)
    }

    pub async fn authenticate_user(&self, username: &str, password: &str) -> Result<User> {
        let user_id = self
            .username_index
            .get(username)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        let user = self
            .users
            .get(user_id)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        if !user.is_active {
            return Err(PolisError::Auth("Usuário inativo".to_string()));
        }

        if !Self::verify_password(password, &user.password_hash)? {
            return Err(PolisError::Auth("Senha incorreta".to_string()));
        }

        Ok(user.clone())
    }

    pub async fn get_user_by_id(&self, user_id: &Uuid) -> Result<User> {
        self.users
            .get(user_id)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))
            .map(|u| u.clone())
    }

    pub async fn get_user_by_username(&self, username: &str) -> Result<User> {
        let user_id = self
            .username_index
            .get(username)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        self.get_user_by_id(user_id).await
    }

    pub async fn update_user(
        &mut self,
        user_id: &Uuid,
        username: Option<String>,
        email: Option<String>,
    ) -> Result<User> {
        let user = self
            .users
            .get_mut(user_id)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        if let Some(new_username) = username {
            if new_username != user.username && self.username_index.contains_key(&new_username) {
                return Err(PolisError::Auth("Nome de usuário já existe".to_string()));
            }

            self.username_index.remove(&user.username);
            self.username_index.insert(new_username.clone(), *user_id);
            user.username = new_username;
        }

        if let Some(new_email) = email {
            if new_email != user.email && self.email_index.contains_key(&new_email) {
                return Err(PolisError::Auth("Email já existe".to_string()));
            }

            self.email_index.remove(&user.email);
            self.email_index.insert(new_email.clone(), *user_id);
            user.email = new_email;
        }

        user.updated_at = Utc::now();
        Ok(user.clone())
    }

    pub async fn change_password(
        &mut self,
        user_id: &Uuid,
        old_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let user = self
            .users
            .get_mut(user_id)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        if !Self::verify_password(old_password, &user.password_hash)? {
            return Err(PolisError::Auth("Senha atual incorreta".to_string()));
        }

        user.password_hash = Self::hash_password(new_password)?;
        user.updated_at = Utc::now();

        Ok(())
    }

    pub async fn deactivate_user(&mut self, user_id: &Uuid) -> Result<()> {
        let user = self
            .users
            .get_mut(user_id)
            .ok_or_else(|| PolisError::Auth("Usuário não encontrado".to_string()))?;

        user.is_active = false;
        user.updated_at = Utc::now();

        Ok(())
    }

    pub async fn list_users(&self) -> Result<Vec<User>> {
        Ok(self.users.values().cloned().collect())
    }

    fn hash_password(password: &str) -> Result<String> {
        let mut salt_bytes = [0u8; 16];
        getrandom(&mut salt_bytes)
            .map_err(|e| PolisError::Auth(format!("Erro ao gerar salt: {}", e)))?;
        
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| PolisError::Auth(format!("Erro ao codificar salt: {}", e)))?;
        
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| PolisError::Auth(format!("Erro ao hash da senha: {}", e)))?;

        Ok(password_hash.to_string())
    }

    fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| PolisError::Auth(format!("Erro ao parsear hash: {}", e)))?;

        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
}
