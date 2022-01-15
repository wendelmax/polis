# Tutorial do Polis - Guia Prático

## Introdução

Este tutorial guia você através dos conceitos básicos do Polis, desde a instalação até a execução de aplicações complexas em containers.

## Pré-requisitos

- Linux (Ubuntu 20.04+ recomendado)
- Rust 1.70+
- Git
- Privilégios de root (para algumas operações)

## Instalação

### 1. Instalar Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update
```

### 2. Clonar e Compilar Polis

```bash
git clone https://github.com/polis/polis.git
cd polis
cargo build --release
```

### 3. Instalar Binários

```bash
sudo cp target/release/polis /usr/local/bin/
sudo cp target/release/polis-api /usr/local/bin/
sudo cp target/release/polis-grpc /usr/local/bin/
```

### 4. Verificar Instalação

```bash
polis --version
polis system info
```

## Primeiros Passos

### 1. Inicializar Polis

```bash
# Inicializar configuração padrão
polis init

# Verificar status
polis system info
```

### 2. Baixar Primeira Imagem

```bash
# Baixar imagem Alpine
polis pull alpine:latest

# Listar imagens
polis images
```

### 3. Criar Primeiro Container

```bash
# Criar container simples
polis create --name hello-world --image alpine:latest --command "echo Hello Polis!"

# Iniciar container
polis start hello-world

# Verificar status
polis list
```

## Conceitos Básicos

### Containers

Um container no Polis é uma unidade de execução isolada que contém uma aplicação e suas dependências.

#### Estados do Container
- **Created**: Container criado mas não iniciado
- **Running**: Container em execução
- **Stopped**: Container parado
- **Paused**: Container pausado
- **Removed**: Container removido

#### Comandos Básicos

```bash
# Criar container
polis create --name meu-app --image nginx:alpine

# Iniciar container
polis start meu-app

# Parar container
polis stop meu-app

# Remover container
polis remove meu-app

# Listar containers
polis list

# Ver logs
polis logs meu-app

# Inspecionar container
polis inspect meu-app
```

### Imagens

Imagens são templates read-only usados para criar containers.

#### Comandos de Imagem

```bash
# Baixar imagem
polis pull nginx:alpine

# Listar imagens
polis images

# Inspecionar imagem
polis inspect nginx:alpine

# Remover imagem
polis rmi nginx:alpine
```

### Redes

Polis permite criar redes personalizadas para conectar containers.

#### Comandos de Rede

```bash
# Criar rede
polis network create --name minha-rede --subnet 192.168.1.0/24

# Listar redes
polis network list

# Conectar container à rede
polis network connect minha-rede meu-app

# Desconectar container da rede
polis network disconnect minha-rede meu-app

# Remover rede
polis network remove minha-rede
```

## Exemplos Práticos

### Exemplo 1: Servidor Web Simples

```bash
# 1. Baixar imagem Nginx
polis pull nginx:alpine

# 2. Criar container
polis create \
  --name web-server \
  --image nginx:alpine \
  --port 8080:80 \
  --env NGINX_HOST=localhost \
  --env NGINX_PORT=80

# 3. Iniciar container
polis start web-server

# 4. Verificar se está funcionando
curl http://localhost:8080

# 5. Ver logs
polis logs web-server

# 6. Parar e remover
polis stop web-server
polis remove web-server
```

### Exemplo 2: Aplicação com Banco de Dados

```bash
# 1. Criar rede para a aplicação
polis network create --name app-network --subnet 172.20.0.0/16

# 2. Criar container do banco de dados
polis create \
  --name database \
  --image postgres:13 \
  --network app-network \
  --env POSTGRES_DB=myapp \
  --env POSTGRES_USER=user \
  --env POSTGRES_PASSWORD=password \
  --volume postgres_data:/var/lib/postgresql/data

# 3. Criar container da aplicação
polis create \
  --name web-app \
  --image node:16-alpine \
  --network app-network \
  --port 3000:3000 \
  --env DATABASE_URL=postgres://user:password@database:5432/myapp \
  --volume ./app:/app \
  --command "npm start"

# 4. Iniciar containers
polis start database
polis start web-app

# 5. Verificar conectividade
polis network test app-network

# 6. Ver logs de ambos
polis logs database
polis logs web-app
```

### Exemplo 3: Aplicação com Múltiplos Serviços

```bash
# 1. Criar rede
polis network create --name microservices --subnet 172.30.0.0/16

# 2. Criar serviço de API
polis create \
  --name api-service \
  --image python:3.9-alpine \
  --network microservices \
  --port 8000:8000 \
  --env FLASK_ENV=production \
  --command "python app.py"

# 3. Criar serviço de cache
polis create \
  --name cache-service \
  --image redis:alpine \
  --network microservices \
  --port 6379:6379

# 4. Criar serviço de frontend
polis create \
  --name frontend \
  --image nginx:alpine \
  --network microservices \
  --port 80:80 \
  --volume ./dist:/usr/share/nginx/html

# 5. Iniciar todos os serviços
polis start cache-service
polis start api-service
polis start frontend

# 6. Verificar status
polis list
```

## Configuração Avançada

### Arquivo de Configuração

Crie um arquivo `polis.yaml`:

```yaml
runtime:
  max_containers: 100
  container_timeout: 30
  log_level: "info"
  root_dir: "/var/lib/polis"

storage:
  root_dir: "/var/lib/polis/storage"
  max_size: 107374182400  # 100GB

network:
  bridge_name: "polis0"
  subnet: "172.17.0.0/16"
  gateway: "172.17.0.1"

security:
  drop_capabilities: ["SYS_ADMIN"]
  read_only_rootfs: false
  enable_seccomp: true
  enable_capabilities: true
  enable_namespaces: true
  enable_cgroups: true

api:
  rest_port: 8080
  grpc_port: 9090
  host: "0.0.0.0"
```

### Usar Configuração Personalizada

```bash
polis --config polis.yaml create --name meu-app --image alpine:latest
```

## Monitoramento

### Métricas do Sistema

```bash
# Ver métricas gerais
polis metrics system

# Ver métricas de um container
polis metrics container meu-app

# Monitorar em tempo real
polis metrics watch
```

### Health Checks

```bash
# Verificar saúde do sistema
polis health

# Verificar saúde de um container
polis health container meu-app

# Configurar health check
polis health set meu-app --interval 30s --timeout 10s --retries 3
```

### Logs

```bash
# Ver logs de um container
polis logs meu-app

# Seguir logs em tempo real
polis logs --follow meu-app

# Ver logs com filtro
polis logs meu-app --level error

# Exportar logs
polis logs --export meu-app > logs.txt
```

## APIs

### API REST

```bash
# Iniciar servidor REST
polis-api --port 8080

# Testar API
curl http://localhost:8080/api/v1/health
curl http://localhost:8080/api/v1/containers
```

### API gRPC

```bash
# Iniciar servidor gRPC
polis-grpc --port 9090

# Testar com grpcurl
grpcurl -plaintext localhost:9090 list
```

## Troubleshooting

### Problemas Comuns

#### 1. Container não inicia

```bash
# Verificar logs
polis logs container-name

# Verificar configuração
polis inspect container-name

# Verificar recursos
polis system stats
```

#### 2. Problemas de rede

```bash
# Verificar redes
polis network list

# Testar conectividade
polis network test network-name

# Verificar port forwarding
polis port list
```

#### 3. Problemas de volume

```bash
# Verificar volumes
polis volume list

# Verificar montagens
polis volume inspect volume-name
```

### Debugging

```bash
# Modo verbose
polis --verbose create --name test alpine:latest

# Logs de debug
polis --log-level debug start test

# Verificar status do sistema
polis system info
polis system stats
```

## Boas Práticas

### 1. Nomenclatura

```bash
# Use nomes descritivos
polis create --name web-server-prod --image nginx:alpine
polis create --name db-staging --image postgres:13

# Use tags específicas
polis pull nginx:1.21-alpine
polis pull postgres:13.4-alpine
```

### 2. Recursos

```bash
# Defina limites de recursos
polis create \
  --name resource-limited \
  --image alpine:latest \
  --memory-limit 512m \
  --cpu-quota 0.5 \
  --pids-limit 100
```

### 3. Segurança

```bash
# Use imagens oficiais
polis pull nginx:alpine
polis pull postgres:13-alpine

# Configure segurança
polis create \
  --name secure-app \
  --image alpine:latest \
  --read-only \
  --no-new-privileges \
  --user 1000:1000
```

### 4. Volumes

```bash
# Use volumes nomeados para dados persistentes
polis volume create app-data
polis create \
  --name app \
  --image alpine:latest \
  --volume app-data:/data
```

### 5. Redes

```bash
# Crie redes específicas para cada aplicação
polis network create --name frontend-net --subnet 172.20.0.0/16
polis network create --name backend-net --subnet 172.21.0.0/16
```

## Próximos Passos

### 1. Orquestração

```bash
# Usar Polis com Kubernetes
kubectl apply -f polis-deployment.yaml

# Usar Polis com Docker Compose
docker-compose up -d
```

### 2. CI/CD

```bash
# Integrar com GitHub Actions
name: Deploy with Polis
on: [push]
jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Deploy with Polis
        run: |
          polis create --name app --image ${{ github.sha }}
          polis start app
```

### 3. Monitoramento Avançado

```bash
# Integrar com Prometheus
polis metrics export --format prometheus --port 9090

# Integrar com Grafana
polis metrics dashboard --port 3000
```

## Recursos Adicionais

### Documentação
- [Referência da API](API_REST.md)
- [Guia de Migração do Docker](MIGRATION_DOCKER.md)
- [Exemplos Avançados](examples/)

### Comunidade
- [GitHub](https://github.com/polis/polis)
- [Discord](https://discord.gg/polis)
- [Stack Overflow](https://stackoverflow.com/tags/polis)

### Suporte
- [FAQ](FAQ.md)
- [Issues](https://github.com/polis/polis/issues)
- [Documentação Oficial](https://docs.polis.dev)

## Conclusão

Este tutorial cobriu os conceitos básicos do Polis. Para aprender mais, explore a documentação avançada e participe da comunidade.

Lembre-se de sempre testar em ambiente de desenvolvimento antes de usar em produção!

