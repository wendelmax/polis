use crate::{JwtClaims, JwtManager, PermissionManager, UserManager};
use polis_core::{PolisError, Result};
use std::collections::HashMap;
use uuid::Uuid;

pub struct AuthManager {
    pub jwt_manager: JwtManager,
    pub user_manager: UserManager,
    pub permission_manager: PermissionManager,
    pub sessions: HashMap<String, UserSession>,
}

#[derive(Debug, Clone)]
pub struct UserSession {
    pub user_id: Uuid,
    pub username: String,
    pub permissions: Vec<String>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct AuthResult {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct UserInfo {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl AuthManager {
    pub fn new(jwt_secret: String) -> Self {
        Self {
            jwt_manager: JwtManager::new(jwt_secret),
            user_manager: UserManager::new(),
            permission_manager: PermissionManager::new(),
            sessions: HashMap::new(),
        }
    }

    pub async fn authenticate(&mut self, username: &str, password: &str) -> Result<AuthResult> {
        // Verificar credenciais do usuário
        let user = self
            .user_manager
            .authenticate_user(username, password)
            .await?;

        // Obter permissões do usuário
        let permissions = self
            .permission_manager
            .get_user_permissions(&user.id)
            .await?;

        // Gerar token JWT
        let claims = JwtClaims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            permissions: permissions.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            iat: Some(chrono::Utc::now().timestamp() as usize),
        };

        let token = self.jwt_manager.generate_token(&claims)?;
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        // Criar sessão
        let session = UserSession {
            user_id: user.id,
            username: user.username.clone(),
            permissions: permissions.clone(),
            expires_at,
        };

        self.sessions.insert(token.clone(), session);

        Ok(AuthResult {
            token,
            user: UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                permissions,
                created_at: user.created_at,
            },
            expires_at,
        })
    }

    pub async fn validate_token(&self, token: &str) -> Result<UserSession> {
        // Verificar se o token é válido
        let _claims = self.jwt_manager.validate_token(token)?;

        // Verificar se a sessão existe e não expirou
        if let Some(session) = self.sessions.get(token) {
            if session.expires_at > chrono::Utc::now() {
                return Ok(session.clone());
            }
        }

        Err(PolisError::Auth("Token inválido ou expirado".to_string()))
    }

    pub async fn refresh_token(&mut self, token: &str) -> Result<AuthResult> {
        let session = self.validate_token(token).await?;

        // Gerar novo token
        let claims = JwtClaims {
            sub: session.user_id.to_string(),
            username: session.username.clone(),
            permissions: session.permissions.clone(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            iat: Some(chrono::Utc::now().timestamp() as usize),
        };

        let new_token = self.jwt_manager.generate_token(&claims)?;
        let expires_at = chrono::Utc::now() + chrono::Duration::hours(24);

        // Atualizar sessão
        let new_session = UserSession {
            user_id: session.user_id,
            username: session.username.clone(),
            permissions: session.permissions.clone(),
            expires_at,
        };

        self.sessions.remove(token);
        self.sessions.insert(new_token.clone(), new_session);

        // Obter informações do usuário
        let user = self.user_manager.get_user_by_id(&session.user_id).await?;

        Ok(AuthResult {
            token: new_token,
            user: UserInfo {
                id: user.id,
                username: user.username,
                email: user.email,
                permissions: session.permissions,
                created_at: user.created_at,
            },
            expires_at,
        })
    }

    pub async fn logout(&mut self, token: &str) -> Result<()> {
        self.sessions.remove(token);
        Ok(())
    }

    pub async fn check_permission(&self, token: &str, permission: &str) -> Result<bool> {
        let session = self.validate_token(token).await?;
        Ok(session.permissions.contains(&permission.to_string()))
    }

    pub async fn get_user_info(&self, token: &str) -> Result<UserInfo> {
        let session = self.validate_token(token).await?;
        let user = self.user_manager.get_user_by_id(&session.user_id).await?;

        Ok(UserInfo {
            id: user.id,
            username: user.username,
            email: user.email,
            permissions: session.permissions,
            created_at: user.created_at,
        })
    }
}
