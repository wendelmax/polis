# Roadmap do Polis - Container Runtime

## 📋 Status Geral
- [x] Estrutura do monorepo
- [x] Configuração do workspace Rust
- [x] CLI básico funcional
- [x] Arquitetura modular definida
- [x] Tipos de dados básicos (Container, Image, etc.)
- [x] Sistema de erros centralizado
- [x] Estrutura de configuração básica
- [x] Sistema de configuração funcional
- [x] Exemplos de configuração (YAML, TOML, JSON)
- [x] Runtime básico de containers funcional
- [x] CLI integrado com runtime
- [x] Suporte a imagens OCI funcional
- [x] Parser de manifestos OCI
- [x] Sistema de cache de imagens
- [x] Sistema de isolamento e segurança funcional
- [x] Namespaces, Cgroups, Seccomp, Capabilities
- [x] APIs REST/gRPC funcionais
- [x] Endpoints completos para containers, imagens e sistema
- [x] Serviços gRPC especializados
- [x] Testes unitários e de integração
- [x] Estrutura de testes robusta para todos os componentes
- [x] Gerenciamento de rede completo
- [x] Sistema de rede integrado com todos os componentes
- [x] **Concluído:** Sistema de monitoramento
- [x] **Concluído:** Documentação e exemplos

---

## 🎯 Fase 1: Fundação Sólida (Semanas 1-2)

### 1.1 Implementar polis-core funcional
- [x] Sistema de configuração com arquivos YAML/JSON
- [x] Tipos de dados mais robustos (Container, Image, Network)
- [x] Sistema de logging estruturado
- [x] Utilitários para validação e serialização
- [x] Tratamento de erros mais detalhado
- [x] Constantes e enums para estados

### 1.2 Criar sistema de configuração
- [x] Parser para arquivos de configuração
- [x] Validação de configurações
- [x] Configurações padrão
- [x] Override de configurações via CLI
- [x] Configuração de runtime
- [x] Configuração de segurança
- [x] Configuração de rede

---

## 🚀 Fase 2: Runtime Básico (Semanas 3-4)

### 2.1 Implementar runtime de containers
- [x] Criação de containers (sem isolamento ainda)
- [x] Gerenciamento de ciclo de vida
- [x] Execução de processos básicos
- [x] Sistema de estados (Created, Running, Stopped)
- [x] Persistência de estado
- [x] Cleanup de recursos

### 2.2 Adicionar suporte a imagens OCI
- [x] Parser para manifestos OCI
- [x] Download de imagens de registries
- [x] Cache local de imagens
- [x] Extração de layers (básica)
- [x] Validação de integridade (básica)
- [x] Gerenciamento de tags (básico)

---

## 🔒 Fase 3: Segurança e Isolamento (Semanas 5-6)

### 3.1 Implementar isolamento e segurança
- [x] Namespaces Linux (PID, Mount, Network, UTS, IPC, User, Cgroup)
- [x] Cgroups para limitação de recursos
- [x] Seccomp profiles
- [x] Capabilities management
- [x] Gerenciamento de hostname
- [ ] AppArmor profiles
- [ ] SELinux contexts

### 3.2 APIs REST/gRPC
- [x] API REST completa com endpoints para containers, imagens e sistema
- [x] API gRPC com serviços especializados (Container, Image, System)
- [x] Integração total com runtime e image manager
- [x] Tratamento de erros robusto
- [x] Suporte a JSON e Protocol Buffers
- [x] Exemplo demonstrativo funcional

### 3.3 Testes e Qualidade
- [x] Testes unitários para polis-core (configuração, tipos, erros)
- [x] Testes de integração para polis-runtime
- [x] Testes para APIs REST/gRPC
- [x] Testes para sistema de segurança
- [x] Testes para gerenciamento de imagens
- [x] Estrutura de testes robusta e organizada

### 3.4 Gerenciamento de rede
- [x] Criação de bridges com interfaces virtuais
- [x] Configuração de interfaces de rede
- [x] Port forwarding com ranges e mapeamento automático
- [x] DNS resolution com zonas locais
- [x] Firewall rules por container, porta e IP
- [x] IPAM com pools de IP e alocação dinâmica
- [x] Integração completa entre todos os componentes

---

## 🌐 Fase 4: APIs e Integração (Semanas 7-8)

### 4.1 Criar APIs REST/gRPC funcionais
- [x] API REST para gerenciamento de containers
- [x] API gRPC para operações em lote
- [x] Integração total com runtime e image manager
- [x] Tratamento de erros robusto
- [x] Suporte a JSON e Protocol Buffers
- [ ] Autenticação e autorização
- [ ] Documentação OpenAPI
- [ ] Rate limiting
- [ ] Middleware de logging

### 4.2 Adicionar testes e qualidade
- [x] Testes unitários para cada componente
- [x] Testes de integração
- [x] Estrutura de testes robusta e organizada
- [ ] Benchmarks de performance
- [ ] CI/CD pipeline
- [ ] Code coverage
- [ ] Linting e formatação

---

## 📊 Fase 5: Testes e Qualidade (Semanas 9-10)

### 5.1 Sistema de testes completo
- [x] Testes unitários para polis-core (27 testes)
- [x] Testes de integração para polis-runtime (8 testes)
- [x] Testes para APIs REST/gRPC (6 testes)
- [x] Testes para sistema de segurança (8 testes)
- [x] Testes para gerenciamento de imagens (10 testes)
- [x] Estrutura de testes robusta e organizada

### 5.2 Gerenciamento de rede
- [x] Criação de bridges com interfaces virtuais
- [x] Configuração de interfaces de rede
- [x] Port forwarding com ranges e mapeamento automático
- [x] DNS resolution com zonas locais
- [x] Firewall rules por container, porta e IP
- [x] IPAM com pools de IP e alocação dinâmica
- [x] Integração completa entre todos os componentes

---

## 📊 Fase 6: Recursos Avançados (Semanas 11-12)

### 6.1 Sistema de monitoramento ✅ **CONCLUÍDO**
- [x] Coleta de métricas de containers
- [x] Health checks
- [x] Logs centralizados
- [x] Alertas e notificações
- [x] Dashboard web
- [x] Exportação de métricas

### 6.2 Documentação e exemplos
- [x] **CONCLUÍDO:** Documentação completa da API
- [x] **CONCLUÍDO:** Exemplos de uso
- [x] **CONCLUÍDO:** Tutoriais
- [x] **CONCLUÍDO:** Guias de migração do Docker
- [ ] Video tutorials
- [ ] Community guidelines

---

## 🎯 Prioridades Imediatas

### Semana 1: Começar com polis-core
- [ ] **EM ANDAMENTO:** Sistema de configuração
- [ ] Melhorar tipos de dados
- [ ] Adicionar logging estruturado
- [ ] Criar testes básicos

### Semana 2: Runtime básico
- [ ] Implementar criação de containers
- [ ] Gerenciamento de processos
- [ ] Sistema de estados
- [ ] Integração com polis-core

---

## 🛠️ Ferramentas e Recursos Necessários

### Dependências Externas
- [ ] **Linux** (para namespaces e cgroups)
- [ ] **Docker** (para testar compatibilidade)
- [ ] **OCI tools** (para validar conformidade)
- [ ] **Kubernetes** (para testes de orquestração)

### Bibliotecas Rust Importantes
- [x] `oci-spec` - Especificação OCI
- [x] `nix` - System calls Linux
- [x] `cgroups` - Gerenciamento de recursos
- [x] `tokio` - Runtime assíncrono
- [x] `serde` - Serialização
- [ ] `yaml` - Parser YAML
- [ ] `tracing` - Logging estruturado
- [ ] `clap` - CLI parsing

---

## 📈 Métricas de Sucesso

### Funcionalidade
- [ ] Container básico executa comandos
- [ ] Imagens OCI são baixadas e executadas
- [ ] Isolamento de recursos funciona
- [ ] APIs respondem corretamente
- [ ] Compatibilidade com Docker

### Performance
- [ ] Tempo de inicialização < 100ms
- [ ] Uso de memória < 50MB
- [ ] Throughput > 100 containers/min
- [ ] Latência de API < 10ms

### Qualidade
- [ ] Code coverage > 80%
- [ ] Zero warnings de compilação
- [ ] Todos os testes passando
- [ ] Documentação completa

---

## 🚧 Bloqueadores Conhecidos

- [ ] **Linux dependencies** - Algumas funcionalidades só funcionam no Linux
- [ ] **Root privileges** - Algumas operações precisam de privilégios elevados
- [ ] **OCI compliance** - Validação de conformidade com especificação OCI
- [ ] **Performance** - Otimização para edge computing

---

## 📝 Notas de Desenvolvimento

### Decisões Arquiteturais
- **Monorepo:** Facilita desenvolvimento e manutenção
- **Rust:** Performance e segurança de memória
- **Async/await:** Para operações I/O intensivas
- **Modular:** Componentes independentes

### Padrões de Código
- **Português:** Nomes de variáveis em português brasileiro
- **Snake_case:** Para funções e variáveis
- **PascalCase:** Para tipos e estruturas
- **Error handling:** `thiserror` + `anyhow`

---

## 🎉 Milestones

### Milestone 1: MVP Funcional (Semana 4)
- [ ] Container básico executa comandos
- [ ] CLI funcional
- [ ] Configuração básica

### Milestone 2: Isolamento (Semana 6)
- [ ] Namespaces funcionando
- [ ] Cgroups implementados
- [ ] Segurança básica

### Milestone 3: APIs (Semana 8)
- [ ] REST API funcional
- [ ] gRPC API funcional
- [ ] Documentação completa

### Milestone 4: Produção (Semana 12)
- [ ] Monitoramento completo
- [ ] Performance otimizada
- [ ] Documentação completa
- [ ] Exemplos e tutoriais

---

**Última atualização:** 17/09/2025
**Próxima revisão:** 24/09/2025
