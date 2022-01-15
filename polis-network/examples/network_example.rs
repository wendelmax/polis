use polis_network::firewall::Protocol as FirewallProtocol;
use polis_network::port_forwarding::Protocol as PortProtocol;
use polis_network::{
    BridgeManager, DnsManager, DnsRecordType, FirewallAction, FirewallManager, IpamManager,
    PortForwardingManager,
};
use std::net::{IpAddr, Ipv4Addr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Exemplo do Sistema de Rede do Polis");
    println!("=====================================\n");

    // 1. Gerenciamento de Bridges
    println!("1. 🌉 Configurando Bridges...");
    let mut bridge_manager = BridgeManager::new();

    // Criar bridge padrão
    bridge_manager.create_default_bridge().await?;

    // Criar bridge adicional
    bridge_manager
        .create_bridge("custom-bridge", "192.168.1.1", "192.168.1.0/24", 1500)
        .await?;

    // Listar bridges
    let bridges = bridge_manager.list_bridges().await?;
    println!("   📋 Bridges criadas: {}", bridges.len());
    for bridge in &bridges {
        println!("   - {}: {} ({})", bridge.name, bridge.ip, bridge.subnet);
    }

    // 2. Gerenciamento de IPAM
    println!("\n2. 📊 Configurando IPAM...");
    let mut ipam_manager = IpamManager::new();

    // Criar pools de IP
    ipam_manager
        .create_pool("default", "172.17.0.0/16", "172.17.0.1")
        .await?;
    ipam_manager
        .create_pool("custom", "192.168.1.0/24", "192.168.1.1")
        .await?;

    // Alocar IPs para containers
    let allocation1 = ipam_manager.allocate_ip("container-1", None).await?;
    let allocation2 = ipam_manager
        .allocate_ip("container-2", Some("custom"))
        .await?;

    println!("   📋 Alocações de IP:");
    println!("   - {}: {}", allocation1.container_id, allocation1.ip);
    println!("   - {}: {}", allocation2.container_id, allocation2.ip);

    // Estatísticas do pool
    let stats = ipam_manager.get_pool_stats(None).await?;
    println!(
        "   📊 Pool '{}': {} IPs alocados, {} disponíveis",
        stats.name, stats.allocated_ips, stats.available_ips
    );

    // 3. Gerenciamento de Firewall
    println!("\n3. 🔥 Configurando Firewall...");
    let mut firewall_manager = FirewallManager::new();

    // Criar regras para containers
    firewall_manager
        .create_container_rule("container-1", FirewallAction::Allow)
        .await?;
    firewall_manager
        .create_container_rule("container-2", FirewallAction::Allow)
        .await?;

    // Criar regras de porta
    firewall_manager
        .create_port_rule(80, FirewallProtocol::Tcp, FirewallAction::Allow)
        .await?;
    firewall_manager
        .create_port_rule(443, FirewallProtocol::Tcp, FirewallAction::Allow)
        .await?;
    firewall_manager
        .create_port_rule(22, FirewallProtocol::Tcp, FirewallAction::Deny)
        .await?;

    // Criar regra de IP
    let test_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
    firewall_manager
        .create_ip_rule(test_ip, FirewallAction::Allow)
        .await?;

    // Listar regras
    let rules = firewall_manager.list_rules(Some("POLIS-FILTER")).await?;
    println!("   📋 Regras do firewall: {}", rules.len());

    // Estatísticas
    let chain_stats = firewall_manager
        .get_chain_stats(Some("POLIS-FILTER"))
        .await?;
    println!(
        "   📊 Chain '{}': {} regras ({} allow, {} deny)",
        chain_stats.name, chain_stats.total_rules, chain_stats.allow_rules, chain_stats.deny_rules
    );

    // 4. Gerenciamento de DNS
    println!("\n4. 🌐 Configurando DNS...");
    let mut dns_manager = DnsManager::new();

    // Criar registros para containers
    dns_manager
        .create_container_record("container-1", allocation1.ip)
        .await?;
    dns_manager
        .create_container_record("container-2", allocation2.ip)
        .await?;

    // Criar alias
    dns_manager
        .create_alias_record("web", "container-1", "container.local")
        .await?;
    dns_manager
        .create_alias_record("api", "container-2", "container.local")
        .await?;

    // Resolver nomes
    let web_records = dns_manager.resolve("web", DnsRecordType::A).await;
    match web_records {
        Ok(records) => {
            if let Some(record) = records.first() {
                println!("   🔍 Resolução: web -> {}", record.value);
            }
        }
        Err(e) => {
            println!("   ⚠️  Erro na resolução: {}", e);
        }
    }

    // Estatísticas da zona
    let zone_stats = dns_manager.get_zone_stats(Some("container.local")).await?;
    println!(
        "   📊 Zona '{}': {} registros",
        zone_stats.name, zone_stats.total_records
    );

    // 5. Port Forwarding
    println!("\n5. 🔀 Configurando Port Forwarding...");
    let mut port_manager = PortForwardingManager::new();

    // Criar port forwarding para containers
    port_manager
        .create_container_forwarding(allocation1.ip, 80, Some(8080), PortProtocol::Tcp)
        .await?;
    port_manager
        .create_container_forwarding(allocation2.ip, 443, Some(8443), PortProtocol::Tcp)
        .await?;

    // Criar range de port forwarding
    let range_rules = port_manager
        .create_range_forwarding(
            Ipv4Addr::new(0, 0, 0, 0).into(),
            9000,
            9005,
            allocation1.ip,
            3000,
            PortProtocol::Tcp,
        )
        .await?;

    println!(
        "   📋 Port forwarding criado: {} regras",
        range_rules.len() + 2
    );

    // Estatísticas
    let pf_stats = port_manager.get_stats().await?;
    println!(
        "   📊 Port Forwarding: {} regras ativas ({} TCP, {} UDP)",
        pf_stats.active_rules, pf_stats.tcp_rules, pf_stats.udp_rules
    );

    // 6. Configuração de Rede de Container
    println!("\n6. 🐳 Configurando Rede de Container...");

    // Simular configuração de rede para container
    bridge_manager
        .setup_container_network("test-container", allocation1.ip)
        .await?;

    // Configurar port forwarding
    port_manager
        .create_container_forwarding(allocation1.ip, 80, None, PortProtocol::Tcp)
        .await?;

    // Criar regra de firewall
    firewall_manager
        .create_container_rule("test-container", FirewallAction::Allow)
        .await?;

    // Criar registro DNS
    dns_manager
        .create_container_record("test-container", allocation1.ip)
        .await?;

    println!("   ✅ Rede do container configurada completamente");

    // 7. Limpeza
    println!("\n7. 🧹 Limpando recursos...");

    // Desalocar IPs
    ipam_manager.deallocate_ip("container-1", None).await?;
    ipam_manager
        .deallocate_ip("container-2", Some("custom"))
        .await?;

    // Limpar port forwarding
    port_manager.clear_container_rules(allocation1.ip).await?;

    // Limpar rede do container
    bridge_manager
        .cleanup_container_network("test-container")
        .await?;

    println!("   ✅ Recursos de rede limpos");

    println!("\n✅ Exemplo de rede concluído com sucesso!");
    println!("\n🌐 Recursos de Rede Implementados:");
    println!("   - Gerenciamento de bridges com interfaces virtuais");
    println!("   - IPAM com pools de IP e alocação dinâmica");
    println!("   - Firewall com regras por container, porta e IP");
    println!("   - DNS com zonas locais e resolução de nomes");
    println!("   - Port forwarding com ranges e mapeamento automático");
    println!("   - Integração completa entre todos os componentes");

    Ok(())
}
