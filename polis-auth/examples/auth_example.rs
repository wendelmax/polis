use polis_auth::{AuthManager, PermissionManager, UserManager};
use polis_core::Result;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    println!("� Exemplo de Autenticação e Autorização - Polis");
    println!("=================================================");

    // 1. Criar gerenciador de autenticação
    println!("\n1.   Criando gerenciador de autenticação...");
    let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "minha-chave-secreta-super-segura".to_string());
    let mut auth_manager = AuthManager::new(jwt_secret);
    println!("    Gerenciador de autenticação criado");

    // 2. Criar usuário
    println!("\n2. � Criando usuário...");
    let user = auth_manager
        .user_manager
        .create_user(
            "usuario_teste".to_string(),
            "teste@polis.local".to_string(),
            "senha123".to_string(),
        )
        .await?;
    println!("    Usuário criado: {}", user.username);

    // 3. Atribuir role ao usuário
    println!("\n3. � Atribuindo role ao usuário...");
    auth_manager
        .permission_manager
        .assign_role_to_user(user.id, "user")
        .await?;
    println!("    Role 'user' atribuída ao usuário");

    // 4. Autenticar usuário
    println!("\n4. � Autenticando usuário...");
    let auth_result = auth_manager
        .authenticate("usuario_teste", "senha123")
        .await?;
    println!("    Usuário autenticado com sucesso");
    println!("   - Token: {}...", &auth_result.token[..20]);
    println!("   - Expira em: {}", auth_result.expires_at);

    // 5. Validar token
    println!("\n5.  Validando token...");
    let session = auth_manager.validate_token(&auth_result.token).await?;
    println!("    Token válido");
    println!("   - Usuário: {}", session.username);
    println!("   - Permissões: {:?}", session.permissions);

    // 6. Verificar permissões
    println!("\n6.  Verificando permissões...");
    let can_read_containers = auth_manager
        .check_permission(&auth_result.token, "containers:read")
        .await?;
    println!("   - Pode ler containers: {}", can_read_containers);

    let can_delete_containers = auth_manager
        .check_permission(&auth_result.token, "containers:delete")
        .await?;
    println!("   - Pode deletar containers: {}", can_delete_containers);

    let can_admin_system = auth_manager
        .check_permission(&auth_result.token, "system:admin")
        .await?;
    println!("   - Pode administrar sistema: {}", can_admin_system);

    // 7. Listar permissões disponíveis
    println!("\n7. � Listando permissões disponíveis...");
    let permissions = auth_manager.permission_manager.list_permissions().await?;
    println!("    {} permissões encontradas:", permissions.len());
    for permission in permissions {
        println!("   - {}: {}", permission.id, permission.name);
    }

    // 8. Listar roles disponíveis
    println!("\n8. � Listando roles disponíveis...");
    let roles = auth_manager.permission_manager.list_roles().await?;
    println!("    {} roles encontradas:", roles.len());
    for role in roles {
        println!(
            "   - {}: {} ({} permissões)",
            role.id,
            role.name,
            role.permissions.len()
        );
    }

    // 9. Refresh token
    println!("\n9. � Renovando token...");
    let new_auth_result = auth_manager.refresh_token(&auth_result.token).await?;
    println!("    Token renovado com sucesso");
    println!("   - Novo token: {}...", &new_auth_result.token[..20]);

    // 10. Logout
    println!("\n10. � Fazendo logout...");
    auth_manager.logout(&new_auth_result.token).await?;
    println!("    Logout realizado com sucesso");

    // 11. Tentar usar token após logout
    println!("\n11.  Tentando usar token após logout...");
    match auth_manager.validate_token(&new_auth_result.token).await {
        Ok(_) => println!("   ⚠  Token ainda válido (não deveria estar)"),
        Err(e) => println!("    Token inválido como esperado: {}", e),
    }

    println!("\n Exemplo de autenticação concluído com sucesso!");
    Ok(())
}
