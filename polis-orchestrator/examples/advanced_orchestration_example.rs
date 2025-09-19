use polis_core::PolisConfig;
use polis_orchestrator::{
    AutoScaler, CheckType, Deployment, HealthCheck, HealthMonitor, HealthStatus, LoadBalancer,
    LoadBalancerRequest, LoadBalancingAlgorithm, Protocol, ResourceLimits, ResourceRequests,
    ScalingMetrics, ScalingPolicy, Service, ServiceDiscovery, ServiceEndpoint, TargetType,
};
use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!(" Exemplo de Orquestração Avançada do Polis");
    println!("=============================================");

    // 1. Service Discovery
    println!("\n1.  Service Discovery");
    println!("------------------------");

    let service_discovery = ServiceDiscovery::new();
    println!("    Service Discovery inicializado");

    // Criar serviço
    let service = Service::new(
        "web-service".to_string(),
        "default".to_string(),
        "1.0.0".to_string(),
    )
    .with_endpoint(ServiceEndpoint::new(
        "127.0.0.1".to_string(),
        8080,
        Protocol::Http,
    ))
    .with_endpoint(ServiceEndpoint::new(
        "127.0.0.1".to_string(),
        8081,
        Protocol::Http,
    ))
    .with_label("app".to_string(), "web".to_string())
    .with_annotation(
        "description".to_string(),
        "Web service for demo".to_string(),
    );

    service_discovery.register_service(service.clone()).await?;
    println!("    Serviço registrado: {}", service.name);

    // Resolver serviço
    let endpoints = service_discovery
        .resolve_service("web-service", Some("default"))
        .await?;
    println!("    Endpoints encontrados: {}", endpoints.len());
    for endpoint in &endpoints {
        println!("     - {}:{}", endpoint.address, endpoint.port);
    }

    // 2. Load Balancer
    println!("\n2. ⚖ Load Balancer");
    println!("-------------------");

    let load_balancer = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);
    println!("    Load Balancer inicializado (Round Robin)");

    // Adicionar endpoints
    for endpoint in endpoints {
        load_balancer.add_endpoint(endpoint).await;
    }
    println!("    Endpoints adicionados ao load balancer");

    // Simular requisições
    for i in 0..5 {
        let request = LoadBalancerRequest {
            client_ip: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                192,
                168,
                1,
                i as u8 + 1,
            ))),
            session_id: Some(format!("session-{}", i)),
            headers: HashMap::new(),
            path: "/api/health".to_string(),
            method: "GET".to_string(),
        };

        let response = load_balancer.handle_request(request).await?;
        println!(
            "   - Requisição {}: Endpoint {} (Status: {})",
            i + 1,
            response.endpoint.id,
            response.status_code
        );
    }

    // Obter estatísticas
    let stats = load_balancer.get_stats().await;
    println!("    Estatísticas do Load Balancer:");
    println!("     - Total de requisições: {}", stats.total_requests);
    println!(
        "     - Requisições bem-sucedidas: {}",
        stats.successful_requests
    );
    println!("     - Requisições falharam: {}", stats.failed_requests);

    // 3. Auto Scaling
    println!("\n3.  Auto Scaling");
    println!("------------------");

    let auto_scaler = AutoScaler::new();
    println!("    Auto Scaler inicializado");

    // Criar deployment
    let deployment = Deployment::new(
        "web-deployment".to_string(),
        "web".to_string(),
        "default".to_string(),
        "nginx:latest".to_string(),
    )
    .with_replicas(2)
    .with_resource_limits(ResourceLimits {
        cpu: Some("500m".to_string()),
        memory: Some("512Mi".to_string()),
        storage: None,
    })
    .with_resource_requests(ResourceRequests {
        cpu: Some("250m".to_string()),
        memory: Some("256Mi".to_string()),
        storage: None,
    })
    .with_label("app".to_string(), "web".to_string());

    auto_scaler.create_deployment(deployment).await?;
    println!("    Deployment criado: {}", deployment.name);

    // Criar política de scaling
    let scaling_policy = ScalingPolicy::new(
        "web-scaling-policy".to_string(),
        "Web Scaling Policy".to_string(),
        "web-deployment".to_string(),
        1,
        10,
    )
    .with_target_cpu_utilization(70.0)
    .with_target_memory_utilization(80.0)
    .with_scale_up_cooldown(Duration::from_secs(300))
    .with_scale_down_cooldown(Duration::from_secs(600));

    auto_scaler.create_scaling_policy(scaling_policy).await?;
    println!("    Política de scaling criada");

    // Simular métricas de alta utilização
    let high_usage_metrics = ScalingMetrics {
        deployment_id: "web-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 85.0,
        memory_utilization: 75.0,
        requests_per_second: 150.0,
        response_time: Duration::from_millis(200),
        error_rate: 0.02,
        active_connections: 100,
    };

    auto_scaler
        .collect_metrics("web-deployment", high_usage_metrics)
        .await?;
    println!("    Métricas de alta utilização coletadas");

    // Avaliar scaling
    let scaling_action = auto_scaler.evaluate_scaling("web-deployment").await?;
    println!("    Ação de scaling avaliada:");
    println!("     - Tipo: {:?}", scaling_action.action_type);
    println!(
        "     - De {} para {} réplicas",
        scaling_action.from_replicas, scaling_action.to_replicas
    );
    println!("     - Razão: {}", scaling_action.reason);

    // Simular métricas de baixa utilização
    let low_usage_metrics = ScalingMetrics {
        deployment_id: "web-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 30.0,
        memory_utilization: 40.0,
        requests_per_second: 20.0,
        response_time: Duration::from_millis(50),
        error_rate: 0.001,
        active_connections: 10,
    };

    auto_scaler
        .collect_metrics("web-deployment", low_usage_metrics)
        .await?;
    println!("    Métricas de baixa utilização coletadas");

    // Avaliar scaling novamente
    let scaling_action = auto_scaler.evaluate_scaling("web-deployment").await?;
    println!("    Ação de scaling avaliada:");
    println!("     - Tipo: {:?}", scaling_action.action_type);
    println!(
        "     - De {} para {} réplicas",
        scaling_action.from_replicas, scaling_action.to_replicas
    );
    println!("     - Razão: {}", scaling_action.reason);

    // 4. Health Monitoring
    println!("\n4.  Health Monitoring");
    println!("------------------------");

    let health_monitor = HealthMonitor::new();
    println!("    Health Monitor inicializado");

    // Criar health check HTTP
    let http_health_check = HealthCheck::new(
        "http-health-check".to_string(),
        "HTTP Health Check".to_string(),
        TargetType::Service,
        "web-service".to_string(),
        CheckType::Http {
            path: "/health".to_string(),
            expected_status: 200,
        },
    )
    .with_interval(Duration::from_secs(30))
    .with_timeout(Duration::from_secs(5))
    .with_retries(3)
    .with_label("app".to_string(), "web".to_string());

    health_monitor
        .create_health_check(http_health_check)
        .await?;
    println!("    Health check HTTP criado");

    // Criar health check TCP
    let tcp_health_check = HealthCheck::new(
        "tcp-health-check".to_string(),
        "TCP Health Check".to_string(),
        TargetType::Container,
        "web-container".to_string(),
        CheckType::Tcp { port: 8080 },
    )
    .with_interval(Duration::from_secs(30))
    .with_timeout(Duration::from_secs(5));

    health_monitor.create_health_check(tcp_health_check).await?;
    println!("    Health check TCP criado");

    // Criar health check de comando
    let cmd_health_check = HealthCheck::new(
        "cmd-health-check".to_string(),
        "Command Health Check".to_string(),
        TargetType::Container,
        "web-container".to_string(),
        CheckType::Command {
            command: "curl".to_string(),
            args: vec!["-f".to_string(), "http://localhost:8080/health".to_string()],
        },
    )
    .with_interval(Duration::from_secs(60))
    .with_timeout(Duration::from_secs(10));

    health_monitor.create_health_check(cmd_health_check).await?;
    println!("    Health check de comando criado");

    // Executar health check manual
    let result = health_monitor.run_health_check("http-health-check").await?;
    println!("    Health check executado:");
    println!("     - Status: {:?}", result.status);
    println!("     - Mensagem: {}", result.message);
    println!("     - Tempo de resposta: {:?}", result.response_time);

    // Obter resumo de saúde
    let summary = health_monitor.get_health_summary(None).await;
    println!("    Resumo de saúde:");
    println!("     - Status geral: {:?}", summary.overall_status);
    println!("     - Total de checks: {}", summary.total_checks);
    println!("     - Checks saudáveis: {}", summary.healthy_checks);
    println!("     - Checks não saudáveis: {}", summary.unhealthy_checks);

    // Obter estatísticas de saúde
    let stats = health_monitor.get_health_stats().await;
    println!("    Estatísticas de saúde:");
    println!("     - Total de checks: {}", stats.total_checks);
    println!("     - Checks saudáveis: {}", stats.healthy_checks);
    println!("     - Checks não saudáveis: {}", stats.unhealthy_checks);
    println!("     - Checks degradados: {}", stats.degraded_checks);
    println!("     - Checks desconhecidos: {}", stats.unknown_checks);
    println!(
        "     - Tempo médio de resposta: {:?}",
        stats.average_response_time
    );
    println!(
        "     - Porcentagem de uptime: {:.2}%",
        stats.uptime_percentage
    );

    // 5. Integração Completa
    println!("\n5. � Integração Completa");
    println!("-------------------------");

    // Simular cenário completo
    println!("    Simulando cenário de produção...");

    // Registrar múltiplos serviços
    for i in 1..=3 {
        let service = Service::new(
            format!("api-service-{}", i),
            "production".to_string(),
            "2.0.0".to_string(),
        )
        .with_endpoint(ServiceEndpoint::new(
            format!("10.0.0.{}", i),
            8080,
            Protocol::Http,
        ))
        .with_label("tier".to_string(), "api".to_string())
        .with_label("version".to_string(), "2.0.0".to_string());

        service_discovery.register_service(service).await?;
    }

    println!("    3 serviços API registrados");

    // Configurar load balancer para múltiplos serviços
    let api_endpoints = service_discovery
        .resolve_service("api-service-1", Some("production"))
        .await?;
    for endpoint in api_endpoints {
        load_balancer.add_endpoint(endpoint).await;
    }

    println!("    Endpoints API adicionados ao load balancer");

    // Simular tráfego
    for i in 0..10 {
        let request = LoadBalancerRequest {
            client_ip: Some(std::net::IpAddr::V4(std::net::Ipv4Addr::new(
                192,
                168,
                1,
                (i % 255) as u8 + 1,
            ))),
            session_id: Some(format!("session-{}", i)),
            headers: HashMap::new(),
            path: "/api/v1/users".to_string(),
            method: "GET".to_string(),
        };

        let response = load_balancer.handle_request(request).await?;
        println!(
            "   - Requisição API {}: Endpoint {} (Status: {})",
            i + 1,
            response.endpoint.id,
            response.status_code
        );
    }

    // Simular métricas de produção
    let production_metrics = ScalingMetrics {
        deployment_id: "web-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 45.0,
        memory_utilization: 60.0,
        requests_per_second: 80.0,
        response_time: Duration::from_millis(120),
        error_rate: 0.005,
        active_connections: 45,
    };

    auto_scaler
        .collect_metrics("web-deployment", production_metrics)
        .await?;
    println!("    Métricas de produção coletadas");

    // Avaliar scaling com métricas de produção
    let scaling_action = auto_scaler.evaluate_scaling("web-deployment").await?;
    println!("    Ação de scaling com métricas de produção:");
    println!("     - Tipo: {:?}", scaling_action.action_type);
    println!(
        "     - De {} para {} réplicas",
        scaling_action.from_replicas, scaling_action.to_replicas
    );
    println!("     - Razão: {}", scaling_action.reason);

    // 6. Relatório Final
    println!("\n6. � Relatório Final");
    println!("---------------------");

    // Service Discovery
    let services = service_discovery.list_services().await;
    println!("    Service Discovery:");
    println!("     - Total de serviços: {}", services.len());
    for service in &services {
        println!(
            "       - {} ({}): {} endpoints",
            service.name,
            service.namespace,
            service.endpoints.len()
        );
    }

    // Load Balancer
    let lb_stats = load_balancer.get_stats().await;
    println!("   ⚖ Load Balancer:");
    println!("     - Total de requisições: {}", lb_stats.total_requests);
    println!(
        "     - Taxa de sucesso: {:.2}%",
        (lb_stats.successful_requests as f64 / lb_stats.total_requests as f64) * 100.0
    );
    println!(
        "     - Tempo médio de resposta: {:?}",
        lb_stats.average_response_time
    );

    // Auto Scaling
    let deployments = auto_scaler.list_deployments().await;
    println!("    Auto Scaling:");
    println!("     - Total de deployments: {}", deployments.len());
    for deployment in &deployments {
        println!(
            "       - {}: {} réplicas (desejadas: {})",
            deployment.name, deployment.replicas, deployment.desired_replicas
        );
    }

    // Health Monitoring
    let health_summary = health_monitor.get_health_summary(None).await;
    println!("    Health Monitoring:");
    println!("     - Status geral: {:?}", health_summary.overall_status);
    println!("     - Total de checks: {}", health_summary.total_checks);
    println!("     - Checks saudáveis: {}", health_summary.healthy_checks);
    println!(
        "     - Checks não saudáveis: {}",
        health_summary.unhealthy_checks
    );

    println!("\n Exemplo de orquestração avançada concluído com sucesso!");
    println!("=========================================================");

    Ok(())
}
