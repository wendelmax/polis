# Arquitetura e Integração do Polis

## Visão Geral da Arquitetura

O Polis é construído como um sistema modular onde cada pasta representa um componente especializado que se integra com outros para formar um container runtime completo. A arquitetura segue o padrão de **microserviços internos** com **comunicação assíncrona**.

## 🏗️ Estrutura de Integração

```
┌─────────────────────────────────────────────────────────────────┐
│                        POLIS ECOSYSTEM                         │
├─────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐        │
│  │   polis-cli │    │  polis-api  │    │ polis-sdk   │        │
│  │  (Interface)│    │ (APIs REST/ │    │(Developer   │        │
│  │             │    │    gRPC)    │    │  Tools)     │        │
│  └─────────────┘    └─────────────┘    └─────────────┘        │
│         │                   │                   │              │
│         └───────────────────┼───────────────────┘              │
│                             │                                  │
│  ┌─────────────────────────────────────────────────────────────┤
│  │                    CORE LAYER                              │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  │ polis-core  │    │polis-runtime│    │polis-image  │    │
│  │  │(Fundações)  │    │(Containers) │    │(Imagens OCI)│    │
│  │  └─────────────┘    └─────────────┘    └─────────────┘    │
│  └─────────────────────────────────────────────────────────────┤
│                             │                                  │
│  ┌─────────────────────────────────────────────────────────────┤
│  │                 SPECIALIZED LAYER                         │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  │polis-network│    │polis-security│   │polis-storage│    │
│  │  │   (Rede)    │    │ (Segurança) │    │ (Volumes)   │    │
│  │  └─────────────┘    └─────────────┘    └─────────────┘    │
│  └─────────────────────────────────────────────────────────────┤
│                             │                                  │
│  ┌─────────────────────────────────────────────────────────────┤
│  │                MANAGEMENT LAYER                           │
│  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    │
│  │  │polis-monitor│    │polis-orchestrator│polis-tests │    │
│  │  │(Monitoramento)│  │(Orquestração)│   │(Testes)    │    │
│  │  └─────────────┘    └─────────────┘    └─────────────┘    │
│  └─────────────────────────────────────────────────────────────┘
└─────────────────────────────────────────────────────────────────┘
```

## 🔗 Fluxo de Integração

### 1. **polis-core** - Fundação do Sistema
```rust
// Todas as outras pastas dependem do polis-core
use polis_core::{PolisConfig, ContainerId, ImageId, Result, PolisError};
```

**Responsabilidades:**
- Tipos de dados compartilhados (Container, Image, Network, Volume)
- Sistema de configuração centralizado
- Tratamento de erros padronizado
- Utilitários comuns (logging, serialização, validação)

**Integração:**
- **Fornece** tipos e utilitários para todas as outras pastas
- **Não depende** de nenhuma outra pasta
- **É a base** de toda a arquitetura

### 2. **polis-runtime** - Motor de Containers
```rust
// Depende do polis-core e integra com outras pastas
use polis_core::{PolisConfig, ContainerId, Result};
use polis_security::SecurityManager;
use polis_network::NetworkManager;
use polis_storage::VolumeManager;
```

**Responsabilidades:**
- Criação e gerenciamento de containers
- Ciclo de vida (Created → Running → Stopped → Removed)
- Execução de processos
- Orquestração de outros componentes

**Integração:**
- **Usa** polis-core para tipos e configuração
- **Coordena** polis-security para isolamento
- **Coordena** polis-network para conectividade
- **Coordena** polis-storage para volumes
- **Fornece** dados para polis-monitor

### 3. **polis-image** - Gerenciamento de Imagens
```rust
// Depende do polis-core e integra com polis-runtime
use polis_core::{Image, ImageId, Result};
use polis_runtime::ContainerRuntime;
```

**Responsabilidades:**
- Download de imagens OCI
- Cache local de imagens
- Validação de manifestos
- Extração de layers

**Integração:**
- **Usa** polis-core para tipos de imagem
- **Fornece** imagens para polis-runtime
- **Integra** com polis-api para exposição via API

### 4. **polis-security** - Isolamento e Segurança
```rust
// Depende do polis-core e é usado por polis-runtime
use polis_core::{Result, ResourceLimits};
use polis_runtime::ContainerRuntime;
```

**Responsabilidades:**
- Namespaces Linux (PID, Mount, Network, UTS, IPC, User, Cgroup)
- Cgroups para limitação de recursos
- Seccomp profiles para restrição de syscalls
- Capabilities management

**Integração:**
- **Usa** polis-core para tipos de recursos
- **É usado** por polis-runtime para isolamento
- **Fornece** dados de segurança para polis-monitor

### 5. **polis-network** - Gerenciamento de Rede
```rust
// Depende do polis-core e é usado por polis-runtime
use polis_core::{Result, NetworkId};
use polis_runtime::ContainerRuntime;
```

**Responsabilidades:**
- Criação de bridges virtuais
- IPAM (IP Address Management)
- Firewall rules
- DNS resolution
- Port forwarding

**Integração:**
- **Usa** polis-core para tipos de rede
- **É usado** por polis-runtime para conectividade
- **Fornece** dados de rede para polis-monitor

### 6. **polis-storage** - Gerenciamento de Volumes
```rust
// Depende do polis-core e é usado por polis-runtime
use polis_core::{Result, VolumeId, VolumeMount};
use polis_runtime::ContainerRuntime;
```

**Responsabilidades:**
- Criação e gerenciamento de volumes
- Montagem de volumes em containers
- Drivers de armazenamento
- Backup e sincronização

**Integração:**
- **Usa** polis-core para tipos de volume
- **É usado** por polis-runtime para persistência
- **Fornece** dados de armazenamento para polis-monitor

### 7. **polis-api** - APIs REST e gRPC
```rust
// Depende de polis-core e integra com polis-runtime e polis-image
use polis_core::{PolisConfig, Result};
use polis_runtime::{PolisRuntime, ContainerRuntime};
use polis_image::ImageManager;
```

**Responsabilidades:**
- API REST para gerenciamento via HTTP
- API gRPC para operações de alta performance
- Serialização JSON e Protocol Buffers
- Tratamento de requisições

**Integração:**
- **Usa** polis-core para tipos e configuração
- **Usa** polis-runtime para operações de containers
- **Usa** polis-image para operações de imagens
- **Expõe** funcionalidades para polis-cli e polis-sdk

### 8. **polis-cli** - Interface de Linha de Comando
```rust
// Depende de polis-core e usa polis-api
use polis_core::{PolisConfig, Result};
use polis_api::{RestServer, GrpcServer};
```

**Responsabilidades:**
- Interface de linha de comando
- Parsing de argumentos
- Comunicação com APIs
- Experiência do usuário

**Integração:**
- **Usa** polis-core para configuração
- **Usa** polis-api para operações
- **Fornece** interface para usuários finais

### 9. **polis-monitor** - Monitoramento e Observabilidade
```rust
// Depende de polis-core e coleta dados de outras pastas
use polis_core::{Result, ContainerId};
use polis_runtime::PolisRuntime;
use polis_network::NetworkManager;
use polis_security::SecurityManager;
```

**Responsabilidades:**
- Coleta de métricas de sistema e containers
- Health checks
- Logs centralizados
- Dashboard web
- Alertas e notificações

**Integração:**
- **Usa** polis-core para tipos
- **Coleta** dados de polis-runtime
- **Coleta** dados de polis-network
- **Coleta** dados de polis-security
- **Fornece** observabilidade para todo o sistema

### 10. **polis-orchestrator** - Orquestração e Agendamento
```rust
// Depende de polis-core e coordena outras pastas
use polis_core::{PolisConfig, Result};
use polis_runtime::PolisRuntime;
use polis_network::NetworkManager;
use polis_monitor::MonitorManager;
```

**Responsabilidades:**
- Orquestração de containers
- Agendamento de recursos
- Balanceamento de carga
- Escalabilidade automática

**Integração:**
- **Usa** polis-core para configuração
- **Coordena** polis-runtime para containers
- **Coordena** polis-network para conectividade
- **Usa** polis-monitor para decisões de escala

### 11. **polis-sdk** - SDK para Desenvolvedores
```rust
// Depende de polis-core e polis-api
use polis_core::{PolisConfig, Result};
use polis_api::{RestClient, GrpcClient};
```

**Responsabilidades:**
- SDK para desenvolvedores
- Bibliotecas de cliente
- Exemplos e documentação
- Ferramentas de desenvolvimento

**Integração:**
- **Usa** polis-core para tipos
- **Usa** polis-api para comunicação
- **Fornece** ferramentas para desenvolvedores

### 12. **polis-tests** - Testes de Integração
```rust
// Depende de todas as pastas para testes
use polis_core::PolisConfig;
use polis_runtime::PolisRuntime;
use polis_api::{RestServer, GrpcServer};
// ... outras dependências
```

**Responsabilidades:**
- Testes de integração end-to-end
- Testes de performance
- Testes de compatibilidade
- Benchmarks

**Integração:**
- **Usa** todas as pastas para testes
- **Valida** integração entre componentes
- **Garante** qualidade do sistema

## 🔄 Fluxo de Dados

### 1. **Criação de Container**
```
polis-cli → polis-api → polis-runtime → polis-security + polis-network + polis-storage
```

### 2. **Download de Imagem**
```
polis-cli → polis-api → polis-image → polis-runtime
```

### 3. **Monitoramento**
```
polis-monitor ← polis-runtime + polis-network + polis-security + polis-storage
```

### 4. **Orquestração**
```
polis-orchestrator → polis-runtime + polis-network + polis-monitor
```

## 📊 Dependências entre Pastas

```
polis-core (base)
    ↑
    ├── polis-runtime
    ├── polis-image
    ├── polis-security
    ├── polis-network
    ├── polis-storage
    ├── polis-api
    ├── polis-monitor
    ├── polis-orchestrator
    ├── polis-sdk
    └── polis-tests

polis-runtime
    ↑
    ├── polis-api
    ├── polis-monitor
    ├── polis-orchestrator
    └── polis-tests

polis-image
    ↑
    ├── polis-api
    ├── polis-monitor
    └── polis-tests

polis-security
    ↑
    ├── polis-runtime
    ├── polis-monitor
    └── polis-tests

polis-network
    ↑
    ├── polis-runtime
    ├── polis-monitor
    ├── polis-orchestrator
    └── polis-tests

polis-storage
    ↑
    ├── polis-runtime
    ├── polis-monitor
    └── polis-tests

polis-api
    ↑
    ├── polis-cli
    ├── polis-sdk
    └── polis-tests

polis-monitor
    ↑
    ├── polis-orchestrator
    └── polis-tests

polis-orchestrator
    ↑
    └── polis-tests
```

## 🚀 Exemplo de Integração Completa

### Cenário: Criar e Executar um Container

```rust
// 1. polis-cli recebe comando
let command = "polis create --name web --image nginx:alpine --port 8080:80";

// 2. polis-cli chama polis-api
let container_id = api_client.create_container(request).await?;

// 3. polis-api chama polis-runtime
let container_id = runtime.create_container(name, image, command).await?;

// 4. polis-runtime coordena outros componentes
// 4.1. polis-image baixa a imagem
let image = image_manager.pull("nginx:alpine").await?;

// 4.2. polis-security cria isolamento
security_manager.create_namespaces(&container_id).await?;
security_manager.create_cgroup(&container_id, &resource_limits).await?;

// 4.3. polis-network configura conectividade
network_manager.setup_container_network(&container_id, &ip).await?;
port_manager.create_port_forwarding(8080, 80, &container_id).await?;

// 4.4. polis-storage monta volumes (se necessário)
volume_manager.mount_volumes(&container_id, &volume_mounts).await?;

// 5. polis-runtime executa o container
runtime.start_container(container_id).await?;

// 6. polis-monitor coleta métricas
monitor_manager.start_monitoring(&container_id).await?;
```

## 🎯 Benefícios da Arquitetura Modular

### 1. **Separação de Responsabilidades**
- Cada pasta tem uma responsabilidade específica
- Fácil manutenção e evolução
- Testes independentes

### 2. **Reutilização de Código**
- polis-core fornece base comum
- Componentes podem ser reutilizados
- Redução de duplicação

### 3. **Escalabilidade**
- Componentes podem ser escalados independentemente
- Fácil adição de novos recursos
- Arquitetura flexível

### 4. **Testabilidade**
- Testes unitários por componente
- Testes de integração entre componentes
- Isolamento de falhas

### 5. **Desenvolvimento Paralelo**
- Equipes podem trabalhar em componentes diferentes
- Redução de conflitos
- Desenvolvimento mais ágil

## 🔧 Configuração de Integração

### Cargo.toml (Workspace)
```toml
[workspace]
members = [
    "polis-core",
    "polis-runtime", 
    "polis-image",
    "polis-network",
    "polis-security",
    "polis-storage",
    "polis-api",
    "polis-cli",
    "polis-orchestrator",
    "polis-monitor",
    "polis-sdk",
    "polis-tests"
]

[workspace.dependencies]
polis-core = { path = "polis-core" }
polis-runtime = { path = "polis-runtime" }
# ... outras dependências
```

### Exemplo de Dependência
```toml
# polis-runtime/Cargo.toml
[dependencies]
polis-core = { workspace = true }
polis-security = { workspace = true }
polis-network = { workspace = true }
polis-storage = { workspace = true }
```

## 📈 Monitoramento da Integração

### Métricas de Integração
- **Latência** entre componentes
- **Throughput** de comunicação
- **Taxa de erro** nas integrações
- **Uso de recursos** por componente

### Health Checks
- **polis-core**: Configuração válida
- **polis-runtime**: Containers funcionando
- **polis-network**: Conectividade OK
- **polis-security**: Isolamento ativo
- **polis-monitor**: Coleta de dados OK

## 🎉 Conclusão

A arquitetura modular do Polis permite que cada pasta trabalhe de forma independente enquanto se integra perfeitamente com outras para formar um sistema coeso e robusto. Essa abordagem garante:

- **Manutenibilidade**: Fácil de manter e evoluir
- **Escalabilidade**: Pode crescer conforme necessário
- **Testabilidade**: Fácil de testar e validar
- **Flexibilidade**: Pode ser adaptado para diferentes casos de uso
- **Performance**: Otimizado para cada componente

O resultado é um container runtime moderno, seguro e eficiente que rivaliza com as melhores soluções do mercado! 🚀

