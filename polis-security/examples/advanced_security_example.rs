use polis_core::types::ContainerId;
use polis_core::types::ResourceLimits;
use polis_core::Result;
use polis_security::{AppArmorManager, SELinuxManager, SecurityManager};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🔒 Exemplo de Segurança Avançada - Polis");
    println!("=========================================");

    // 1. Inicializar gerenciador de segurança
    println!("\n1. 🏗️  Inicializando gerenciador de segurança...");
    let mut security_manager = SecurityManager::new();
    security_manager.initialize().await?;
    println!("   ✅ Gerenciador de segurança inicializado");

    // 2. Verificar status dos sistemas de segurança
    println!("\n2. 📊 Verificando status dos sistemas de segurança...");
    let status = security_manager.get_status().await?;
    println!(
        "   - Namespaces: {}",
        if status.namespaces_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   - Cgroups: {}",
        if status.cgroups_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   - Seccomp: {}",
        if status.seccomp_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   - Capabilities: {}",
        if status.capabilities_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   - AppArmor: {}",
        if status.apparmor_available {
            "✅"
        } else {
            "❌"
        }
    );
    println!(
        "   - SELinux: {}",
        if status.selinux_available {
            "✅"
        } else {
            "❌"
        }
    );

    // 3. Testar AppArmor
    println!("\n3. 🛡️  Testando AppArmor...");
    let apparmor_manager = AppArmorManager::new();
    if apparmor_manager.is_available().await {
        println!("   ✅ AppArmor disponível");

        // Criar perfil de container
        let profile = apparmor_manager
            .create_container_profile("exemplo-container")
            .await?;
        println!("   ✅ Perfil AppArmor criado: {}", profile.name);
        println!("   - Regras: {} regras definidas", profile.rules.len());
        println!("   - Modo: {:?}", profile.mode);
    } else {
        println!("   ❌ AppArmor não disponível");
    }

    // 4. Testar SELinux
    println!("\n4. 🔐 Testando SELinux...");
    let selinux_manager = SELinuxManager::new();
    if selinux_manager.is_available().await {
        println!("   ✅ SELinux disponível");

        // Obter status
        let selinux_status = selinux_manager.get_status().await?;
        println!("   - Modo: {}", selinux_status.mode);

        // Criar contexto de container
        let context = selinux_manager
            .create_container_context("exemplo-container")
            .await?;
        println!("   ✅ Contexto SELinux criado:");
        println!("   - Usuário: {}", context.user);
        println!("   - Role: {}", context.role);
        println!("   - Tipo: {}", context.r#type);
        println!("   - Nível: {}", context.level);
    } else {
        println!("   ❌ SELinux não disponível");
    }

    // 5. Criar perfis de segurança para containers
    println!("\n5. 🐳 Criando perfis de segurança para containers...");

    let container_id_1 = ContainerId::new();
    let container_id_2 = ContainerId::new();
    let container_id_3 = ContainerId::new();

    // Perfil padrão
    let default_profile = security_manager
        .create_container_profile(&container_id_1)
        .await?;
    println!(
        "   ✅ Perfil padrão criado para container {:?}",
        container_id_1
    );
    println!("   - Namespaces: {:?}", default_profile.namespaces);
    println!(
        "   - Capabilities: {} capabilities",
        default_profile.capabilities.len()
    );

    // Perfil de alta segurança
    let high_security_profile = security_manager
        .create_high_security_profile(&container_id_2)
        .await?;
    println!(
        "   ✅ Perfil de alta segurança criado para container {:?}",
        container_id_2
    );
    println!("   - Namespaces: {:?}", high_security_profile.namespaces);
    println!(
        "   - Capabilities: {} capabilities",
        high_security_profile.capabilities.len()
    );
    if let Some(sandbox_config) = &high_security_profile.sandbox_config {
        println!(
            "   - RootFS somente leitura: {}",
            sandbox_config.read_only_rootfs
        );
        println!(
            "   - Sem novos privilégios: {}",
            sandbox_config.no_new_privileges
        );
    }

    // Perfil privilegiado
    let privileged_profile = security_manager
        .create_privileged_profile(&container_id_3)
        .await?;
    println!(
        "   ✅ Perfil privilegiado criado para container {:?}",
        container_id_3
    );
    println!("   - Capabilities: {:?}", privileged_profile.capabilities);

    // 6. Demonstrar gerenciamento de perfis
    println!("\n6. 📋 Gerenciando perfis de segurança...");

    let profiles = security_manager.list_container_profiles().await;
    println!("   ✅ {} perfis de container ativos:", profiles.len());
    for profile in profiles {
        println!(
            "   - Container {:?}: {} namespaces, {} capabilities",
            profile.container_id,
            profile.namespaces.len(),
            profile.capabilities.len()
        );
    }

    // 7. Atualizar limites de recursos
    println!("\n7. ⚙️  Atualizando limites de recursos...");
    let new_limits = ResourceLimits {
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        memory_swap: Some(512 * 1024 * 1024),  // 512MB
        cpu_quota: Some(50000.0),              // 50% de CPU
        cpu_period: Some(100000),
        pids_limit: Some(100),
        disk_quota: Some(1024 * 1024 * 1024), // 1GB
    };

    security_manager
        .update_cgroup_limits(&container_id_1, new_limits)
        .await?;
    println!(
        "   ✅ Limites de recursos atualizados para container {:?}",
        container_id_1
    );

    // 8. Atualizar capabilities
    println!("\n8. 🔑 Atualizando capabilities...");
    let new_capabilities = vec![
        "CHOWN".to_string(),
        "DAC_OVERRIDE".to_string(),
        "FOWNER".to_string(),
        "KILL".to_string(),
    ];

    security_manager
        .update_capabilities(&container_id_1, new_capabilities)
        .await?;
    println!(
        "   ✅ Capabilities atualizadas para container {:?}",
        container_id_1
    );

    // 9. Demonstrar remoção de perfis
    println!("\n9. 🗑️  Removendo perfis de segurança...");

    security_manager
        .remove_container_profile(&container_id_1)
        .await?;
    println!("   ✅ Perfil removido para container {:?}", container_id_1);

    let remaining_profiles = security_manager.list_container_profiles().await;
    println!("   ✅ {} perfis restantes", remaining_profiles.len());

    // 10. Demonstrar configurações de sandbox
    println!("\n10. 🏖️  Configurações de sandbox...");

    if let Some(profile) = security_manager
        .get_container_profile(&container_id_2)
        .await
        .ok()
    {
        if let Some(sandbox_config) = &profile.sandbox_config {
            println!(
                "   - RootFS somente leitura: {}",
                sandbox_config.read_only_rootfs
            );
            println!(
                "   - Sem novos privilégios: {}",
                sandbox_config.no_new_privileges
            );
            println!(
                "   - Caminhos mascarados: {} itens",
                sandbox_config.masked_paths.len()
            );
            println!(
                "   - Caminhos somente leitura: {} itens",
                sandbox_config.readonly_paths.len()
            );
            println!(
                "   - Montagens tmpfs: {} itens",
                sandbox_config.tmpfs_mounts.len()
            );
        }
    }

    // 11. Limpeza
    println!("\n11. 🧹 Limpando recursos...");

    security_manager
        .remove_container_profile(&container_id_2)
        .await?;
    security_manager
        .remove_container_profile(&container_id_3)
        .await?;

    let final_profiles = security_manager.list_container_profiles().await;
    println!(
        "   ✅ Limpeza concluída. {} perfis restantes",
        final_profiles.len()
    );

    println!("\n✅ Exemplo de segurança avançada concluído com sucesso!");
    println!("\n📝 Resumo das funcionalidades demonstradas:");
    println!("   - Gerenciamento de namespaces Linux");
    println!("   - Controle de recursos com cgroups");
    println!("   - Filtros de syscalls com seccomp");
    println!("   - Controle de capabilities");
    println!("   - Perfis de segurança AppArmor");
    println!("   - Contextos de segurança SELinux");
    println!("   - Configurações de sandbox");
    println!("   - Perfis de segurança personalizáveis");

    Ok(())
}
