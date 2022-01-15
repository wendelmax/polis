use polis_core::ResourceLimits;
use polis_security::{
    Capability, CapabilityManager, CgroupManager, NamespaceManager, SeccompManager,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔒 Exemplo do Sistema de Segurança do Polis");
    println!("==========================================\n");

    // 1. Gerenciamento de Namespaces
    println!("1. 🏗️  Criando Namespaces...");
    let mut namespace_manager = NamespaceManager::new();

    let namespaces = namespace_manager.create_container_namespaces().await?;
    println!("   ✅ {} namespaces criados", namespaces.len());

    for ns in &namespaces {
        println!("   - {:?}: {}", ns.namespace_type, ns.path);
    }

    // 2. Gerenciamento de Cgroups
    println!("\n2. 📊 Configurando Cgroups...");
    let cgroup_path = PathBuf::from("/sys/fs/cgroup");
    let mut cgroup_manager = CgroupManager::new(cgroup_path);

    let limits = ResourceLimits {
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        cpu_quota: Some(50000.0),              // 50% CPU
        cpu_period: Some(100000),              // 100ms period
        pids_limit: Some(100),
        disk_quota: Some(1024 * 1024 * 1024),  // 1GB
        memory_swap: Some(1024 * 1024 * 1024), // 1GB swap
    };

    let cgroup_info = cgroup_manager
        .create_cgroup("polis-container", limits)
        .await?;
    println!("   ✅ Cgroup '{}' criado", cgroup_info.name);

    // Adicionar processo ao cgroup
    cgroup_manager.add_process("polis-container", 1234).await?;
    println!("   ✅ Processo adicionado ao cgroup");

    // 3. Gerenciamento de Seccomp
    println!("\n3. 🛡️  Configurando Seccomp...");
    let mut seccomp_manager = SeccompManager::new();

    // Criar perfil padrão
    seccomp_manager.create_default_profile().await?;
    println!("   ✅ Perfil Seccomp padrão criado");

    // Aplicar perfil
    seccomp_manager.apply_profile("default").await?;

    // Listar perfis disponíveis
    let profiles = seccomp_manager.list_profiles().await?;
    println!("   📋 Perfis disponíveis: {:?}", profiles);

    // 4. Gerenciamento de Capabilities
    println!("\n4. 🔐 Configurando Capabilities...");
    let mut capability_manager = CapabilityManager::new();

    // Criar conjunto mínimo de capabilities
    capability_manager.create_minimal_capset().await?;
    println!("   ✅ Conjunto mínimo de capabilities definido");

    // Listar capabilities atuais
    let current_caps = capability_manager.list_capabilities().await?;
    println!(
        "   📋 Capabilities atuais: {} capabilities",
        current_caps.len()
    );

    // Adicionar capabilities específicas
    let additional_caps = vec![Capability::NetAdmin, Capability::SysAdmin];
    capability_manager.add_capabilities(additional_caps).await?;

    // Remover capabilities específicas
    let caps_to_remove = vec![
        Capability::SysAdmin, // Remover SysAdmin por segurança
    ];
    capability_manager.drop_capabilities(caps_to_remove).await?;

    // 5. Configuração de Hostname
    println!("\n5. 🏷️  Configurando Hostname...");
    namespace_manager.setup_hostname("polis-container").await?;

    // 6. Estatísticas do Cgroup
    println!("\n6. 📈 Estatísticas do Cgroup...");
    let stats = cgroup_manager.get_cgroup_stats("polis-container").await?;
    println!("   - Uso de memória: {} bytes", stats.memory_usage);
    println!("   - Uso de CPU: {} ns", stats.cpu_usage);
    println!("   - Número de processos: {}", stats.process_count);

    // 7. Listar Cgroups
    println!("\n7. 📋 Listando Cgroups...");
    let cgroups = cgroup_manager.list_cgroups().await?;
    for cgroup in cgroups {
        println!(
            "   - {}: {} bytes de memória",
            cgroup.name,
            cgroup.limits.memory_limit.unwrap_or(0)
        );
    }

    println!("\n✅ Exemplo de segurança concluído com sucesso!");
    println!("\n🔒 Recursos de Segurança Implementados:");
    println!("   - Namespaces para isolamento de processos");
    println!("   - Cgroups para limitação de recursos");
    println!("   - Seccomp para filtragem de syscalls");
    println!("   - Capabilities para controle de privilégios");
    println!("   - Configuração de hostname");

    Ok(())
}
