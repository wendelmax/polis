# Community Guidelines - Polis

## 🤝 Bem-vindo à Comunidade Polis!

A comunidade Polis é um espaço acolhedor e inclusivo onde desenvolvedores, usuários e contribuidores se reúnem para construir e melhorar o Polis, um container runtime moderno e seguro.

## 📋 Código de Conduta

### Nossos Valores
- **Respeito**: Trate todos os membros da comunidade com respeito e cortesia
- **Inclusão**: Seja acolhedor e inclusivo, independentemente de background ou experiência
- **Colaboração**: Trabalhe juntos para resolver problemas e melhorar o projeto
- **Aprendizado**: Compartilhe conhecimento e ajude outros a aprender
- **Transparência**: Seja aberto e honesto em todas as interações

### Comportamento Esperado
- ✅ Use linguagem respeitosa e construtiva
- ✅ Seja paciente com iniciantes e perguntas básicas
- ✅ Compartilhe conhecimento e recursos úteis
- ✅ Critique ideias, não pessoas
- ✅ Ajude a manter um ambiente positivo e produtivo

### Comportamento Inaceitável
- ❌ Linguagem ofensiva, discriminatória ou abusiva
- ❌ Spam, trolling ou comportamento disruptivo
- ❌ Assédio de qualquer tipo
- ❌ Compartilhamento de conteúdo inadequado
- ❌ Violação de privacidade ou confidencialidade

## 🚀 Como Contribuir

### 1. Reportando Issues

#### Antes de Criar um Issue
- [ ] Verifique se o problema já foi reportado
- [ ] Consulte a documentação e FAQ
- [ ] Teste com a versão mais recente
- [ ] Colete informações relevantes

#### Template para Issues
```markdown
**Descrição**
Uma descrição clara e concisa do problema.

**Passos para Reproduzir**
1. Vá para '...'
2. Clique em '...'
3. Role até '...'
4. Veja o erro

**Comportamento Esperado**
O que você esperava que acontecesse.

**Comportamento Atual**
O que realmente aconteceu.

**Informações do Sistema**
- OS: [ex: Ubuntu 20.04]
- Versão do Polis: [ex: 0.1.0]
- Versão do Rust: [ex: 1.70.0]

**Logs e Screenshots**
Se aplicável, adicione logs e screenshots.

**Contexto Adicional**
Qualquer outra informação relevante.
```

### 2. Propondo Mudanças

#### Pull Requests
- [ ] Fork o repositório
- [ ] Crie uma branch para sua feature
- [ ] Faça commits descritivos
- [ ] Adicione testes se necessário
- [ ] Atualize a documentação
- [ ] Abra um Pull Request

#### Template para Pull Requests
```markdown
**Descrição**
Uma descrição clara do que este PR faz.

**Tipo de Mudança**
- [ ] Bug fix
- [ ] Nova feature
- [ ] Breaking change
- [ ] Documentação
- [ ] Refatoração

**Como Testar**
Passos para testar as mudanças.

**Checklist**
- [ ] Código segue o style guide
- [ ] Testes passam
- [ ] Documentação atualizada
- [ ] Changelog atualizado

**Screenshots**
Se aplicável, adicione screenshots.

**Contexto Adicional**
Qualquer informação adicional relevante.
```

### 3. Discussões e Suporte

#### Fórum de Discussão
- Use o [GitHub Discussions](https://github.com/polis-project/polis/discussions)
- Categorize sua pergunta adequadamente
- Seja específico e forneça contexto
- Use markdown para formatação

#### Canais de Comunicação
- **GitHub Issues**: Bugs e feature requests
- **GitHub Discussions**: Perguntas e discussões gerais
- **Discord**: Chat em tempo real (link no README)
- **Email**: contato@polis-project.org

## 📚 Guias de Contribuição

### 1. Desenvolvimento

#### Configuração do Ambiente
```bash
# Fork e clone o repositório
git clone https://github.com/seu-usuario/polis.git
cd polis

# Instalar dependências
cargo build

# Executar testes
cargo test

# Executar linting
cargo clippy
cargo fmt
```

#### Estrutura do Projeto
```
polis/
├── polis-core/          # Tipos e utilitários básicos
├── polis-runtime/       # Runtime de containers
├── polis-api/           # APIs REST e gRPC
├── polis-cli/           # Interface de linha de comando
├── polis-image/         # Gerenciamento de imagens
├── polis-network/       # Gerenciamento de rede
├── polis-security/      # Segurança e isolamento
├── polis-storage/       # Gerenciamento de armazenamento
├── polis-orchestrator/  # Orquestração
├── polis-monitor/       # Monitoramento
├── polis-auth/          # Autenticação
├── polis-sdk/           # SDK para desenvolvedores
├── polis-tests/         # Testes de integração
└── polis-benchmarks/    # Benchmarks de performance
```

#### Convenções de Código
- **Rust**: Siga o [Rust Style Guide](https://doc.rust-lang.org/1.0.0/style-guide/)
- **Commits**: Use [Conventional Commits](https://www.conventionalcommits.org/)
- **Documentação**: Use rustdoc para documentação de código
- **Testes**: Escreva testes para novas funcionalidades

### 2. Documentação

#### Tipos de Documentação
- **README**: Visão geral e início rápido
- **Tutoriais**: Guias passo a passo
- **Referência**: Documentação técnica completa
- **Exemplos**: Código de exemplo e casos de uso
- **FAQ**: Perguntas frequentes

#### Contribuindo com Documentação
- Use linguagem clara e concisa
- Inclua exemplos práticos
- Mantenha a documentação atualizada
- Use markdown para formatação
- Adicione screenshots quando útil

### 3. Testes

#### Tipos de Testes
- **Unit Tests**: Testes de funções individuais
- **Integration Tests**: Testes de integração entre componentes
- **End-to-End Tests**: Testes completos do sistema
- **Performance Tests**: Benchmarks e testes de performance

#### Executando Testes
```bash
# Todos os testes
cargo test

# Testes específicos
cargo test --package polis-runtime

# Testes com output
cargo test -- --nocapture

# Benchmarks
cargo bench
```

## 🏷️ Labels e Categorização

### Issues
- `bug`: Algo não está funcionando
- `enhancement`: Nova feature ou melhoria
- `documentation`: Melhorias na documentação
- `question`: Pergunta ou dúvida
- `help wanted`: Precisa de ajuda da comunidade
- `good first issue`: Bom para iniciantes
- `priority: high`: Alta prioridade
- `priority: medium`: Média prioridade
- `priority: low`: Baixa prioridade

### Pull Requests
- `ready for review`: Pronto para revisão
- `work in progress`: Em desenvolvimento
- `needs testing`: Precisa de testes
- `needs documentation`: Precisa de documentação
- `breaking change`: Mudança que quebra compatibilidade

## 🎯 Roadmap e Prioridades

### Como Propor Features
1. Abra uma issue com a label `enhancement`
2. Descreva o problema que resolve
3. Proponha uma solução
4. Discuta com a comunidade
5. Aguarde aprovação antes de implementar

### Processo de Decisão
- **RFCs**: Para mudanças significativas
- **Discussão**: GitHub Discussions para feedback
- **Votação**: Para decisões importantes
- **Consenso**: Busca por consenso da comunidade

## 🏆 Reconhecimento

### Contribuidores
- **Maintainers**: Responsáveis pela manutenção do projeto
- **Core Contributors**: Contribuidores regulares e ativos
- **Contributors**: Qualquer pessoa que contribuiu
- **Reviewers**: Pessoas que ajudam com revisões

### Como Ser Reconhecido
- Contribua regularmente
- Ajude outros contribuidores
- Participe de discussões
- Melhore a documentação
- Reporte bugs e issues

## 📞 Suporte e Contato

### Onde Buscar Ajuda
1. **Documentação**: Consulte primeiro a documentação
2. **FAQ**: Verifique perguntas frequentes
3. **GitHub Issues**: Para bugs e problemas
4. **GitHub Discussions**: Para perguntas gerais
5. **Discord**: Para chat em tempo real

### Contato Direto
- **Email**: contato@polis-project.org
- **Twitter**: @polis_project
- **LinkedIn**: Polis Project

## 🔄 Processo de Moderação

### Reportando Problemas
Se você encontrar comportamento inaceitável:
1. Reporte via email: moderacao@polis-project.org
2. Inclua evidências (screenshots, logs)
3. Descreva o incidente detalhadamente
4. Aguarde resposta da equipe de moderação

### Ações de Moderação
- **Aviso**: Para violações menores
- **Suspensão temporária**: Para violações repetidas
- **Banimento**: Para violações graves ou persistentes

## 📈 Métricas da Comunidade

### Objetivos
- **Crescimento**: Aumentar o número de contribuidores
- **Qualidade**: Melhorar a qualidade das contribuições
- **Diversidade**: Promover diversidade na comunidade
- **Engajamento**: Aumentar participação ativa

### Como Medimos
- Número de contribuidores ativos
- Frequência de commits e PRs
- Qualidade das discussões
- Resolução de issues
- Feedback da comunidade

## 🎉 Eventos e Atividades

### Eventos Regulares
- **Sprint Planning**: Planejamento de sprints
- **Code Review Sessions**: Sessões de revisão de código
- **Community Calls**: Chamadas da comunidade
- **Hackathons**: Eventos de desenvolvimento

### Como Participar
- Acompanhe o calendário de eventos
- Participe das discussões
- Contribua com ideias e feedback
- Ajude a organizar eventos

## 📝 Changelog e Releases

### Versionamento
- Seguimos [Semantic Versioning](https://semver.org/)
- **MAJOR**: Mudanças incompatíveis
- **MINOR**: Novas funcionalidades compatíveis
- **PATCH**: Correções de bugs compatíveis

### Release Notes
- Documentamos todas as mudanças
- Destacamos novas funcionalidades
- Listamos correções de bugs
- Incluímos breaking changes

## 🤝 Agradecimentos

### Aos Contribuidores
Obrigado a todos que contribuem para o Polis:
- Desenvolvedores que escrevem código
- Usuários que reportam bugs
- Documentadores que melhoram a documentação
- Moderadores que mantêm a comunidade saudável
- Tradutores que tornam o projeto acessível

### Aos Apoiadores
- Organizações que usam o Polis
- Patrocinadores que financiam o projeto
- Comunidade open source que inspira
- Mantenedores de projetos relacionados

---

**Última atualização**: Dezembro 2024
**Versão**: 1.0.0

*Estas diretrizes são um documento vivo e serão atualizadas conforme a comunidade cresce e evolui.*
