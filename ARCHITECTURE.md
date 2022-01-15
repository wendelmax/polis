# Arquitetura do Polis

## Visão Geral

O Polis é uma plataforma de containers moderna construída em Rust, organizada como um monorepo modular. Cada componente tem responsabilidades específicas e pode ser desenvolvido independentemente.

## Estrutura do Monorepo

```
polis/
├── polis-core/          # Tipos fundamentais e utilitários
├── polis-runtime/       # Runtime de containers (OCI-compliant)
├── polis-api/           # APIs REST e gRPC
├── polis-cli/           # Interface de linha de comando
├── polis-image/         # Gerenciamento de imagens
├── polis-network/       # Rede e conectividade
├── polis-security/      # Sandboxing e isolamento
├── polis-storage/       # Gerenciamento de volumes
├── polis-orchestrator/  # Orquestração e scheduling
├── polis-monitor/       # Monitoramento e observabilidade
├── polis-sdk/           # SDK para desenvolvedores
└── polis-tests/         # Testes de integração
```

## Componentes Principais

### polis-core
**Responsabilidade:** Tipos fundamentais, erros, configurações e utilitários compartilhados.

**Dependências:** Apenas bibliotecas externas básicas.

**Módulos:**
- `types.rs` - Tipos de dados principais (Container, Image, etc.)
- `error.rs` - Sistema de erros centralizado
- `config.rs` - Estruturas de configuração
- `utils.rs` - Funções utilitárias

### polis-runtime
**Responsabilidade:** Runtime de containers OCI-compliant.

**Dependências:** polis-core, polis-security, polis-storage, polis-network

**Módulos:**
- `runtime.rs` - Interface principal do runtime
- `container.rs` - Gerenciamento de containers
- `process.rs` - Gerenciamento de processos

### polis-api
**Responsabilidade:** APIs REST e gRPC para interação externa.

**Dependências:** polis-core, polis-runtime, polis-image, polis-orchestrator

**Módulos:**
- `rest/` - API REST
- `grpc/` - API gRPC
- `handlers/` - Handlers de requisições

### polis-cli
**Responsabilidade:** Interface de linha de comando.

**Dependências:** polis-core, polis-api

**Comandos:**
- `container` - Gerenciamento de containers
- `image` - Gerenciamento de imagens
- `system` - Informações do sistema

## Fluxo de Dados

```
CLI/API → polis-api → polis-runtime → polis-security/polis-storage/polis-network
```

## Vantagens da Arquitetura Modular

1. **Desenvolvimento Paralelo:** Equipes podem trabalhar em componentes independentes
2. **Testabilidade:** Cada componente pode ser testado isoladamente
3. **Reutilização:** Componentes podem ser reutilizados em outros projetos
4. **Manutenibilidade:** Mudanças em um componente não afetam outros
5. **Escalabilidade:** Novos componentes podem ser adicionados facilmente

## Padrões de Design

### Dependency Injection
Componentes recebem dependências através de construtores, facilitando testes e configuração.

### Async/Await
Toda operação I/O é assíncrona usando Tokio.

### Error Handling
Sistema centralizado de erros com `thiserror` e `anyhow`.

### Configuration
Configuração centralizada em `polis-core` com serialização JSON.

## Próximos Passos

1. Implementar funcionalidades básicas em cada componente
2. Criar testes de integração
3. Implementar APIs REST/gRPC
4. Adicionar suporte a imagens OCI
5. Implementar isolamento de containers

