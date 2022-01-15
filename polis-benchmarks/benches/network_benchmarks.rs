use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use polis_core::PolisConfig;
use polis_network::{
    BridgeManager, DnsManager, FirewallManager, IpamManager, PortForwardingManager,
};
use tokio::runtime::Runtime;

fn network_creation_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let bridge_manager = BridgeManager::new(config.network.bridge_name.clone());
        let ipam_manager = IpamManager::new(config.network.subnet.clone());
        let firewall_manager = FirewallManager::new();
        let dns_manager = DnsManager::new();
        let port_forwarding_manager = PortForwardingManager::new();

        let mut group = c.benchmark_group("network_creation");

        // Benchmark bridge creation
        group.bench_function("create_bridge", |b| {
            b.to_async(&rt)
                .iter(|| async { bridge_manager.create_bridge("test-bridge").await });
        });

        // Benchmark subnet allocation
        group.bench_function("allocate_subnet", |b| {
            b.to_async(&rt)
                .iter(|| async { ipam_manager.allocate_subnet(24).await });
        });

        // Benchmark IP allocation
        group.bench_function("allocate_ip", |b| {
            b.to_async(&rt).iter(|| async {
                let subnet = ipam_manager.allocate_subnet(24).await.unwrap();
                ipam_manager.allocate_ip(&subnet).await
            });
        });

        group.finish();
    });
}

fn firewall_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let firewall_manager = FirewallManager::new();

        let mut group = c.benchmark_group("firewall");

        // Benchmark rule creation
        group.bench_function("create_rule", |b| {
            b.to_async(&rt).iter(|| async {
                firewall_manager
                    .create_rule(
                        "test-container",
                        "INPUT",
                        "ACCEPT",
                        Some("tcp"),
                        Some(8080),
                        None,
                    )
                    .await
            });
        });

        // Benchmark rule listing
        group.bench_function("list_rules", |b| {
            b.to_async(&rt)
                .iter(|| async { firewall_manager.list_rules().await });
        });

        // Benchmark rule deletion
        group.bench_function("delete_rule", |b| {
            b.to_async(&rt).iter(|| async {
                let rule_id = firewall_manager
                    .create_rule(
                        "test-container",
                        "INPUT",
                        "ACCEPT",
                        Some("tcp"),
                        Some(8080),
                        None,
                    )
                    .await
                    .unwrap();

                firewall_manager.delete_rule(&rule_id).await
            });
        });

        group.finish();
    });
}

fn dns_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let dns_manager = DnsManager::new();

        let mut group = c.benchmark_group("dns");

        // Benchmark record creation
        group.bench_function("create_record", |b| {
            b.to_async(&rt).iter(|| async {
                dns_manager
                    .create_record("test.example.com", "A", "192.168.1.100")
                    .await
            });
        });

        // Benchmark record lookup
        group.bench_function("lookup_record", |b| {
            b.to_async(&rt).iter(|| async {
                let _ = dns_manager
                    .create_record("test.example.com", "A", "192.168.1.100")
                    .await;

                dns_manager.lookup_record("test.example.com", "A").await
            });
        });

        // Benchmark zone creation
        group.bench_function("create_zone", |b| {
            b.to_async(&rt)
                .iter(|| async { dns_manager.create_zone("example.com").await });
        });

        group.finish();
    });
}

fn port_forwarding_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let port_forwarding_manager = PortForwardingManager::new();

        let mut group = c.benchmark_group("port_forwarding");

        // Benchmark port forwarding creation
        group.bench_function("create_port_forwarding", |b| {
            b.to_async(&rt).iter(|| async {
                port_forwarding_manager
                    .create_port_forwarding("test-container", 8080, 80)
                    .await
            });
        });

        // Benchmark port forwarding listing
        group.bench_function("list_port_forwardings", |b| {
            b.to_async(&rt)
                .iter(|| async { port_forwarding_manager.list_port_forwardings().await });
        });

        // Benchmark port forwarding deletion
        group.bench_function("delete_port_forwarding", |b| {
            b.to_async(&rt).iter(|| async {
                let forwarding_id = port_forwarding_manager
                    .create_port_forwarding("test-container", 8080, 80)
                    .await
                    .unwrap();

                port_forwarding_manager
                    .delete_port_forwarding(&forwarding_id)
                    .await
            });
        });

        group.finish();
    });
}

fn concurrent_network_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let config = PolisConfig::default();
        let bridge_manager = BridgeManager::new(config.network.bridge_name.clone());
        let ipam_manager = IpamManager::new(config.network.subnet.clone());
        let firewall_manager = FirewallManager::new();
        let dns_manager = DnsManager::new();
        let port_forwarding_manager = PortForwardingManager::new();

        let mut group = c.benchmark_group("concurrent_network_operations");

        group.bench_function("concurrent_network_setup", |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::new();

                // Create bridge
                let bridge_manager = bridge_manager.clone();
                let handle1 =
                    tokio::spawn(async move { bridge_manager.create_bridge("test-bridge").await });
                handles.push(handle1);

                // Allocate subnet
                let ipam_manager = ipam_manager.clone();
                let handle2 = tokio::spawn(async move { ipam_manager.allocate_subnet(24).await });
                handles.push(handle2);

                // Create firewall rule
                let firewall_manager = firewall_manager.clone();
                let handle3 = tokio::spawn(async move {
                    firewall_manager
                        .create_rule(
                            "test-container",
                            "INPUT",
                            "ACCEPT",
                            Some("tcp"),
                            Some(8080),
                            None,
                        )
                        .await
                });
                handles.push(handle3);

                // Create DNS record
                let dns_manager = dns_manager.clone();
                let handle4 = tokio::spawn(async move {
                    dns_manager
                        .create_record("test.example.com", "A", "192.168.1.100")
                        .await
                });
                handles.push(handle4);

                // Create port forwarding
                let port_forwarding_manager = port_forwarding_manager.clone();
                let handle5 = tokio::spawn(async move {
                    port_forwarding_manager
                        .create_port_forwarding("test-container", 8080, 80)
                        .await
                });
                handles.push(handle5);

                for handle in handles {
                    handle.await.unwrap().unwrap();
                }
            });
        });

        group.finish();
    });
}

fn network_serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_serialization");

    // Benchmark bridge serialization
    group.bench_function("serialize_bridge", |b| {
        b.iter(|| {
            let bridge = polis_network::Bridge {
                name: "test-bridge".to_string(),
                subnet: "192.168.1.0/24".to_string(),
                gateway: "192.168.1.1".to_string(),
                containers: vec!["container1".to_string(), "container2".to_string()],
            };

            serde_json::to_string(&bridge)
        });
    });

    // Benchmark firewall rule serialization
    group.bench_function("serialize_firewall_rule", |b| {
        b.iter(|| {
            let rule = polis_network::FirewallRule {
                id: uuid::Uuid::new_v4().to_string(),
                container_id: "test-container".to_string(),
                chain: "INPUT".to_string(),
                action: "ACCEPT".to_string(),
                protocol: Some("tcp".to_string()),
                port: Some(8080),
                source_ip: None,
            };

            serde_json::to_string(&rule)
        });
    });

    // Benchmark DNS record serialization
    group.bench_function("serialize_dns_record", |b| {
        b.iter(|| {
            let record = polis_network::DnsRecord {
                name: "test.example.com".to_string(),
                r#type: "A".to_string(),
                value: "192.168.1.100".to_string(),
                ttl: 300,
            };

            serde_json::to_string(&record)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    network_creation_benchmark,
    firewall_benchmark,
    dns_benchmark,
    port_forwarding_benchmark,
    concurrent_network_operations_benchmark,
    network_serialization_benchmark
);
criterion_main!(benches);
