# Documentação do Polis

Bem-vindo à documentação completa do Polis Container Runtime! Esta documentação foi projetada para ajudar desenvolvedores, DevOps e usuários a entender, instalar e usar o Polis de forma eficiente.

## 📚 Índice da Documentação

### 🚀 Guias de Início
- [Tutorial Completo](TUTORIAL.md) - Guia passo a passo para começar
- [Guia de Instalação](INSTALLATION.md) - Como instalar o Polis
- [Exemplos Práticos](examples/) - Código de exemplo e demonstrações

### 🔧 Referências Técnicas
- [Referência da API REST](API_REST.md) - Documentação completa da API REST
- [Referência da API gRPC](API_GRPC.md) - Documentação completa da API gRPC
- [Arquitetura do Sistema](ARCHITECTURE.md) - Visão técnica da arquitetura

### 🔄 Migração e Integração
- [Guia de Migração do Docker](MIGRATION_DOCKER.md) - Como migrar do Docker
- [Integração com Kubernetes](KUBERNETES.md) - Usando Polis com K8s
- [CI/CD Integration](CICD.md) - Integração com pipelines

### 🛠️ Desenvolvimento
- [Guia do Desenvolvedor](DEVELOPER.md) - Como contribuir
- [SDK e Bibliotecas](SDK.md) - SDKs disponíveis
- [Testes e Qualidade](TESTING.md) - Estratégias de teste

### 🔒 Segurança
- [Guia de Segurança](SECURITY.md) - Práticas de segurança
- [Isolamento e Sandboxing](ISOLATION.md) - Recursos de isolamento
- [Auditoria e Compliance](COMPLIANCE.md) - Conformidade e auditoria

### 📊 Monitoramento e Observabilidade
- [Sistema de Monitoramento](MONITORING.md) - Métricas e observabilidade
- [Logs e Debugging](LOGGING.md) - Gerenciamento de logs
- [Alertas e Notificações](ALERTS.md) - Sistema de alertas

### 🌐 Rede e Conectividade
- [Gerenciamento de Rede](NETWORKING.md) - Configuração de redes
- [Firewall e Segurança](FIREWALL.md) - Regras de firewall
- [DNS e Resolução](DNS.md) - Configuração de DNS

### 📦 Armazenamento
- [Gerenciamento de Volumes](STORAGE.md) - Volumes e persistência
- [Backup e Recuperação](BACKUP.md) - Estratégias de backup
- [Sincronização](SYNC.md) - Sincronização de dados

### 🚀 Performance e Otimização
- [Guia de Performance](PERFORMANCE.md) - Otimização de performance
- [Benchmarks](BENCHMARKS.md) - Comparações de performance
- [Troubleshooting](TROUBLESHOOTING.md) - Resolução de problemas

### 🏢 Enterprise e Produção
- [Guia de Produção](PRODUCTION.md) - Deploy em produção
- [Alta Disponibilidade](HA.md) - Configurações de HA
- [Escalabilidade](SCALABILITY.md) - Estratégias de escala

### 🤝 Comunidade e Suporte
- [FAQ](FAQ.md) - Perguntas frequentes
- [Changelog](CHANGELOG.md) - Histórico de mudanças
- [Contribuindo](CONTRIBUTING.md) - Como contribuir
- [Código de Conduta](CODE_OF_CONDUCT.md) - Normas da comunidade

## 🎯 Como Usar Esta Documentação

### Para Iniciantes
1. Comece com o [Tutorial Completo](TUTORIAL.md)
2. Siga o [Guia de Instalação](INSTALLATION.md)
3. Experimente os [Exemplos Práticos](examples/)

### Para Desenvolvedores
1. Leia a [Arquitetura do Sistema](ARCHITECTURE.md)
2. Consulte as [Referências da API](API_REST.md)
3. Explore o [Guia do Desenvolvedor](DEVELOPER.md)

### Para DevOps
1. Veja o [Guia de Produção](PRODUCTION.md)
2. Configure o [Sistema de Monitoramento](MONITORING.md)
3. Implemente [Estratégias de Backup](BACKUP.md)

### Para Migração
1. Consulte o [Guia de Migração do Docker](MIGRATION_DOCKER.md)
2. Use os [Scripts de Migração](MIGRATION_DOCKER.md#scripts-de-migração)
3. Teste em ambiente de desenvolvimento

## 🔍 Busca Rápida

### Comandos Essenciais
```bash
# Instalação
polis init

# Containers
polis create --name app --image nginx:alpine
polis start app
polis list
polis logs app

# Imagens
polis pull alpine:latest
polis images

# Rede
polis network create --name mynet --subnet 172.20.0.0/16
polis network list

# Monitoramento
polis metrics system
polis health
```

### APIs Principais
```bash
# REST API
curl http://localhost:8080/api/v1/health
curl http://localhost:8080/api/v1/containers

# gRPC API
grpcurl -plaintext localhost:9090 list
```

## 📖 Convenções da Documentação

### Símbolos Utilizados
- ✅ **Concluído** - Funcionalidade implementada
- 🔄 **Em Desenvolvimento** - Funcionalidade em progresso
- ⏳ **Planejado** - Funcionalidade futura
- ⚠️ **Atenção** - Informação importante
- 💡 **Dica** - Sugestão útil
- 🚨 **Aviso** - Cuidado necessário

### Formato dos Códigos
- **Bash**: Comandos do terminal
- **Rust**: Código Rust
- **YAML**: Arquivos de configuração
- **JSON**: Respostas de API
- **Protobuf**: Definições gRPC

## 🆕 Novidades

### Versão 0.1.0 (Janeiro 2025)
- ✅ Sistema completo de containers
- ✅ APIs REST e gRPC
- ✅ Monitoramento avançado
- ✅ Documentação completa
- ✅ Exemplos práticos

### Próximas Versões
- 🔄 Interface web
- 🔄 Suporte a Windows
- 🔄 Plugins e extensões
- 🔄 Orquestração avançada

## 🤝 Contribuindo com a Documentação

A documentação do Polis é um projeto vivo e sempre em evolução. Suas contribuições são bem-vindas!

### Como Contribuir
1. **Reportar Problemas**: Use o [GitHub Issues](https://github.com/polis/polis/issues)
2. **Sugerir Melhorias**: Abra uma issue com a tag `documentation`
3. **Enviar Correções**: Faça um pull request
4. **Adicionar Conteúdo**: Contribua com novos guias e exemplos

### Padrões de Contribuição
- Use português brasileiro
- Siga o formato markdown
- Inclua exemplos práticos
- Teste os comandos antes de documentar
- Mantenha a consistência com o resto da documentação

## 📞 Suporte

### Canais de Suporte
- **GitHub Issues**: [github.com/polis/polis/issues](https://github.com/polis/polis/issues)
- **Discord**: [discord.gg/polis](https://discord.gg/polis)
- **Stack Overflow**: [stackoverflow.com/tags/polis](https://stackoverflow.com/tags/polis)
- **Email**: support@polis.dev

### Recursos Adicionais
- **Blog**: [blog.polis.dev](https://blog.polis.dev)
- **Tutoriais em Vídeo**: [youtube.com/polis](https://youtube.com/polis)
- **Webinars**: [webinars.polis.dev](https://webinars.polis.dev)

## 📄 Licença

Esta documentação está licenciada sob a [Licença MIT](LICENSE). Você pode usar, modificar e distribuir livremente.

---

**Última atualização**: Janeiro 2025  
**Versão da documentação**: 1.0.0  
**Status**: Ativa e mantida

**Polis** - Container Runtime moderno, seguro e eficiente. Feito com ❤️ no Brasil.

