# Polis Orchestrator

Sistema avançado de orquestração de containers para o Polis, incluindo service discovery, load balancing, auto-scaling e health monitoring.

## 🚀 Funcionalidades

### 1. Service Discovery
- **Registro de Serviços**: Registro automático de serviços e endpoints
- **Resolução de Serviços**: Descoberta de serviços por nome e namespace
- **Health Checks**: Verificação de saúde de endpoints
- **Eventos**: Sistema de eventos para mudanças de serviços
- **DNS Integration**: Integração com resolução DNS

### 2. Load Balancing
- **Múltiplos Algoritmos**: Round Robin, Weighted Round Robin, Least Connections, Random, IP Hash, Consistent Hash
- **Sticky Sessions**: Suporte a sessões persistentes
- **Health-aware**: Balanceamento baseado em saúde dos endpoints
- **Estatísticas**: Métricas detalhadas de performance
- **Protocolos**: Suporte a HTTP, HTTPS, TCP, UDP, gRPC

### 3. Auto Scaling
- **Scaling Policies**: Políticas configuráveis de scaling
- **Métricas**: CPU, memória, requests por segundo
- **Cooldowns**: Períodos de cooldown para scaling up/down
- **Deployments**: Gerenciamento de deployments
- **Histórico**: Histórico de ações de scaling

### 4. Health Monitoring
- **Múltiplos Tipos**: HTTP, TCP, UDP, gRPC, Command, File, Custom
- **Configuração Flexível**: Intervalos, timeouts, retries
- **Status Tracking**: Rastreamento de status de saúde
- **Eventos**: Eventos de mudança de status
- **Relatórios**: Relatórios e estatísticas de saúde

## 📦 Uso

### Service Discovery

```rust
use polis_orchestrator::{ServiceDiscovery, Service, ServiceEndpoint, Protocol};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let discovery = ServiceDiscovery::new();
    
    // Criar serviço
    let service = Service::new(
        "web-service".to_string(),
        "default".to_string(),
        "1.0.0".to_string(),
    ).with_endpoint(ServiceEndpoint::new("127.0.0.1".to_string(), 8080, Protocol::Http))
     .with_label("app".to_string(), "web".to_string());
    
    // Registrar serviço
    discovery.register_service(service).await?;
    
    // Resolver serviço
    let endpoints = discovery.resolve_service("web-service", Some("default")).await?;
    
    Ok(())
}
```

### Load Balancing

```rust
use polis_orchestrator::{LoadBalancer, LoadBalancingAlgorithm, LoadBalancerRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lb = LoadBalancer::new(LoadBalancingAlgorithm::RoundRobin);
    
    // Adicionar endpoints
    lb.add_endpoint(endpoint1).await;
    lb.add_endpoint(endpoint2).await;
    
    // Fazer requisição
    let request = LoadBalancerRequest {
        client_ip: None,
        session_id: None,
        headers: HashMap::new(),
        path: "/api/health".to_string(),
        method: "GET".to_string(),
    };
    
    let response = lb.handle_request(request).await?;
    println!("Response: {:?}", response);
    
    Ok(())
}
```

### Auto Scaling

```rust
use polis_orchestrator::{AutoScaler, ScalingPolicy, Deployment, ScalingMetrics};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let auto_scaler = AutoScaler::new();
    
    // Criar deployment
    let deployment = Deployment::new(
        "web-deployment".to_string(),
        "web".to_string(),
        "default".to_string(),
        "nginx:latest".to_string(),
    ).with_replicas(2);
    
    auto_scaler.create_deployment(deployment).await?;
    
    // Criar política de scaling
    let policy = ScalingPolicy::new(
        "web-scaling-policy".to_string(),
        "Web Scaling Policy".to_string(),
        "web-deployment".to_string(),
        1,
        10,
    ).with_target_cpu_utilization(70.0)
     .with_target_memory_utilization(80.0);
    
    auto_scaler.create_scaling_policy(policy).await?;
    
    // Coletar métricas
    let metrics = ScalingMetrics {
        deployment_id: "web-deployment".to_string(),
        timestamp: chrono::Utc::now(),
        cpu_utilization: 85.0,
        memory_utilization: 75.0,
        requests_per_second: 150.0,
        response_time: Duration::from_millis(200),
        error_rate: 0.02,
        active_connections: 100,
    };
    
    auto_scaler.collect_metrics("web-deployment", metrics).await?;
    
    // Avaliar scaling
    let action = auto_scaler.evaluate_scaling("web-deployment").await?;
    println!("Scaling action: {:?}", action);
    
    Ok(())
}
```

### Health Monitoring

```rust
use polis_orchestrator::{HealthMonitor, HealthCheck, TargetType, CheckType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let monitor = HealthMonitor::new();
    
    // Criar health check HTTP
    let http_check = HealthCheck::new(
        "http-health-check".to_string(),
        "HTTP Health Check".to_string(),
        TargetType::Service,
        "web-service".to_string(),
        CheckType::Http {
            path: "/health".to_string(),
            expected_status: 200,
        },
    ).with_interval(Duration::from_secs(30))
     .with_timeout(Duration::from_secs(5))
     .with_retries(3);
    
    monitor.create_health_check(http_check).await?;
    
    // Executar health check
    let result = monitor.run_health_check("http-health-check").await?;
    println!("Health check result: {:?}", result);
    
    // Obter resumo de saúde
    let summary = monitor.get_health_summary(None).await;
    println!("Health summary: {:?}", summary);
    
    Ok(())
}
```

## 🔧 Configuração

### Service Discovery

```rust
let service = Service::new(name, namespace, version)
    .with_endpoint(endpoint)
    .with_label("app", "web")
    .with_annotation("description", "Web service")
    .with_health_check(health_check)
    .with_load_balancer(load_balancer_config);
```

### Load Balancing

```rust
let lb = LoadBalancer::new(LoadBalancingAlgorithm::WeightedRoundRobin);
// ou
let lb = LoadBalancer::new(LoadBalancingAlgorithm::ConsistentHash);
```

### Auto Scaling

```rust
let policy = ScalingPolicy::new(id, name, deployment_id, min_replicas, max_replicas)
    .with_target_cpu_utilization(70.0)
    .with_target_memory_utilization(80.0)
    .with_scale_up_cooldown(Duration::from_secs(300))
    .with_scale_down_cooldown(Duration::from_secs(600));
```

### Health Monitoring

```rust
let check = HealthCheck::new(id, name, target_type, target_id, check_type)
    .with_interval(Duration::from_secs(30))
    .with_timeout(Duration::from_secs(5))
    .with_retries(3)
    .with_label("app", "web");
```

## 📊 Métricas e Monitoramento

### Service Discovery Metrics
- **Total de Serviços**: Número total de serviços registrados
- **Endpoints por Serviço**: Número de endpoints por serviço
- **Health Status**: Status de saúde dos endpoints
- **Resolução de Serviços**: Taxa de sucesso na resolução

### Load Balancer Metrics
- **Total de Requisições**: Número total de requisições processadas
- **Taxa de Sucesso**: Porcentagem de requisições bem-sucedidas
- **Tempo de Resposta**: Tempo médio de resposta
- **Distribuição de Carga**: Distribuição entre endpoints

### Auto Scaling Metrics
- **Replicas Atuais**: Número atual de réplicas
- **Replicas Desejadas**: Número desejado de réplicas
- **Utilização de CPU**: Utilização média de CPU
- **Utilização de Memória**: Utilização média de memória
- **Requests por Segundo**: Taxa de requisições

### Health Monitoring Metrics
- **Total de Checks**: Número total de health checks
- **Checks Saudáveis**: Número de checks saudáveis
- **Checks Não Saudáveis**: Número de checks não saudáveis
- **Tempo de Resposta**: Tempo médio de resposta dos checks
- **Uptime**: Porcentagem de uptime

## 🧪 Testes

```bash
# Executar todos os testes
cargo test

# Executar testes específicos
cargo test test_service_discovery
cargo test test_load_balancer
cargo test test_auto_scaler
cargo test test_health_monitor
```

## 📈 Exemplos

### Exemplo Completo

```bash
# Executar exemplo completo
cargo run --example advanced_orchestration_example
```

### Exemplo de Service Discovery

```bash
# Executar exemplo de service discovery
cargo run --example service_discovery_example
```

### Exemplo de Load Balancing

```bash
# Executar exemplo de load balancing
cargo run --example load_balancer_example
```

### Exemplo de Auto Scaling

```bash
# Executar exemplo de auto scaling
cargo run --example auto_scaling_example
```

### Exemplo de Health Monitoring

```bash
# Executar exemplo de health monitoring
cargo run --example health_monitor_example
```

## 🔍 Arquitetura

### Service Discovery
- **Service Registry**: Registro central de serviços
- **Health Checker**: Verificação de saúde de endpoints
- **DNS Resolver**: Resolução de nomes DNS
- **Event System**: Sistema de eventos para mudanças

### Load Balancing
- **Algorithm Engine**: Motor de algoritmos de balanceamento
- **Health Integration**: Integração com health checks
- **Session Management**: Gerenciamento de sessões
- **Statistics Collector**: Coletor de estatísticas

### Auto Scaling
- **Metrics Collector**: Coletor de métricas
- **Scaling Engine**: Motor de scaling
- **Policy Manager**: Gerenciador de políticas
- **Deployment Manager**: Gerenciador de deployments

### Health Monitoring
- **Check Scheduler**: Agendador de checks
- **Result Storage**: Armazenamento de resultados
- **Event Generator**: Gerador de eventos
- **Statistics Calculator**: Calculador de estatísticas

## 🚀 Performance

### Benchmarks Típicos
- **Service Discovery**: < 1ms para resolução
- **Load Balancing**: < 0.1ms para seleção de endpoint
- **Auto Scaling**: < 5s para avaliação de scaling
- **Health Monitoring**: < 100ms para execução de check

### Otimizações Aplicadas
- **Caching**: Cache de resultados de resolução
- **Connection Pooling**: Pool de conexões HTTP
- **Async Operations**: Operações assíncronas
- **Batch Processing**: Processamento em lote

## 🤝 Contribuição

1. Fork o repositório
2. Crie uma branch para sua feature
3. Faça commit das mudanças
4. Abra um Pull Request

## 📄 Licença

MIT License - veja o arquivo [LICENSE](../../LICENSE) para detalhes.

## 🔗 Links Relacionados

- [Polis Core](../polis-core/README.md)
- [Polis Runtime](../polis-runtime/README.md)
- [Polis API](../polis-api/README.md)
- [Polis Monitor](../polis-monitor/README.md)
- [Documentação Completa](../../docs/README.md)
