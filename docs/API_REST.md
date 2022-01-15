# API REST do Polis

## Visão Geral

A API REST do Polis fornece endpoints para gerenciar containers, imagens, redes e monitoramento. A API segue os padrões REST e retorna respostas em JSON.

**Base URL:** `http://localhost:8080/api/v1`

## Autenticação

Atualmente, a API não requer autenticação. Em versões futuras, será implementado sistema de autenticação baseado em tokens.

## Headers Padrão

```
Content-Type: application/json
Accept: application/json
```

## Códigos de Status HTTP

- `200 OK` - Operação realizada com sucesso
- `201 Created` - Recurso criado com sucesso
- `400 Bad Request` - Requisição inválida
- `404 Not Found` - Recurso não encontrado
- `409 Conflict` - Conflito (ex: container já existe)
- `500 Internal Server Error` - Erro interno do servidor

## Endpoints

### Containers

#### Listar Containers
```http
GET /containers
```

**Resposta:**
```json
{
  "containers": [
    {
      "id": "container-123",
      "name": "meu-container",
      "image": "alpine:latest",
      "status": "Running",
      "created_at": "2024-01-15T10:30:00Z",
      "ports": [
        {
          "host_port": 8080,
          "container_port": 80,
          "protocol": "Tcp"
        }
      ],
      "environment": {
        "NODE_ENV": "production"
      }
    }
  ]
}
```

#### Criar Container
```http
POST /containers
```

**Body:**
```json
{
  "name": "meu-container",
  "image": "alpine:latest",
  "command": ["echo", "Hello World"],
  "ports": [
    {
      "host_port": 8080,
      "container_port": 80,
      "protocol": "Tcp"
    }
  ],
  "environment": {
    "NODE_ENV": "production"
  },
  "resource_limits": {
    "memory_limit": 1073741824,
    "cpu_quota": 0.5
  }
}
```

**Resposta:**
```json
{
  "id": "container-123",
  "name": "meu-container",
  "image": "alpine:latest",
  "status": "Created",
  "created_at": "2024-01-15T10:30:00Z"
}
```

#### Obter Container
```http
GET /containers/{id}
```

**Resposta:**
```json
{
  "id": "container-123",
  "name": "meu-container",
  "image": "alpine:latest",
  "status": "Running",
  "created_at": "2024-01-15T10:30:00Z",
  "ports": [
    {
      "host_port": 8080,
      "container_port": 80,
      "protocol": "Tcp"
    }
  ],
  "environment": {
    "NODE_ENV": "production"
  },
  "resource_limits": {
    "memory_limit": 1073741824,
    "cpu_quota": 0.5
  }
}
```

#### Iniciar Container
```http
POST /containers/{id}/start
```

**Resposta:**
```json
{
  "message": "Container iniciado com sucesso",
  "container_id": "container-123"
}
```

#### Parar Container
```http
POST /containers/{id}/stop
```

**Resposta:**
```json
{
  "message": "Container parado com sucesso",
  "container_id": "container-123"
}
```

#### Remover Container
```http
DELETE /containers/{id}
```

**Resposta:**
```json
{
  "message": "Container removido com sucesso",
  "container_id": "container-123"
}
```

#### Logs do Container
```http
GET /containers/{id}/logs
```

**Query Parameters:**
- `follow` (boolean): Seguir logs em tempo real
- `tail` (number): Número de linhas para retornar

**Resposta:**
```json
{
  "logs": [
    {
      "timestamp": "2024-01-15T10:30:00Z",
      "level": "INFO",
      "message": "Container iniciado"
    }
  ]
}
```

### Imagens

#### Listar Imagens
```http
GET /images
```

**Resposta:**
```json
{
  "images": [
    {
      "id": "image-123",
      "name": "alpine",
      "tag": "latest",
      "digest": "sha256:abc123...",
      "size": 5368709120,
      "created_at": "2024-01-15T10:30:00Z"
    }
  ]
}
```

#### Baixar Imagem
```http
POST /images/pull
```

**Body:**
```json
{
  "name": "alpine:latest",
  "registry": "docker.io"
}
```

**Resposta:**
```json
{
  "message": "Imagem baixada com sucesso",
  "image_id": "image-123",
  "name": "alpine:latest"
}
```

#### Remover Imagem
```http
DELETE /images/{id}
```

**Resposta:**
```json
{
  "message": "Imagem removida com sucesso",
  "image_id": "image-123"
}
```

### Redes

#### Listar Redes
```http
GET /networks
```

**Resposta:**
```json
{
  "networks": [
    {
      "name": "polis0",
      "ip": "172.17.0.1",
      "subnet": "172.17.0.0/16",
      "mtu": 1500,
      "interfaces": ["veth-123", "veth-456"],
      "enabled": true
    }
  ]
}
```

#### Criar Rede
```http
POST /networks
```

**Body:**
```json
{
  "name": "minha-rede",
  "ip": "192.168.1.1",
  "subnet": "192.168.1.0/24",
  "mtu": 1500
}
```

**Resposta:**
```json
{
  "message": "Rede criada com sucesso",
  "network_name": "minha-rede"
}
```

#### Conectar Container à Rede
```http
POST /networks/{name}/connect
```

**Body:**
```json
{
  "container_id": "container-123"
}
```

**Resposta:**
```json
{
  "message": "Container conectado à rede com sucesso",
  "container_id": "container-123",
  "network_name": "minha-rede"
}
```

### Monitoramento

#### Métricas do Sistema
```http
GET /metrics/system
```

**Resposta:**
```json
{
  "cpu_usage": 45.2,
  "memory_usage": 67.8,
  "disk_usage": 23.1,
  "network_rx": 1024000,
  "network_tx": 2048000,
  "containers_running": 5,
  "containers_total": 10
}
```

#### Métricas de Container
```http
GET /metrics/containers/{id}
```

**Resposta:**
```json
{
  "container_id": "container-123",
  "cpu_usage": 12.5,
  "memory_usage": 256000000,
  "memory_limit": 1073741824,
  "network_rx": 512000,
  "network_tx": 1024000,
  "disk_usage": 104857600
}
```

#### Health Check
```http
GET /health
```

**Resposta:**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "services": {
    "runtime": "healthy",
    "network": "healthy",
    "storage": "healthy",
    "monitoring": "healthy"
  }
}
```

### Sistema

#### Informações do Sistema
```http
GET /system/info
```

**Resposta:**
```json
{
  "version": "0.1.0",
  "os": "Linux",
  "arch": "x86_64",
  "kernel": "5.4.0-74-generic",
  "containers_running": 5,
  "containers_total": 10,
  "images_total": 15,
  "networks_total": 3
}
```

#### Estatísticas do Sistema
```http
GET /system/stats
```

**Resposta:**
```json
{
  "containers": {
    "running": 5,
    "stopped": 3,
    "created": 2,
    "total": 10
  },
  "images": {
    "total": 15,
    "size_total": 53687091200
  },
  "networks": {
    "total": 3,
    "active": 2
  },
  "storage": {
    "used": 10737418240,
    "available": 42949672960
  }
}
```

## Tratamento de Erros

### Formato de Erro
```json
{
  "error": {
    "code": "CONTAINER_NOT_FOUND",
    "message": "Container não encontrado",
    "details": "Container com ID 'container-123' não existe"
  }
}
```

### Códigos de Erro Comuns

- `CONTAINER_NOT_FOUND` - Container não encontrado
- `IMAGE_NOT_FOUND` - Imagem não encontrada
- `NETWORK_NOT_FOUND` - Rede não encontrada
- `INVALID_REQUEST` - Requisição inválida
- `RESOURCE_LIMIT_EXCEEDED` - Limite de recursos excedido
- `CONTAINER_ALREADY_RUNNING` - Container já está em execução
- `CONTAINER_NOT_RUNNING` - Container não está em execução
- `PORT_ALREADY_IN_USE` - Porta já está em uso
- `INSUFFICIENT_RESOURCES` - Recursos insuficientes

## Exemplos de Uso

### Criar e Executar um Container

```bash
# 1. Criar container
curl -X POST http://localhost:8080/api/v1/containers \
  -H "Content-Type: application/json" \
  -d '{
    "name": "nginx-server",
    "image": "nginx:alpine",
    "ports": [
      {
        "host_port": 8080,
        "container_port": 80,
        "protocol": "Tcp"
      }
    ]
  }'

# 2. Iniciar container
curl -X POST http://localhost:8080/api/v1/containers/container-123/start

# 3. Verificar status
curl http://localhost:8080/api/v1/containers/container-123

# 4. Ver logs
curl http://localhost:8080/api/v1/containers/container-123/logs
```

### Gerenciar Imagens

```bash
# 1. Baixar imagem
curl -X POST http://localhost:8080/api/v1/images/pull \
  -H "Content-Type: application/json" \
  -d '{
    "name": "alpine:latest",
    "registry": "docker.io"
  }'

# 2. Listar imagens
curl http://localhost:8080/api/v1/images

# 3. Remover imagem
curl -X DELETE http://localhost:8080/api/v1/images/image-123
```

### Monitoramento

```bash
# 1. Verificar saúde do sistema
curl http://localhost:8080/api/v1/health

# 2. Obter métricas do sistema
curl http://localhost:8080/api/v1/metrics/system

# 3. Obter métricas de um container
curl http://localhost:8080/api/v1/metrics/containers/container-123
```

## Rate Limiting

A API implementa rate limiting para prevenir abuso:

- **Containers:** 100 requisições por minuto
- **Imagens:** 50 requisições por minuto
- **Monitoramento:** 200 requisições por minuto
- **Sistema:** 30 requisições por minuto

Quando o limite é excedido, a API retorna:
```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit excedido",
    "retry_after": 60
  }
}
```

## Versionamento

A API usa versionamento semântico na URL:
- `v1` - Versão atual (estável)
- `v2` - Próxima versão (em desenvolvimento)

## Changelog

### v1.0.0 (2024-01-15)
- Endpoints básicos para containers
- Endpoints para imagens
- Endpoints para redes
- Sistema de monitoramento
- Health checks
- Rate limiting básico

