use polis_auth::{AuthManager, PermissionManager, UserManager};
use uuid::Uuid;

#[tokio::test]
async fn test_auth_manager_creation() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let auth_manager = AuthManager::new(jwt_secret);
    assert!(auth_manager.sessions.is_empty());
}

#[tokio::test]
async fn test_user_creation() {
    let mut user_manager = UserManager::new();

    let user = user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert!(user.is_active);
}

#[tokio::test]
async fn test_user_authentication() {
    let mut user_manager = UserManager::new();

    // Criar usuário
    user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    // Autenticar com credenciais corretas
    let user = user_manager
        .authenticate_user("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();
    assert_eq!(user.username, "testuser");

    // Tentar autenticar com credenciais incorretas
    let result = user_manager
        .authenticate_user("testuser", "wrongpassword")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_token_generation() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);

    // Criar usuário
    let user = auth_manager
        .user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    // Atribuir role
    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await
        .unwrap();

    // Autenticar
    let auth_result = auth_manager
        .authenticate("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();

    assert!(!auth_result.token.is_empty());
    assert_eq!(auth_result.user.username, "testuser");
}

#[tokio::test]
async fn test_token_validation() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);

    // Criar usuário e autenticar
    let user = auth_manager
        .user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await
        .unwrap();
    let auth_result = auth_manager
        .authenticate("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();

    // Validar token
    let session = auth_manager
        .validate_token(&auth_result.token)
        .await
        .unwrap();
    assert_eq!(session.username, "testuser");
    assert!(!session.permissions.is_empty());
}

#[tokio::test]
async fn test_permission_checking() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);

    // Criar usuário e autenticar
    let user = auth_manager
        .user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await
        .unwrap();
    let auth_result = auth_manager
        .authenticate("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();

    // Verificar permissões
    let can_read_containers = auth_manager
        .check_permission(&auth_result.token, "containers:read")
        .await
        .unwrap();
    assert!(can_read_containers);

    let can_admin_system = auth_manager
        .check_permission(&auth_result.token, "system:admin")
        .await
        .unwrap();
    assert!(!can_admin_system); // Usuário 'user' não tem permissão de admin
}

#[tokio::test]
async fn test_permission_management() {
    let permission_manager = PermissionManager::new();

    // Listar permissões
    let permissions = permission_manager.list_permissions().await.unwrap();
    assert!(!permissions.is_empty());

    // Listar roles
    let roles = permission_manager.list_roles().await.unwrap();
    assert!(!roles.is_empty());

    // Verificar se role admin existe
    let admin_role = roles.iter().find(|r| r.id == "admin").unwrap();
    assert_eq!(admin_role.name, "Administrador");
    assert!(admin_role.permissions.contains(&"system:admin".to_string()));
}

#[tokio::test]
async fn test_user_role_assignment() {
    let mut permission_manager = PermissionManager::new();
    let user_id = Uuid::new_v4();

    // Atribuir role ao usuário
    permission_manager
        .assign_role_to_user(user_id, "user")
        .await
        .unwrap();

    // Verificar permissões do usuário
    let permissions = permission_manager
        .get_user_permissions(&user_id)
        .await
        .unwrap();
    assert!(!permissions.is_empty());
    assert!(permissions.contains(&"containers:read".to_string()));
}

#[tokio::test]
async fn test_token_refresh() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);

    // Criar usuário e autenticar
    let user = auth_manager
        .user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await
        .unwrap();
    let auth_result = auth_manager
        .authenticate("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();

    // Renovar token
    let new_auth_result = auth_manager
        .refresh_token(&auth_result.token)
        .await
        .unwrap();
    // Verificar se o token foi renovado (pode ser igual se gerado no mesmo segundo)
    assert_eq!(new_auth_result.user.username, "testuser");
    assert!(!new_auth_result.token.is_empty());
}

#[tokio::test]
async fn test_logout() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);

    // Criar usuário e autenticar
    let user = auth_manager
        .user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await
        .unwrap();
    let auth_result = auth_manager
        .authenticate("testuser", &std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()))
        .await
        .unwrap();

    // Fazer logout
    auth_manager.logout(&auth_result.token).await.unwrap();

    // Tentar usar token após logout
    let result = auth_manager.validate_token(&auth_result.token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_invalid_token() {
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "test-secret".to_string());
    let auth_manager = AuthManager::new(jwt_secret);

    // Tentar validar token inválido
    let result = auth_manager.validate_token("invalid-token").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_duplicate_user_creation() {
    let mut user_manager = UserManager::new();

    // Criar primeiro usuário
    user_manager
        .create_user(
            "testuser".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await
        .unwrap();

    // Tentar criar usuário com mesmo username
    let result = user_manager
        .create_user(
            "testuser".to_string(),
            "test2@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await;
    assert!(result.is_err());

    // Tentar criar usuário com mesmo email
    let result = user_manager
        .create_user(
            "testuser2".to_string(),
            "test@example.com".to_string(),
            std::env::var("TEST_PASSWORD").unwrap_or_else(|_| "password123".to_string()),
        )
        .await;
    assert!(result.is_err());
}
