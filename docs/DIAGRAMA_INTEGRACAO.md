# Diagrama de Integração do Polis

## 🏗️ Arquitetura Visual

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                              POLIS ECOSYSTEM                                  │
├─────────────────────────────────────────────────────────────────────────────────┤
│                                                                                 │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │   polis-cli │    │  polis-api  │    │ polis-sdk   │    │polis-tests  │    │
│  │  (Interface)│    │ (APIs REST/ │    │(Developer   │    │(Testes      │    │
│  │             │    │    gRPC)    │    │  Tools)     │    │Integração)  │    │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘    │
│         │                   │                   │                   │        │
│         └───────────────────┼───────────────────┼───────────────────┘        │
│                             │                   │                            │
│  ┌─────────────────────────────────────────────────────────────────────────────┤
│  │                           CORE LAYER                                      │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  │ polis-core  │    │polis-runtime│    │polis-image  │    │polis-monitor│ │
│  │  │(Fundações)  │    │(Containers) │    │(Imagens OCI)│    │(Monitoramento)│
│  │  │             │    │             │    │             │    │             │ │
│  │  │ • Tipos     │    │ • Criação   │    │ • Download  │    │ • Métricas  │ │
│  │  │ • Config    │    │ • Execução  │    │ • Cache     │    │ • Health    │ │
│  │  │ • Erros     │    │ • Lifecycle │    │ • Validação │    │ • Logs      │ │
│  │  │ • Utils     │    │ • Estado    │    │ • Layers    │    │ • Dashboard │ │
│  │  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
│  └─────────────────────────────────────────────────────────────────────────────┤
│                             │                   │                            │
│  ┌─────────────────────────────────────────────────────────────────────────────┤
│  │                      SPECIALIZED LAYER                                   │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐ │
│  │  │polis-network│    │polis-security│   │polis-storage│    │polis-orchestrator│
│  │  │   (Rede)    │    │ (Segurança) │    │ (Volumes)   │    │(Orquestração) │
│  │  │             │    │             │    │             │    │             │ │
│  │  │ • Bridges   │    │ • Namespaces│    │ • Volumes   │    │ • Scheduling│ │
│  │  │ • IPAM      │    │ • Cgroups   │    │ • Mounts    │    │ • Scaling   │ │
│  │  │ • Firewall  │    │ • Seccomp   │    │ • Drivers   │    │ • Load Bal. │ │
│  │  │ • DNS       │    │ • Capabilities│   │ • Backup    │    │ • Auto Heal │ │
│  │  │ • Port Fwd  │    │ • Isolation │    │ • Sync      │    │ • Policies  │ │
│  │  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘ │
│  └─────────────────────────────────────────────────────────────────────────────┤
└─────────────────────────────────────────────────────────────────────────────────┘
```

## 🔄 Fluxo de Dados Detalhado

### 1. Criação de Container
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   polis-cli │───▶│  polis-api  │───▶│polis-runtime│
│             │    │             │    │             │
│ • Parse cmd │    │ • REST/gRPC │    │ • Create    │
│ • Validate  │    │ • Auth      │    │ • Configure │
│ • Call API  │    │ • Serialize │    │ • Execute   │
└─────────────┘    └─────────────┘    └─────────────┘
                                              │
                                              ▼
                    ┌─────────────────────────────────┐
                    │        COORDENAÇÃO              │
                    │                                 │
                    │  ┌─────────────┐  ┌─────────────┐
                    │  │polis-security│  │polis-network│
                    │  │             │  │             │
                    │  │ • Namespaces│  │ • Bridge    │
                    │  │ • Cgroups   │  │ • IPAM      │
                    │  │ • Seccomp   │  │ • Port Fwd  │
                    │  └─────────────┘  └─────────────┘
                    │                                 │
                    │  ┌─────────────┐  ┌─────────────┐
                    │  │polis-storage│  │polis-image  │
                    │  │             │  │             │
                    │  │ • Volumes   │  │ • Download  │
                    │  │ • Mounts    │  │ • Cache     │
                    │  └─────────────┘  └─────────────┘
                    └─────────────────────────────────┘
                                              │
                                              ▼
                    ┌─────────────────────────────────┐
                    │        MONITORAMENTO            │
                    │                                 │
                    │  ┌─────────────┐               │
                    │  │polis-monitor│               │
                    │  │             │               │
                    │  │ • Métricas  │               │
                    │  │ • Health    │               │
                    │  │ • Logs      │               │
                    │  │ • Alerts    │               │
                    │  └─────────────┘               │
                    └─────────────────────────────────┘
```

### 2. Download de Imagem
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   polis-cli │───▶│  polis-api  │───▶│polis-image  │
│             │    │             │    │             │
│ • pull cmd  │    │ • REST/gRPC │    │ • Registry  │
│ • image name│    │ • Validate  │    │ • Download  │
│ • call API  │    │ • Serialize │    │ • Cache     │
└─────────────┘    └─────────────┘    └─────────────┘
                                              │
                                              ▼
                    ┌─────────────────────────────────┐
                    │        INTEGRAÇÃO               │
                    │                                 │
                    │  ┌─────────────┐               │
                    │  │polis-runtime│               │
                    │  │             │               │
                    │  │ • Use image │               │
                    │  │ • Create    │               │
                    │  │ • Execute   │               │
                    │  └─────────────┘               │
                    └─────────────────────────────────┘
```

### 3. Monitoramento
```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│polis-monitor│◀───│polis-runtime│    │polis-network│
│             │    │             │    │             │
│ • Collect   │    │ • Metrics   │    │ • Stats     │
│ • Process   │    │ • Events    │    │ • Traffic   │
│ • Store     │    │ • Logs      │    │ • Errors    │
└─────────────┘    └─────────────┘    └─────────────┘
       ▲                   │                   │
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
                    ┌─────────────┐
                    │polis-security│
                    │             │
                    │ • Security  │
                    │ • Isolation │
                    │ • Violations│
                    └─────────────┘
```

## 🔗 Matriz de Dependências

```
                    ┌─────────────────────────────────────────────────────────────┐
                    │                    DEPENDÊNCIAS                            │
                    ├─────────────────────────────────────────────────────────────┤
│                   │ polis- │ polis- │ polis- │ polis- │ polis- │ polis- │ polis- │
│                   │ core  │runtime │ image  │network │security│storage │  api  │
├───────────────────┼───────┼────────┼────────┼────────┼────────┼────────┼───────┤
│ polis-core        │   -   │   ✓    │   ✓    │   ✓    │   ✓    │   ✓    │   ✓   │
│ polis-runtime     │   ✓   │   -    │   ✓    │   ✓    │   ✓    │   ✓    │   ✓   │
│ polis-image       │   ✓   │   ✓    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-network     │   ✓   │   ✓    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-security    │   ✓   │   ✓    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-storage     │   ✓   │   ✓    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-api         │   ✓   │   ✓    │   ✓    │   -    │   -    │   -    │   -   │
│ polis-cli         │   ✓   │   -    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-monitor     │   ✓   │   ✓    │   -    │   ✓    │   ✓    │   ✓    │   -   │
│ polis-orchestrator│   ✓   │   ✓    │   -    │   ✓    │   -    │   -    │   ✓   │
│ polis-sdk         │   ✓   │   -    │   -    │   -    │   -    │   -    │   ✓   │
│ polis-tests       │   ✓   │   ✓    │   ✓    │   ✓    │   ✓    │   ✓    │   ✓   │
└───────────────────┴───────┴────────┴────────┴────────┴────────┴────────┴───────┘
```

## 🎯 Pontos de Integração

### 1. **Configuração Centralizada**
```rust
// polis-core fornece configuração para todos
let config = PolisConfig::load()?;
// Usado por: polis-runtime, polis-api, polis-monitor, etc.
```

### 2. **Tipos Compartilhados**
```rust
// polis-core define tipos usados por todos
pub struct Container { ... }
pub struct Image { ... }
pub struct Network { ... }
// Usado por: todas as pastas
```

### 3. **Sistema de Erros**
```rust
// polis-core define erros padronizados
pub enum PolisError { ... }
// Usado por: todas as pastas
```

### 4. **Logging Estruturado**
```rust
// polis-core fornece logging
tracing::info!("Container created: {}", container_id);
// Usado por: todas as pastas
```

### 5. **Comunicação Assíncrona**
```rust
// polis-runtime coordena outros componentes
let security_result = security_manager.create_namespaces(id).await?;
let network_result = network_manager.setup_network(id).await?;
let storage_result = storage_manager.mount_volumes(id).await?;
```

## 🚀 Exemplo Prático de Integração

### Cenário: Deploy de Aplicação Web

```rust
// 1. polis-cli recebe comando
let cmd = "polis run -d --name webapp -p 8080:80 nginx:alpine";

// 2. polis-cli chama polis-api
let response = api_client.post("/containers", request).await?;

// 3. polis-api valida e chama polis-runtime
let container_id = runtime.create_container(name, image, command).await?;

// 4. polis-runtime coordena outros componentes
async fn create_container(name: String, image: String, command: Vec<String>) -> Result<ContainerId> {
    // 4.1. Baixar imagem via polis-image
    let image_info = image_manager.pull(&image).await?;
    
    // 4.2. Configurar segurança via polis-security
    security_manager.create_namespaces(&container_id).await?;
    security_manager.create_cgroup(&container_id, &resource_limits).await?;
    
    // 4.3. Configurar rede via polis-network
    let ip = network_manager.allocate_ip(&container_id).await?;
    network_manager.setup_bridge(&container_id, &ip).await?;
    port_manager.create_forwarding(8080, 80, &container_id).await?;
    
    // 4.4. Configurar volumes via polis-storage (se necessário)
    if !volume_mounts.is_empty() {
        storage_manager.mount_volumes(&container_id, &volume_mounts).await?;
    }
    
    // 4.5. Executar container
    let process_id = process_manager.spawn(&container_id, &command).await?;
    
    // 4.6. Iniciar monitoramento via polis-monitor
    monitor_manager.start_monitoring(&container_id).await?;
    
    Ok(container_id)
}

// 5. polis-monitor coleta métricas
monitor_manager.collect_metrics(&container_id).await?;

// 6. polis-orchestrator pode escalar se necessário
orchestrator.check_scaling_policies().await?;
```

## 📊 Métricas de Integração

### Performance
- **Latência** entre componentes: < 1ms
- **Throughput** de operações: > 1000 ops/s
- **Uso de memória** por componente: < 50MB
- **CPU overhead** de integração: < 5%

### Confiabilidade
- **Taxa de erro** nas integrações: < 0.1%
- **Disponibilidade** do sistema: > 99.9%
- **Recuperação** de falhas: < 1s
- **Consistência** de dados: 100%

## 🎉 Conclusão

A integração entre as pastas do Polis é baseada em:

1. **polis-core** como fundação comum
2. **polis-runtime** como orquestrador principal
3. **Componentes especializados** para funcionalidades específicas
4. **APIs padronizadas** para comunicação
5. **Monitoramento integrado** para observabilidade

Essa arquitetura garante que o Polis seja:
- **Modular**: Cada pasta tem responsabilidade específica
- **Escalável**: Componentes podem crescer independentemente
- **Confiável**: Falhas são isoladas e recuperáveis
- **Performático**: Otimizado para cada caso de uso
- **Manutenível**: Fácil de evoluir e corrigir

O resultado é um container runtime de classe mundial! 🚀

