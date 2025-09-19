# Tutorial Completo do Polis

Este tutorial guia você através dos conceitos básicos e avançados do Polis Container Runtime, desde a instalação até o uso em produção.

## 📚 Índice

1. [Introdução](#introdução)
2. [Instalação](#instalação)
3. [Primeiros Passos](#primeiros-passos)
4. [Gerenciamento de Containers](#gerenciamento-de-containers)
5. [Gerenciamento de Imagens](#gerenciamento-de-imagens)
6. [Rede e Conectividade](#rede-e-conectividade)
7. [Armazenamento e Volumes](#armazenamento-e-volumes)
8. [Orquestração](#orquestração)
9. [Monitoramento](#monitoramento)
10. [Segurança](#segurança)
11. [APIs](#apis)
12. [Exemplos Práticos](#exemplos-práticos)

## 🚀 Introdução

### O que é o Polis?

O Polis é um container runtime moderno e eficiente escrito em Rust, projetado para ser uma alternativa profissional ao Docker com foco em performance, segurança e simplicidade.

### Características Principais

- **Performance**: Inicialização rápida e baixo overhead
- **Segurança**: Isolamento robusto com namespaces e cgroups
- **Simplicidade**: Interface CLI intuitiva
- **Compatibilidade**: Suporte completo ao padrão OCI
- **Modularidade**: Arquitetura baseada em componentes

### Conceitos Básicos

- **Container**: Ambiente isolado que executa aplicações
- **Imagem**: Template para criar containers
- **Registry**: Repositório de imagens
- **Volume**: Armazenamento persistente
- **Rede**: Conectividade entre containers

## 📦 Instalação

### Instalação Rápida

```bash
# Windows
.\installers\windows\install.ps1

# Linux/macOS
./installers/linux/install.sh
```

### Instalação Manual

```bash
# Clone o repositório
git clone https://github.com/polis/polis.git
cd polis

# Compile
cargo build --release

# Instale
sudo cp target/release/polis* /usr/local/bin/
```

### Verificação

```bash
# Verificar instalação
polis --version
polis-api --version
polis-grpc --version
```

## 🎯 Primeiros Passos

### 1. Inicializar o Polis

```bash
# Inicializar configuração
polis init

# Verificar configuração
polis config show
```

### 2. Primeiro Container

```bash
# Baixar uma imagem
polis image pull alpine:latest

# Criar um container
polis container create --name hello --image alpine:latest --command "echo 'Olá, Polis!'"

# Executar o container
polis container start hello

# Ver logs
polis container logs hello

# Limpar
polis container stop hello
polis container remove hello
```

### 3. Explorar o Sistema

```bash
# Ver informações do sistema
polis system info

# Ver status dos serviços
polis system status

# Ver configuração
polis config show
```

## 🐳 Gerenciamento de Containers

### Criar e Executar Containers

```bash
# Criar container simples
polis container create --name nginx --image nginx:alpine

# Criar com comando personalizado
polis container create --name app --image alpine:latest --command "sleep 3600"

# Criar com variáveis de ambiente
polis container create --name web --image nginx:alpine \
  --env NGINX_HOST=localhost \
  --env NGINX_PORT=8080

# Criar com mapeamento de porta
polis container create --name web --image nginx:alpine --port 8080:80

# Executar container
polis container start nginx
```

### Gerenciar Containers

```bash
# Listar containers
polis container list

# Listar todos (incluindo parados)
polis container list --all

# Ver detalhes de um container
polis container inspect nginx

# Ver logs
polis container logs nginx

# Seguir logs em tempo real
polis container logs --follow nginx

# Executar comando em container rodando
polis container exec nginx sh

# Parar container
polis container stop nginx

# Reiniciar container
polis container restart nginx

# Remover container
polis container remove nginx

# Remover container parado
polis container remove --force nginx
```

### Configuração Avançada

```bash
# Criar com limites de recursos
polis container create --name app --image alpine:latest \
  --memory-limit 512m \
  --cpu-limit 0.5

# Criar com volume montado
polis container create --name app --image alpine:latest \
  --volume /host/path:/container/path

# Criar com rede personalizada
polis container create --name app --image alpine:latest \
  --network mynetwork

# Criar com perfil de segurança
polis container create --name app --image alpine:latest \
  --security-profile apparmor:docker-default
```

## 🖼️ Gerenciamento de Imagens

### Baixar Imagens

```bash
# Baixar imagem
polis image pull alpine:latest

# Baixar de registry específico
polis image pull registry.example.com/app:1.0

# Baixar com autenticação
polis image pull --username user --password pass private-registry.com/app:1.0
```

### Gerenciar Imagens

```bash
# Listar imagens
polis image list

# Ver detalhes de uma imagem
polis image inspect alpine:latest

# Ver histórico de uma imagem
polis image history alpine:latest

# Remover imagem
polis image remove alpine:latest

# Remover imagens não utilizadas
polis image cleanup

# Remover imagens antigas
polis image cleanup --older-than 7d
```

### Build de Imagens

```bash
# Build a partir de Dockerfile
polis build -t myapp:1.0 .

# Build com contexto específico
polis build -t myapp:1.0 /path/to/context

# Build com argumentos
polis build -t myapp:1.0 --build-arg VERSION=1.0 .

# Build com cache
polis build -t myapp:1.0 --cache-from myapp:latest .
```

### Configurar Registries

```bash
# Adicionar registry
polis registry add docker.io --username user --password token

# Listar registries
polis registry list

# Remover registry
polis registry remove docker.io

# Testar conectividade
polis registry ping docker.io
```

## 🌐 Rede e Conectividade

### Gerenciar Redes

```bash
# Criar rede
polis network create --name mynet --subnet 172.20.0.0/16

# Listar redes
polis network list

# Ver detalhes da rede
polis network inspect mynet

# Conectar container à rede
polis network connect mynet container1

# Desconectar container da rede
polis network disconnect mynet container1

# Remover rede
polis network remove mynet
```

### Port Forwarding

```bash
# Mapear porta
polis container create --name web --image nginx:alpine --port 8080:80

# Adicionar port forwarding
polis port-forward add --container web --host-port 8080 --container-port 80

# Listar port forwards
polis port-forward list

# Remover port forward
polis port-forward remove web:8080
```

### DNS e Resolução

```bash
# Criar registro DNS
polis dns add --name app.local --ip 172.20.0.10

# Listar registros DNS
polis dns list

# Resolver nome
polis dns resolve app.local
```

## 💾 Armazenamento e Volumes

### Gerenciar Volumes

```bash
# Criar volume
polis volume create --name mydata

# Listar volumes
polis volume list

# Ver detalhes do volume
polis volume inspect mydata

# Montar volume em container
polis container create --name app --image alpine:latest \
  --volume mydata:/data

# Remover volume
polis volume remove mydata
```

### Configuração de Volumes

```bash
# Criar volume com opções
polis volume create --name mydata \
  --driver local \
  --opt type=tmpfs \
  --opt device=tmpfs

# Criar volume com labels
polis volume create --name mydata \
  --label env=production \
  --label app=web
```

## 🎛️ Orquestração

### Deploy de Aplicações

```bash
# Deploy simples
polis deploy create --name webapp --image nginx:alpine --replicas 3

# Deploy com configuração
polis deploy create --name webapp --image nginx:alpine \
  --replicas 3 \
  --port 8080:80 \
  --env NGINX_HOST=localhost

# Listar deployments
polis deploy list

# Ver status do deployment
polis deploy status webapp

# Escalar deployment
polis deploy scale webapp 5

# Remover deployment
polis deploy remove webapp
```

### Service Discovery

```bash
# Registrar serviço
polis service register --name web --port 8080 --target nginx

# Listar serviços
polis service list

# Ver detalhes do serviço
polis service inspect web

# Remover serviço
polis service remove web
```

### Load Balancing

```bash
# Configurar load balancer
polis load-balancer create --name lb --port 80 --targets web:8080

# Listar load balancers
polis load-balancer list

# Adicionar target
polis load-balancer add-target lb web2:8080

# Remover target
polis load-balancer remove-target lb web2:8080
```

## 📊 Monitoramento

### Métricas do Sistema

```bash
# Ver métricas do sistema
polis stats system

# Ver métricas de um container
polis stats container nginx

# Ver métricas em tempo real
polis stats container --follow nginx
```

### Health Checks

```bash
# Verificar saúde do sistema
polis health

# Verificar saúde de um container
polis health container nginx

# Configurar health check
polis container create --name app --image nginx:alpine \
  --health-check "curl -f http://localhost/health"
```

### Logs

```bash
# Ver logs de um container
polis container logs nginx

# Seguir logs em tempo real
polis container logs --follow nginx

# Ver logs com filtro
polis container logs --since 1h nginx
```

## 🔒 Segurança

### Configuração de Segurança

```bash
# Criar container com perfil de segurança
polis container create --name secure --image alpine:latest \
  --security-profile apparmor:docker-default

# Criar com capabilities limitadas
polis container create --name secure --image alpine:latest \
  --cap-drop ALL --cap-add NET_BIND_SERVICE

# Criar com usuário não-root
polis container create --name secure --image alpine:latest \
  --user 1000:1000
```

### Autenticação

```bash
# Fazer login em registry
polis login registry.example.com --username user --password pass

# Fazer logout
polis logout registry.example.com

# Verificar autenticação
polis auth status
```

## 🔌 APIs

### API REST

```bash
# Iniciar servidor REST
polis-api --port 8080

# Testar API
curl http://localhost:8080/api/v1/health
curl http://localhost:8080/api/v1/containers
curl http://localhost:8080/api/v1/images
```

### API gRPC

```bash
# Iniciar servidor gRPC
polis-grpc --port 9090

# Testar com grpcurl
grpcurl -plaintext localhost:9090 list
grpcurl -plaintext localhost:9090 polis.ContainerService/ListContainers
```

## 🏗️ Exemplos Práticos

### Aplicação Web com Banco de Dados

```bash
# 1. Criar rede
polis network create --name app-net --subnet 172.20.0.0/16

# 2. Criar banco de dados
polis container create --name db --image postgres:13 \
  --network app-net \
  --env POSTGRES_DB=myapp \
  --env POSTGRES_PASSWORD=secret \
  --volume db-data:/var/lib/postgresql/data

# 3. Criar aplicação
polis container create --name app --image node:16 \
  --network app-net \
  --port 3000:3000 \
  --env DATABASE_URL=postgres://db:5432/myapp \
  --volume app-code:/app

# 4. Iniciar serviços
polis container start db
polis container start app

# 5. Verificar status
polis container list
```

### Deploy com Load Balancer

```bash
# 1. Deploy da aplicação
polis deploy create --name webapp --image nginx:alpine --replicas 3

# 2. Configurar load balancer
polis load-balancer create --name lb --port 80 --targets webapp:80

# 3. Verificar status
polis deploy status webapp
polis load-balancer status lb
```

### Monitoramento Completo

```bash
# 1. Ver métricas do sistema
polis stats system

# 2. Ver métricas dos containers
polis stats container webapp

# 3. Verificar saúde
polis health

# 4. Ver logs
polis container logs --follow webapp
```

## 🎯 Próximos Passos

### Recursos Avançados

1. **Configuração de Produção**: Configure para ambiente de produção
2. **Monitoramento**: Implemente monitoramento completo
3. **Segurança**: Aplique práticas de segurança avançadas
4. **CI/CD**: Integre com pipelines de CI/CD
5. **Orquestração**: Use orquestração avançada

### Recursos de Aprendizado

- **Documentação**: [docs.polis.dev](https://docs.polis.dev)
- **Exemplos**: [examples.polis.dev](https://examples.polis.dev)
- **Comunidade**: [discord.gg/polis](https://discord.gg/polis)
- **GitHub**: [github.com/polis/polis](https://github.com/polis/polis)

## 📞 Suporte

### Canais de Suporte
- **GitHub Issues**: [github.com/polis/polis/issues](https://github.com/polis/polis/issues)
- **Discord**: [discord.gg/polis](https://discord.gg/polis)
- **Stack Overflow**: [stackoverflow.com/tags/polis](https://stackoverflow.com/tags/polis)
- **Email**: support@polis.dev

---

**Última atualização**: Janeiro 2025  
**Versão**: 1.0.0  
**Status**: Ativa e mantida

**Polis** - Container Runtime moderno, seguro e eficiente. Feito com ❤ no Brasil.
