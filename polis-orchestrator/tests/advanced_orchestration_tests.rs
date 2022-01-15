use polis_orchestrator::{
    AutoScaler, CheckType, Deployment, HealthCheck, HealthMonitor, HealthStatus, LoadBalancer,
    LoadBalancerRequest, LoadBalancingAlgorithm, Protocol, ResourceLimits, ResourceRequests,
    ScalingMetrics, ScalingPolicy, Service, ServiceDiscovery, ServiceEndpoint, TargetType,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

#[tokio::test]
async fn test_service_discovery() {
    let discovery = ServiceDiscovery::new();

    let service = Service::new(
        "test-service".to_string(),
        "default".to_string(),
        "1.0.0".to_string(),
    )
    .with_endpoint(ServiceEndpoint::new(
        "127.0.0.1".to_string(),
        8080,
        Protocol::Http,
    ))
    .with_label("app".to_string(), "test".to_string());

    discovery.register_service(service.clone()).await.unwrap();

    let found_service = discovery.get_service(&service.id).await;
    assert!(found_service.is_some());
    assert_eq!(found_service.unwrap().name, "test-service");

    let services = discovery.list_services().await;
    assert_eq!(services.len(), 1);

    let found_services = discovery
        .find_services("test-service", Some("default"))
        .await;
    assert_eq!(found_services.len(), 1);
    assert_eq!(found_services[0].name, "test-service");
}

#[tokio::test]
async fn test_service_endpoints() {
    let discovery = ServiceDiscovery::new();

    let service = Service::new(
        "test-service".to_string(),
        "default".to_string(),
        "1.0.0".to_string(),
    );

    discovery.register_service(service).await.unwrap();

    let endpoint1 = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http)
        .with_weight(2)
        .with_priority(1);

    let endpoint2 = ServiceEndpoint::new("127.0.0.1".to_string(), 8081, Protocol::Http)
        .with_weight(1)
        .with_priority(2);

    discovery
        .add_endpoint(&service.id, endpoint1)
        .await
        .unwrap();
    discovery
        .add_endpoint(&service.id, endpoint2)
        .await
        .unwrap();

    let endpoints = discovery
        .resolve_service("test-service", Some("default"))
        .await
        .unwrap();
    assert_eq!(endpoints.len(), 2);

    let healthy_endpoints = discovery.get_healthy_endpoints(&service.id).await;
    assert_eq!(healthy_endpoints.len(), 2);
}

#[tokio::test]
async fn test_load_balancer_round_robin() {
    let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);

    let endpoint1 = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http);
    let endpoint2 = ServiceEndpoint::new("127.0.0.1".to_string(), 8081, Protocol::Http);

    lb.add_endpoint(endpoint1).await;
    lb.add_endpoint(endpoint2).await;

    let request = LoadBalancerRequest {
        client_ip: None,
        session_id: None,
        headers: HashMap::new(),
        path: "/".to_string(),
        method: "GET".to_string(),
    };

    let selected1 = lb.select_endpoint(&request).await.unwrap();
    let selected2 = lb.select_endpoint(&request).await.unwrap();

    // Should alternate between endpoints
    assert_ne!(selected1.id, selected2.id);
}

#[tokio::test]
async fn test_load_balancer_weighted_round_robin() {
    let lb = LoadBalancer::new(LoadBalancingAlgorithm::WeightedRoundRobin);

    let mut endpoint1 = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http);
    endpoint1.weight = 3;

    let mut endpoint2 = ServiceEndpoint::new("127.0.0.1".to_string(), 8081, Protocol::Http);
    endpoint2.weight = 1;

    lb.add_endpoint(endpoint1).await;
    lb.add_endpoint(endpoint2).await;

    let request = LoadBalancerRequest {
        client_ip: None,
        session_id: None,
        headers: HashMap::new(),
        path: "/".to_string(),
        method: "GET".to_string(),
    };

    // Should select endpoint1 more often due to higher weight
    let mut endpoint1_count = 0;
    for _ in 0..100 {
        let selected = lb.select_endpoint(&request).await.unwrap();
        if selected.weight == 3 {
            endpoint1_count += 1;
        }
    }

    assert!(endpoint1_count > 50); // Should be more than 50% due to weight 3:1
}

#[tokio::test]
async fn test_load_balancer_ip_hash() {
    let lb = LoadBalancer::new(LoadBalancingAlgorithm::IpHash);

    let endpoint1 = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http);
    let endpoint2 = ServiceEndpoint::new("127.0.0.1".to_string(), 8081, Protocol::Http);

    lb.add_endpoint(endpoint1).await;
    lb.add_endpoint(endpoint2).await;

    let request = LoadBalancerRequest {
        client_ip: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
            192, 168, 1, 1,
        ))),
        session_id: None,
        headers: HashMap::new(),
        path: "/".to_string(),
        method: "GET".to_string(),
    };

    // Same IP should always select the same endpoint
    let selected1 = lb.select_endpoint(&request).await.unwrap();
    let selected2 = lb.select_endpoint(&request).await.unwrap();

    assert_eq!(selected1.id, selected2.id);
}

#[tokio::test]
async fn test_auto_scaler() {
    let auto_scaler = AutoScaler::new();

    let deployment = Deployment::new(
        "test-deployment".to_string(),
        "test".to_string(),
        "default".to_string(),
        "nginx:latest".to_string(),
    )
    .with_replicas(2);

    auto_scaler.create_deployment(deployment).await.unwrap();

    let policy = ScalingPolicy::new(
        "test-policy".to_string(),
        "test".to_string(),
        "test-deployment".to_string(),
        1,
        10,
    )
    .with_target_cpu_utilization(50.0);

    auto_scaler.create_scaling_policy(policy).await.unwrap();

    let metrics = ScalingMetrics {
        deployment_id: "test-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 80.0,
        memory_utilization: 60.0,
        requests_per_second: 150.0,
        response_time: Duration::from_millis(100),
        error_rate: 0.01,
        active_connections: 50,
    };

    auto_scaler
        .collect_metrics("test-deployment", metrics)
        .await
        .unwrap();

    let action = auto_scaler
        .evaluate_scaling("test-deployment")
        .await
        .unwrap();
    assert!(action.to_replicas > action.from_replicas);
}

#[tokio::test]
async fn test_scaling_policy() {
    let policy = ScalingPolicy::new(
        "test-policy".to_string(),
        "test".to_string(),
        "test-deployment".to_string(),
        1,
        10,
    )
    .with_target_cpu_utilization(70.0)
    .with_target_memory_utilization(80.0)
    .with_scale_up_cooldown(Duration::from_secs(300));

    assert_eq!(policy.min_replicas, 1);
    assert_eq!(policy.max_replicas, 10);
    assert_eq!(policy.target_cpu_utilization, 70.0);
    assert_eq!(policy.target_memory_utilization, 80.0);
    assert_eq!(policy.scale_up_cooldown, Duration::from_secs(300));
}

#[tokio::test]
async fn test_deployment() {
    let deployment = Deployment::new(
        "test-deployment".to_string(),
        "test".to_string(),
        "default".to_string(),
        "nginx:latest".to_string(),
    )
    .with_replicas(3)
    .with_resource_limits(ResourceLimits {
        cpu: Some("500m".to_string()),
        memory: Some("512Mi".to_string()),
        storage: None,
    })
    .with_label("app".to_string(), "nginx".to_string());

    assert_eq!(deployment.replicas, 3);
    assert_eq!(deployment.desired_replicas, 3);
    assert_eq!(deployment.labels.get("app"), Some(&"nginx".to_string()));
}

#[tokio::test]
async fn test_health_monitor() {
    let monitor = HealthMonitor::new();

    let check = HealthCheck::new(
        "test-check".to_string(),
        "test".to_string(),
        TargetType::Container,
        "test-container".to_string(),
        CheckType::Http {
            path: "/health".to_string(),
            expected_status: 200,
        },
    )
    .with_interval(Duration::from_secs(10))
    .with_timeout(Duration::from_secs(5));

    monitor.create_health_check(check).await.unwrap();

    let checks = monitor.list_health_checks().await;
    assert_eq!(checks.len(), 1);
    assert_eq!(checks[0].name, "test");

    let summary = monitor.get_health_summary(None).await;
    assert_eq!(summary.total_checks, 0); // No results yet

    let stats = monitor.get_health_stats().await;
    assert_eq!(stats.total_checks, 0);
}

#[tokio::test]
async fn test_health_check_types() {
    let monitor = HealthMonitor::new();

    // HTTP check
    let http_check = HealthCheck::new(
        "http-check".to_string(),
        "HTTP".to_string(),
        TargetType::Service,
        "test-service".to_string(),
        CheckType::Http {
            path: "/health".to_string(),
            expected_status: 200,
        },
    );

    // TCP check
    let tcp_check = HealthCheck::new(
        "tcp-check".to_string(),
        "TCP".to_string(),
        TargetType::Container,
        "test-container".to_string(),
        CheckType::Tcp { port: 8080 },
    );

    // Command check
    let cmd_check = HealthCheck::new(
        "cmd-check".to_string(),
        "Command".to_string(),
        TargetType::Container,
        "test-container".to_string(),
        CheckType::Command {
            command: "echo".to_string(),
            args: vec!["hello".to_string()],
        },
    );

    monitor.create_health_check(http_check).await.unwrap();
    monitor.create_health_check(tcp_check).await.unwrap();
    monitor.create_health_check(cmd_check).await.unwrap();

    let checks = monitor.list_health_checks().await;
    assert_eq!(checks.len(), 3);
}

#[tokio::test]
async fn test_health_check_result() {
    let monitor = HealthMonitor::new();

    let check = HealthCheck::new(
        "test-check".to_string(),
        "test".to_string(),
        TargetType::Container,
        "test-container".to_string(),
        CheckType::Http {
            path: "/health".to_string(),
            expected_status: 200,
        },
    );

    monitor.create_health_check(check).await.unwrap();

    let result = monitor.run_health_check("test-check").await.unwrap();
    assert_eq!(result.check_id, "test-check");
    assert_eq!(result.target_id, "test-container");
    // Status will likely be Unhealthy since we're not running a real service
    assert!(matches!(
        result.status,
        HealthStatus::Unhealthy | HealthStatus::Unknown
    ));
}

#[tokio::test]
async fn test_service_endpoint_creation() {
    let endpoint = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http)
        .with_weight(2)
        .with_priority(1)
        .with_metadata("region".to_string(), "us-west".to_string());

    assert_eq!(endpoint.address, "127.0.0.1");
    assert_eq!(endpoint.port, 8080);
    assert_eq!(endpoint.protocol, Protocol::Http);
    assert_eq!(endpoint.weight, 2);
    assert_eq!(endpoint.priority, 1);
    assert_eq!(
        endpoint.metadata.get("region"),
        Some(&"us-west".to_string())
    );
}

#[tokio::test]
async fn test_service_creation() {
    let service = Service::new(
        "test-service".to_string(),
        "default".to_string(),
        "1.0.0".to_string(),
    )
    .with_endpoint(ServiceEndpoint::new(
        "127.0.0.1".to_string(),
        8080,
        Protocol::Http,
    ))
    .with_label("app".to_string(), "test".to_string())
    .with_annotation("description".to_string(), "Test service".to_string());

    assert_eq!(service.name, "test-service");
    assert_eq!(service.namespace, "default");
    assert_eq!(service.version, "1.0.0");
    assert_eq!(service.endpoints.len(), 1);
    assert_eq!(service.labels.get("app"), Some(&"test".to_string()));
    assert_eq!(
        service.annotations.get("description"),
        Some(&"Test service".to_string())
    );
}

#[tokio::test]
async fn test_load_balancer_stats() {
    let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);

    let endpoint = ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http);
    lb.add_endpoint(endpoint).await;

    let stats = lb.get_stats().await;
    assert_eq!(stats.total_requests, 0);
    assert_eq!(stats.successful_requests, 0);
    assert_eq!(stats.failed_requests, 0);
}

#[tokio::test]
async fn test_scaling_metrics() {
    let metrics = ScalingMetrics {
        deployment_id: "test-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 50.0,
        memory_utilization: 60.0,
        requests_per_second: 100.0,
        response_time: Duration::from_millis(200),
        error_rate: 0.01,
        active_connections: 25,
    };

    assert_eq!(metrics.deployment_id, "test-deployment");
    assert_eq!(metrics.cpu_utilization, 50.0);
    assert_eq!(metrics.memory_utilization, 60.0);
    assert_eq!(metrics.requests_per_second, 100.0);
    assert_eq!(metrics.response_time, Duration::from_millis(200));
    assert_eq!(metrics.error_rate, 0.01);
    assert_eq!(metrics.active_connections, 25);
}

#[tokio::test]
async fn test_health_status_enum() {
    assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
    assert_eq!(HealthStatus::Unhealthy, HealthStatus::Unhealthy);
    assert_eq!(HealthStatus::Degraded, HealthStatus::Degraded);
    assert_eq!(HealthStatus::Unknown, HealthStatus::Unknown);

    assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Degraded);
    assert_ne!(HealthStatus::Healthy, HealthStatus::Unknown);
}

#[tokio::test]
async fn test_protocol_enum() {
    assert_eq!(Protocol::Http, Protocol::Http);
    assert_eq!(Protocol::Https, Protocol::Https);
    assert_eq!(Protocol::Tcp, Protocol::Tcp);
    assert_eq!(Protocol::Udp, Protocol::Udp);
    assert_eq!(Protocol::Grpc, Protocol::Grpc);

    assert_ne!(Protocol::Http, Protocol::Https);
    assert_ne!(Protocol::Http, Protocol::Tcp);
    assert_ne!(Protocol::Http, Protocol::Udp);
    assert_ne!(Protocol::Http, Protocol::Grpc);
}

#[tokio::test]
async fn test_target_type_enum() {
    assert_eq!(TargetType::Container, TargetType::Container);
    assert_eq!(TargetType::Service, TargetType::Service);
    assert_eq!(TargetType::Deployment, TargetType::Deployment);
    assert_eq!(TargetType::Node, TargetType::Node);
    assert_eq!(TargetType::Custom, TargetType::Custom);

    assert_ne!(TargetType::Container, TargetType::Service);
    assert_ne!(TargetType::Container, TargetType::Deployment);
    assert_ne!(TargetType::Container, TargetType::Node);
    assert_ne!(TargetType::Container, TargetType::Custom);
}

#[tokio::test]
async fn test_check_type_enum() {
    let http_check = CheckType::Http {
        path: "/health".to_string(),
        expected_status: 200,
    };

    let tcp_check = CheckType::Tcp { port: 8080 };
    let udp_check = CheckType::Udp { port: 8080 };

    let grpc_check = CheckType::Grpc {
        service: "health".to_string(),
        method: "check".to_string(),
    };

    let cmd_check = CheckType::Command {
        command: "curl".to_string(),
        args: vec!["-f".to_string(), "http://localhost:8080/health".to_string()],
    };

    let file_check = CheckType::File {
        path: "/tmp/health".to_string(),
        exists: true,
    };

    let custom_check = CheckType::Custom {
        script: "#!/bin/bash\necho 'healthy'".to_string(),
    };

    // Test that all variants can be created
    assert!(matches!(http_check, CheckType::Http { .. }));
    assert!(matches!(tcp_check, CheckType::Tcp { .. }));
    assert!(matches!(udp_check, CheckType::Udp { .. }));
    assert!(matches!(grpc_check, CheckType::Grpc { .. }));
    assert!(matches!(cmd_check, CheckType::Command { .. }));
    assert!(matches!(file_check, CheckType::File { .. }));
    assert!(matches!(custom_check, CheckType::Custom { .. }));
}
