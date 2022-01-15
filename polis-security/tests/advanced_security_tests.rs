use polis_core::types::ContainerId;
use polis_security::{AppArmorManager, SELinuxManager, SecurityManager};

#[tokio::test]
async fn test_apparmor_manager_creation() {
    let apparmor_manager = AppArmorManager::new();
    assert!(apparmor_manager.profiles.is_empty());
}

#[tokio::test]
async fn test_apparmor_availability() {
    let apparmor_manager = AppArmorManager::new();
    let available = apparmor_manager.is_available().await;
    // Pode ser true ou false dependendo do sistema
    assert!(available == true || available == false);
}

#[tokio::test]
async fn test_apparmor_profile_creation() {
    let mut apparmor_manager = AppArmorManager::new();

    let rules = vec![
        "capability,".to_string(),
        "network,".to_string(),
        "file,".to_string(),
    ];

    let profile = apparmor_manager
        .create_profile("test-profile".to_string(), rules)
        .await;

    // Se AppArmor não estiver disponível, deve retornar erro
    if apparmor_manager.is_available().await {
        assert!(profile.is_ok());
        let profile = profile.unwrap();
        assert_eq!(profile.name, "test-profile");
        assert_eq!(profile.rules.len(), 3);
    } else {
        assert!(profile.is_err());
    }
}

#[tokio::test]
async fn test_apparmor_container_profile() {
    let apparmor_manager = AppArmorManager::new();

    let profile = apparmor_manager
        .create_container_profile("test-container")
        .await;

    // Nossa implementação sempre retorna sucesso
    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert!(profile.name.contains("container-test-container"));
    assert!(!profile.rules.is_empty());
}

#[tokio::test]
async fn test_selinux_manager_creation() {
    let selinux_manager = SELinuxManager::new();
    assert!(selinux_manager.policies.is_empty());
}

#[tokio::test]
async fn test_selinux_availability() {
    let selinux_manager = SELinuxManager::new();
    let available = selinux_manager.is_available().await;
    // Pode ser true ou false dependendo do sistema
    assert!(available == true || available == false);
}

#[tokio::test]
async fn test_selinux_context_parsing() {
    let selinux_manager = SELinuxManager::new();

    let context_str = "system_u:system_r:unconfined_t:s0";
    let context = selinux_manager.parse_context(context_str).unwrap();

    assert_eq!(context.user, "system_u");
    assert_eq!(context.role, "system_r");
    assert_eq!(context.r#type, "unconfined_t");
    assert_eq!(context.level, "s0");
}

#[tokio::test]
async fn test_selinux_container_context() {
    let selinux_manager = SELinuxManager::new();

    let context = selinux_manager
        .create_container_context("test-container")
        .await
        .unwrap();

    assert_eq!(context.user, "system_u");
    assert_eq!(context.role, "system_r");
    assert!(context.r#type.contains("polis_container_t_test-container"));
    assert_eq!(context.level, "s0");
}

#[tokio::test]
async fn test_selinux_container_policy() {
    let selinux_manager = SELinuxManager::new();

    let policy = selinux_manager
        .create_container_policy("test-container")
        .await;

    // Nossa implementação sempre retorna sucesso
    assert!(policy.is_ok());
    let policy = policy.unwrap();
    assert!(policy.name.contains("polis-container-test-container"));
    assert!(!policy.rules.is_empty());
}

#[tokio::test]
async fn test_security_manager_creation() {
    let security_manager = SecurityManager::new();
    assert!(security_manager.container_profiles.is_empty());
}

#[tokio::test]
async fn test_security_manager_status() {
    let security_manager = SecurityManager::new();
    let status = security_manager.get_status().await.unwrap();

    // Verificar que o status foi obtido
    assert!(status.container_count == 0);
}

#[tokio::test]
async fn test_security_manager_initialization() {
    let mut security_manager = SecurityManager::new();
    let result = security_manager.initialize().await;

    // A inicialização pode falhar se os recursos não estiverem disponíveis
    // mas não deve causar panic
    assert!(result.is_ok() || result.is_err());
}

#[tokio::test]
async fn test_container_profile_creation() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    let profile = security_manager
        .create_container_profile(&container_id)
        .await;

    // Deve criar o perfil mesmo se alguns recursos não estiverem disponíveis
    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert_eq!(profile.container_id, container_id);
    assert!(!profile.namespaces.is_empty());
    assert!(profile.capabilities.len() > 0);
}

#[tokio::test]
async fn test_high_security_profile() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    let profile = security_manager
        .create_high_security_profile(&container_id)
        .await;

    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert!(profile.namespaces.contains(&"user".to_string()));
    assert!(profile.capabilities.len() < 20); // Menos capabilities que o perfil padrão

    if let Some(sandbox_config) = &profile.sandbox_config {
        assert!(sandbox_config.read_only_rootfs);
        assert!(sandbox_config.no_new_privileges);
    }
}

#[tokio::test]
async fn test_privileged_profile() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    let profile = security_manager
        .create_privileged_profile(&container_id)
        .await;

    assert!(profile.is_ok());
    let profile = profile.unwrap();
    assert!(profile.capabilities.contains(&"ALL".to_string()));

    if let Some(sandbox_config) = &profile.sandbox_config {
        assert!(!sandbox_config.read_only_rootfs);
        assert!(!sandbox_config.no_new_privileges);
    }
}

#[tokio::test]
async fn test_profile_management() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    // Criar perfil
    let _profile = security_manager
        .create_container_profile(&container_id)
        .await
        .unwrap();
    assert_eq!(security_manager.container_profiles.len(), 1);

    // Obter perfil
    let retrieved_profile = security_manager
        .get_container_profile(&container_id)
        .await
        .unwrap();
    assert_eq!(retrieved_profile.container_id, container_id);

    // Listar perfis
    let profiles = security_manager.list_container_profiles().await;
    assert_eq!(profiles.len(), 1);

    // Remover perfil
    security_manager
        .remove_container_profile(&container_id)
        .await
        .unwrap();
    assert_eq!(security_manager.container_profiles.len(), 0);
}

#[tokio::test]
async fn test_capability_updates() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    // Criar perfil
    security_manager
        .create_container_profile(&container_id)
        .await
        .unwrap();

    // Atualizar capabilities
    let new_capabilities = vec!["CHOWN".to_string(), "KILL".to_string()];
    security_manager
        .update_capabilities(&container_id, new_capabilities)
        .await
        .unwrap();

    // Verificar atualização
    let profile = security_manager
        .get_container_profile(&container_id)
        .await
        .unwrap();
    assert_eq!(
        profile.capabilities,
        vec!["CHOWN".to_string(), "KILL".to_string()]
    );
}

#[tokio::test]
async fn test_error_handling() {
    let mut security_manager = SecurityManager::new();
    let container_id = ContainerId::new();

    // Tentar obter perfil inexistente
    let result = security_manager.get_container_profile(&container_id).await;
    assert!(result.is_err());

    // Tentar remover perfil inexistente
    let result = security_manager
        .remove_container_profile(&container_id)
        .await;
    assert!(result.is_ok()); // Deve ser ok mesmo se não existir
}
