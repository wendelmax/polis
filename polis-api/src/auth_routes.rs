use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::{Body, Method, Request, Response, StatusCode};
use polis_auth::AuthManager;
use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: UserInfo,
    pub expires_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub username: String,
    pub email: String,
    pub permissions: Vec<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogoutRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

pub struct AuthRoutes {
    auth_manager: Arc<RwLock<AuthManager>>,
}

impl AuthRoutes {
    pub fn new(auth_manager: Arc<RwLock<AuthManager>>) -> Self {
        Self { auth_manager }
    }

    pub async fn handle_request(&self, req: Request<Body>) -> Result<Response<Body>> {
        match (req.method(), req.uri().path()) {
            (&Method::POST, "/auth/login") => self.handle_login(req).await,
            (&Method::POST, "/auth/refresh") => self.handle_refresh(req).await,
            (&Method::POST, "/auth/logout") => self.handle_logout(req).await,
            (&Method::GET, "/auth/me") => self.handle_me(req).await,
            _ => Ok(Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Endpoint não encontrado"))
                .unwrap()),
        }
    }

    async fn handle_login(&self, mut req: Request<Body>) -> Result<Response<Body>> {
        // Ler body da requisição
        let body_bytes = hyper::body::to_bytes(req.body_mut())
            .await
            .map_err(|e| PolisError::Api(format!("Erro ao ler body: {}", e)))?;

        let login_req: LoginRequest = serde_json::from_slice(&body_bytes)
            .map_err(|e| PolisError::Api(format!("Erro ao parsear JSON: {}", e)))?;

        // Autenticar usuário
        let mut auth_manager = self.auth_manager.write().await;
        let auth_result = auth_manager
            .authenticate(&login_req.username, &login_req.password)
            .await?;

        let response = LoginResponse {
            token: auth_result.token,
            user: UserInfo {
                id: auth_result.user.id.to_string(),
                username: auth_result.user.username,
                email: auth_result.user.email,
                permissions: auth_result.user.permissions,
                created_at: auth_result.user.created_at.to_rfc3339(),
            },
            expires_at: auth_result.expires_at.to_rfc3339(),
        };

        let json_response = serde_json::to_string(&response)
            .map_err(|e| PolisError::Api(format!("Erro ao serializar JSON: {}", e)))?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json_response))
            .unwrap())
    }

    async fn handle_refresh(&self, mut req: Request<Body>) -> Result<Response<Body>> {
        // Ler body da requisição
        let body_bytes = hyper::body::to_bytes(req.body_mut())
            .await
            .map_err(|e| PolisError::Api(format!("Erro ao ler body: {}", e)))?;

        let refresh_req: RefreshRequest = serde_json::from_slice(&body_bytes)
            .map_err(|e| PolisError::Api(format!("Erro ao parsear JSON: {}", e)))?;

        // Renovar token
        let mut auth_manager = self.auth_manager.write().await;
        let auth_result = auth_manager.refresh_token(&refresh_req.token).await?;

        let response = LoginResponse {
            token: auth_result.token,
            user: UserInfo {
                id: auth_result.user.id.to_string(),
                username: auth_result.user.username,
                email: auth_result.user.email,
                permissions: auth_result.user.permissions,
                created_at: auth_result.user.created_at.to_rfc3339(),
            },
            expires_at: auth_result.expires_at.to_rfc3339(),
        };

        let json_response = serde_json::to_string(&response)
            .map_err(|e| PolisError::Api(format!("Erro ao serializar JSON: {}", e)))?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json_response))
            .unwrap())
    }

    async fn handle_logout(&self, mut req: Request<Body>) -> Result<Response<Body>> {
        // Ler body da requisição
        let body_bytes = hyper::body::to_bytes(req.body_mut())
            .await
            .map_err(|e| PolisError::Api(format!("Erro ao ler body: {}", e)))?;

        let logout_req: LogoutRequest = serde_json::from_slice(&body_bytes)
            .map_err(|e| PolisError::Api(format!("Erro ao parsear JSON: {}", e)))?;

        // Fazer logout
        let mut auth_manager = self.auth_manager.write().await;
        auth_manager.logout(&logout_req.token).await?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(r#"{"message": "Logout realizado com sucesso"}"#))
            .unwrap())
    }

    async fn handle_me(&self, req: Request<Body>) -> Result<Response<Body>> {
        // Extrair token do header Authorization
        let auth_header = req
            .headers()
            .get(AUTHORIZATION)
            .ok_or_else(|| PolisError::Auth("Header Authorization não encontrado".to_string()))?;

        let auth_str = auth_header
            .to_str()
            .map_err(|_| PolisError::Auth("Header Authorization inválido".to_string()))?;

        if !auth_str.starts_with("Bearer ") {
            return Err(PolisError::Auth(
                "Formato de token inválido. Use 'Bearer <token>'".to_string(),
            ));
        }

        let token = &auth_str[7..];

        // Obter informações do usuário
        let auth_manager = self.auth_manager.read().await;
        let user_info = auth_manager.get_user_info(token).await?;

        let response = UserInfo {
            id: user_info.id.to_string(),
            username: user_info.username,
            email: user_info.email,
            permissions: user_info.permissions,
            created_at: user_info.created_at.to_rfc3339(),
        };

        let json_response = serde_json::to_string(&response)
            .map_err(|e| PolisError::Api(format!("Erro ao serializar JSON: {}", e)))?;

        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(CONTENT_TYPE, "application/json")
            .body(Body::from(json_response))
            .unwrap())
    }
}

pub fn create_error_response(status: StatusCode, error: &str, message: &str) -> Response<Body> {
    let error_response = ErrorResponse {
        error: error.to_string(),
        message: message.to_string(),
    };

    let json_response = serde_json::to_string(&error_response)
        .unwrap_or_else(|_| format!(r#"{{"error": "{}", "message": "{}"}}"#, error, message));

    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, "application/json")
        .body(Body::from(json_response))
        .unwrap()
}
