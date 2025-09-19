use polis_core::{PolisConfig, ContainerId, ResourceLimits, Protocol, PortMapping};
use polis_runtime::{PolisRuntime, ContainerRuntime};
use polis_image::{ImageManager, RegistryClient};
use polis_network::{NetworkManager, BridgeManager, IpamManager};
use polis_security::{SecurityManager, NamespaceManager, CgroupManager};
use polis_monitor::{MonitorManager, MetricsCollector, HealthChecker};
use std::collections::HashMap;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Exemplo de Uso Básico do Polis");
    println!("=================================");

    // 1. Inicializar configuração
    let config = PolisConfig::default();
    println!(" Configuração carregada");

    // 2. Inicializar runtime
    let runtime = PolisRuntime::new(config.clone());
    runtime.initialize().await?;
    println!(" Runtime inicializado");

    // 3. Inicializar gerenciadores
    let image_manager = ImageManager::new(config.clone());
    let network_manager = NetworkManager::new(config.clone());
    let security_manager = SecurityManager::new(config.clone());
    let monitor_manager = MonitorManager::new(config.clone());

    // 4. Configurar rede
    let mut bridge_manager = BridgeManager::new();
    bridge_manager.create_default_bridge().await?;
    println!(" Bridge padrão criada");

    let mut ipam_manager = IpamManager::new();
    ipam_manager.create_pool("default", "172.17.0.0/16", "172.17.0.1").await?;
    println!(" Pool IP configurado");

    // 5. Baixar imagem
    let registry_client = RegistryClient::new();
    let image_id = registry_client.pull_image("alpine", "latest", "docker.io").await?;
    println!(" Imagem baixada: {}", image_id);

    // 6. Criar container
    let container_id = runtime.create_container(
        "exemplo-container".to_string(),
        "alpine:latest".to_string(),
        vec!["echo".to_string(), "Hello from Polis!".to_string()],
    ).await?;
    println!(" Container criado: {}", container_id);

    // 7. Configurar port forwarding
    let mut port_manager = polis_network::PortForwardingManager::new();
    let port_rule = port_manager.create_container_forwarding(
        &container_id.to_string(),
        8080,
        80,
        polis_network::Protocol::Tcp,
    ).await?;
    println!(" Port forwarding configurado: {}", port_rule.id);

    // 8. Configurar segurança
    let mut namespace_manager = NamespaceManager::new();
    namespace_manager.create_namespaces(&container_id.to_string()).await?;
    println!(" Namespaces criados");

    let mut cgroup_manager = CgroupManager::new();
    let resource_limits = ResourceLimits {
        memory_limit: Some(1073741824), // 1GB
        memory_swap: Some(2147483648),  // 2GB
        cpu_quota: Some(0.5),           // 50% CPU
        cpu_period: Some(100000),       // 100ms
        disk_quota: Some(10737418240),  // 10GB
        pids_limit: Some(100),          // 100 processos
    };
    cgroup_manager.create_cgroup(&container_id.to_string(), &resource_limits).await?;
    println!(" Cgroup configurado");

    // 9. Iniciar container
    runtime.start_container(container_id.clone()).await?;
    println!(" Container iniciado");

    // 10. Configurar monitoramento
    let mut metrics_collector = MetricsCollector::new();
    metrics_collector.start_collection().await?;
    println!(" Coleta de métricas iniciada");

    let mut health_checker = HealthChecker::new();
    health_checker.add_container_check(&container_id.to_string()).await?;
    println!(" Health check configurado");

    // 11. Aguardar um pouco para o container executar
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // 12. Verificar status do container
    let container = runtime.get_container(container_id.clone()).await?;
    println!(" Status do container: {:?}", container.status);

    // 13. Obter métricas
    let system_metrics = metrics_collector.get_system_metrics().await?;
    println!(" Métricas do sistema:");
    println!("   CPU: {:.2}%", system_metrics.cpu_usage);
    println!("   Memória: {:.2}%", system_metrics.memory_usage);
    println!("   Disco: {:.2}%", system_metrics.disk_usage);

    let container_metrics = metrics_collector.get_container_metrics(&container_id).await?;
    println!(" Métricas do container:");
    println!("   CPU: {:.2}%", container_metrics.cpu_usage);
    println!("   Memória: {} bytes", container_metrics.memory_usage);

    // 14. Verificar saúde
    let health_status = health_checker.check_container_health(&container_id).await?;
    println!(" Status de saúde: {:?}", health_status);

    // 15. Parar container
    runtime.stop_container(container_id.clone()).await?;
    println!(" Container parado");

    // 16. Limpar recursos
    runtime.remove_container(container_id).await?;
    println!(" Container removido");

    // 17. Parar monitoramento
    metrics_collector.stop_collection().await?;
    println!(" Monitoramento parado");

    println!("\n Exemplo concluído com sucesso!");
    Ok(())
}

