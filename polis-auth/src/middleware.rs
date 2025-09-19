use crate::UserSession;
use hyper::header::AUTHORIZATION;
use hyper::{Request, Response, StatusCode};
use hyper::body::Bytes;
use polis_core::{PolisError, Result};

pub fn extract_token_from_request(req: &Request<Bytes>) -> Result<String> {
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

    Ok(auth_str[7..].to_string())
}

pub fn get_user_session_from_request(req: &Request<Bytes>) -> Result<&UserSession> {
    req.extensions()
        .get::<UserSession>()
        .ok_or_else(|| PolisError::Auth("Sessão do usuário não encontrada".to_string()))
}

pub fn create_unauthorized_response() -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header("WWW-Authenticate", "Bearer")
        .body(Bytes::from("Token de autenticação inválido ou ausente"))
        .unwrap()
}

pub fn create_forbidden_response() -> Response<Bytes> {
    Response::builder()
        .status(StatusCode::FORBIDDEN)
        .body(Bytes::from("Acesso negado: permissão insuficiente"))
        .unwrap()
}
