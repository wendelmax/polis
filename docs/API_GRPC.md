# API gRPC do Polis

## Visão Geral

A API gRPC do Polis fornece serviços para gerenciar containers, imagens e sistema com alta performance e streaming bidirecional. A API usa Protocol Buffers para serialização eficiente.

**Endpoint:** `localhost:9090`

## Serviços Disponíveis

### 1. ContainerService

Gerencia o ciclo de vida de containers.

#### Métodos

##### CreateContainer
```protobuf
rpc CreateContainer(CreateContainerRequest) returns (CreateContainerResponse);
```

**Request:**
```protobuf
message CreateContainerRequest {
  string name = 1;
  string image = 2;
  repeated string command = 3;
  repeated PortMapping ports = 4;
  map<string, string> environment = 5;
  ResourceLimits resource_limits = 6;
}

message PortMapping {
  int32 host_port = 1;
  int32 container_port = 2;
  Protocol protocol = 3;
  string host_ip = 4;
}

enum Protocol {
  TCP = 0;
  UDP = 1;
}

message ResourceLimits {
  optional int64 memory_limit = 1;
  optional int64 memory_swap = 2;
  optional double cpu_quota = 3;
  optional int64 cpu_period = 4;
  optional int64 disk_quota = 5;
  optional int64 pids_limit = 6;
}
```

**Response:**
```protobuf
message CreateContainerResponse {
  string container_id = 1;
  string name = 2;
  string image = 3;
  ContainerStatus status = 4;
  string created_at = 5;
}

enum ContainerStatus {
  CREATED = 0;
  RUNNING = 1;
  STOPPED = 2;
  PAUSED = 3;
  REMOVED = 4;
}
```

##### StartContainer
```protobuf
rpc StartContainer(StartContainerRequest) returns (StartContainerResponse);
```

**Request:**
```protobuf
message StartContainerRequest {
  string container_id = 1;
}
```

**Response:**
```protobuf
message StartContainerResponse {
  bool success = 1;
  string message = 2;
  string container_id = 3;
}
```

##### StopContainer
```protobuf
rpc StopContainer(StopContainerRequest) returns (StopContainerResponse);
```

**Request:**
```protobuf
message StopContainerRequest {
  string container_id = 1;
  int32 timeout = 2; // segundos
}
```

**Response:**
```protobuf
message StopContainerResponse {
  bool success = 1;
  string message = 2;
  string container_id = 3;
}
```

##### RemoveContainer
```protobuf
rpc RemoveContainer(RemoveContainerRequest) returns (RemoveContainerResponse);
```

**Request:**
```protobuf
message RemoveContainerRequest {
  string container_id = 1;
  bool force = 2;
}
```

**Response:**
```protobuf
message RemoveContainerResponse {
  bool success = 1;
  string message = 2;
  string container_id = 3;
}
```

##### ListContainers
```protobuf
rpc ListContainers(ListContainersRequest) returns (ListContainersResponse);
```

**Request:**
```protobuf
message ListContainersRequest {
  optional ContainerStatus status = 1;
  optional string image = 2;
}
```

**Response:**
```protobuf
message ListContainersResponse {
  repeated ContainerInfo containers = 1;
}

message ContainerInfo {
  string id = 1;
  string name = 2;
  string image = 3;
  ContainerStatus status = 4;
  string created_at = 5;
  repeated PortMapping ports = 6;
  map<string, string> environment = 7;
  ResourceLimits resource_limits = 8;
}
```

##### GetContainer
```protobuf
rpc GetContainer(GetContainerRequest) returns (GetContainerResponse);
```

**Request:**
```protobuf
message GetContainerRequest {
  string container_id = 1;
}
```

**Response:**
```protobuf
message GetContainerResponse {
  ContainerInfo container = 1;
}
```

##### StreamLogs
```protobuf
rpc StreamLogs(StreamLogsRequest) returns (stream LogEntry);
```

**Request:**
```protobuf
message StreamLogsRequest {
  string container_id = 1;
  bool follow = 2;
  int32 tail = 3;
  optional string since = 4; // timestamp
}
```

**Response:**
```protobuf
message LogEntry {
  string timestamp = 1;
  string level = 2;
  string message = 3;
  string container_id = 4;
}
```

### 2. ImageService

Gerencia imagens OCI.

#### Métodos

##### PullImage
```protobuf
rpc PullImage(PullImageRequest) returns (stream PullImageResponse);
```

**Request:**
```protobuf
message PullImageRequest {
  string name = 1;
  string tag = 2;
  string registry = 3;
  bool insecure = 4;
}
```

**Response:**
```protobuf
message PullImageResponse {
  string status = 1;
  string progress = 2;
  string image_id = 3;
  int64 size = 4;
}
```

##### ListImages
```protobuf
rpc ListImages(ListImagesRequest) returns (ListImagesResponse);
```

**Request:**
```protobuf
message ListImagesRequest {
  optional string name = 1;
  optional string tag = 2;
}
```

**Response:**
```protobuf
message ListImagesResponse {
  repeated ImageInfo images = 1;
}

message ImageInfo {
  string id = 1;
  string name = 2;
  string tag = 3;
  string digest = 4;
  int64 size = 5;
  string created_at = 6;
  repeated string labels = 7;
}
```

##### RemoveImage
```protobuf
rpc RemoveImage(RemoveImageRequest) returns (RemoveImageResponse);
```

**Request:**
```protobuf
message RemoveImageRequest {
  string image_id = 1;
  bool force = 2;
}
```

**Response:**
```protobuf
message RemoveImageResponse {
  bool success = 1;
  string message = 2;
  string image_id = 3;
}
```

##### InspectImage
```protobuf
rpc InspectImage(InspectImageRequest) returns (InspectImageResponse);
```

**Request:**
```protobuf
message InspectImageRequest {
  string image_id = 1;
}
```

**Response:**
```protobuf
message InspectImageResponse {
  ImageInfo image = 1;
  ImageConfig config = 2;
  repeated string layers = 3;
}

message ImageConfig {
  string architecture = 1;
  string os = 2;
  repeated string cmd = 3;
  repeated string entrypoint = 4;
  map<string, string> env = 5;
  repeated string volumes = 6;
  string working_dir = 7;
  map<string, string> labels = 8;
}
```

### 3. SystemService

Fornece informações do sistema e monitoramento.

#### Métodos

##### GetSystemInfo
```protobuf
rpc GetSystemInfo(GetSystemInfoRequest) returns (GetSystemInfoResponse);
```

**Request:**
```protobuf
message GetSystemInfoRequest {}
```

**Response:**
```protobuf
message GetSystemInfoResponse {
  string version = 1;
  string os = 2;
  string arch = 3;
  string kernel = 4;
  int32 containers_running = 5;
  int32 containers_total = 6;
  int32 images_total = 7;
  int32 networks_total = 8;
}
```

##### GetSystemStats
```protobuf
rpc GetSystemStats(GetSystemStatsRequest) returns (GetSystemStatsResponse);
```

**Request:**
```protobuf
message GetSystemStatsRequest {}
```

**Response:**
```protobuf
message GetSystemStatsResponse {
  SystemStats stats = 1;
}

message SystemStats {
  double cpu_usage = 1;
  double memory_usage = 2;
  double disk_usage = 3;
  int64 network_rx = 4;
  int64 network_tx = 5;
  int32 containers_running = 6;
  int32 containers_total = 7;
  int64 memory_total = 8;
  int64 memory_available = 9;
  int64 disk_total = 10;
  int64 disk_available = 11;
}
```

##### StreamMetrics
```protobuf
rpc StreamMetrics(StreamMetricsRequest) returns (stream MetricsData);
```

**Request:**
```protobuf
message StreamMetricsRequest {
  int32 interval = 1; // segundos
  repeated string container_ids = 2;
}
```

**Response:**
```protobuf
message MetricsData {
  string timestamp = 1;
  SystemStats system = 2;
  repeated ContainerMetrics containers = 3;
}

message ContainerMetrics {
  string container_id = 1;
  double cpu_usage = 2;
  int64 memory_usage = 3;
  int64 memory_limit = 4;
  int64 network_rx = 5;
  int64 network_tx = 6;
  int64 disk_usage = 7;
}
```

##### HealthCheck
```protobuf
rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
```

**Request:**
```protobuf
message HealthCheckRequest {}
```

**Response:**
```protobuf
message HealthCheckResponse {
  HealthStatus status = 1;
  string timestamp = 2;
  map<string, HealthStatus> services = 3;
}

enum HealthStatus {
  HEALTHY = 0;
  UNHEALTHY = 1;
  UNKNOWN = 2;
}
```

## Exemplos de Uso

### Cliente em Python

```python
import grpc
from polis_pb2 import *
from polis_pb2_grpc import *

# Conectar ao servidor
channel = grpc.insecure_channel('localhost:9090')
container_client = ContainerServiceStub(channel)
image_client = ImageServiceStub(channel)
system_client = SystemServiceStub(channel)

# Criar container
request = CreateContainerRequest(
    name="meu-container",
    image="alpine:latest",
    command=["echo", "Hello World"],
    ports=[PortMapping(
        host_port=8080,
        container_port=80,
        protocol=Protocol.TCP
    )],
    environment={"NODE_ENV": "production"},
    resource_limits=ResourceLimits(
        memory_limit=1073741824,  # 1GB
        cpu_quota=0.5
    )
)

response = container_client.CreateContainer(request)
print(f"Container criado: {response.container_id}")

# Iniciar container
start_request = StartContainerRequest(container_id=response.container_id)
start_response = container_client.StartContainer(start_request)
print(f"Container iniciado: {start_response.success}")

# Listar containers
list_request = ListContainersRequest()
list_response = container_client.ListContainers(list_request)
for container in list_response.containers:
    print(f"Container: {container.name} - {container.status}")

# Stream de logs
logs_request = StreamLogsRequest(
    container_id=response.container_id,
    follow=True
)
for log_entry in container_client.StreamLogs(logs_request):
    print(f"[{log_entry.timestamp}] {log_entry.level}: {log_entry.message}")

# Baixar imagem
pull_request = PullImageRequest(
    name="nginx",
    tag="alpine",
    registry="docker.io"
)
for response in image_client.PullImage(pull_request):
    print(f"Status: {response.status} - {response.progress}")

# Obter métricas do sistema
stats_request = GetSystemStatsRequest()
stats_response = system_client.GetSystemStats(stats_request)
print(f"CPU: {stats_response.stats.cpu_usage}%")
print(f"Memória: {stats_response.stats.memory_usage}%")

# Stream de métricas
metrics_request = StreamMetricsRequest(interval=5)
for metrics in system_client.StreamMetrics(metrics_request):
    print(f"CPU: {metrics.system.cpu_usage}%")
    for container_metrics in metrics.containers:
        print(f"Container {container_metrics.container_id}: {container_metrics.cpu_usage}%")
```

### Cliente em Go

```go
package main

import (
    "context"
    "log"
    "time"

    "google.golang.org/grpc"
    pb "polis/polis_pb2"
)

func main() {
    // Conectar ao servidor
    conn, err := grpc.Dial("localhost:9090", grpc.WithInsecure())
    if err != nil {
        log.Fatalf("Falha ao conectar: %v", err)
    }
    defer conn.Close()

    // Criar clientes
    containerClient := pb.NewContainerServiceClient(conn)
    imageClient := pb.NewImageServiceClient(conn)
    systemClient := pb.NewSystemServiceClient(conn)

    // Criar container
    createReq := &pb.CreateContainerRequest{
        Name:  "meu-container",
        Image: "alpine:latest",
        Command: []string{"echo", "Hello World"},
        Ports: []*pb.PortMapping{
            {
                HostPort:      8080,
                ContainerPort: 80,
                Protocol:      pb.Protocol_TCP,
            },
        },
        Environment: map[string]string{
            "NODE_ENV": "production",
        },
        ResourceLimits: &pb.ResourceLimits{
            MemoryLimit: 1073741824, // 1GB
            CpuQuota:    0.5,
        },
    }

    createResp, err := containerClient.CreateContainer(context.Background(), createReq)
    if err != nil {
        log.Fatalf("Falha ao criar container: %v", err)
    }
    log.Printf("Container criado: %s", createResp.ContainerId)

    // Iniciar container
    startReq := &pb.StartContainerRequest{
        ContainerId: createResp.ContainerId,
    }
    startResp, err := containerClient.StartContainer(context.Background(), startReq)
    if err != nil {
        log.Fatalf("Falha ao iniciar container: %v", err)
    }
    log.Printf("Container iniciado: %v", startResp.Success)

    // Listar containers
    listReq := &pb.ListContainersRequest{}
    listResp, err := containerClient.ListContainers(context.Background(), listReq)
    if err != nil {
        log.Fatalf("Falha ao listar containers: %v", err)
    }
    for _, container := range listResp.Containers {
        log.Printf("Container: %s - %s", container.Name, container.Status)
    }

    // Obter métricas do sistema
    statsReq := &pb.GetSystemStatsRequest{}
    statsResp, err := systemClient.GetSystemStats(context.Background(), statsReq)
    if err != nil {
        log.Fatalf("Falha ao obter métricas: %v", err)
    }
    log.Printf("CPU: %.2f%%", statsResp.Stats.CpuUsage)
    log.Printf("Memória: %.2f%%", statsResp.Stats.MemoryUsage)
}
```

### Cliente em JavaScript/Node.js

```javascript
const grpc = require('@grpc/grpc-js');
const protoLoader = require('@grpc/proto-loader');

// Carregar definições do protobuf
const packageDefinition = protoLoader.loadSync('polis.proto', {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
});

const polisProto = grpc.loadPackageDefinition(packageDefinition).polis;

// Conectar ao servidor
const client = new polisProto.ContainerService('localhost:9090', 
    grpc.credentials.createInsecure());

// Criar container
const createRequest = {
    name: 'meu-container',
    image: 'alpine:latest',
    command: ['echo', 'Hello World'],
    ports: [{
        hostPort: 8080,
        containerPort: 80,
        protocol: 'TCP'
    }],
    environment: {
        'NODE_ENV': 'production'
    },
    resourceLimits: {
        memoryLimit: 1073741824, // 1GB
        cpuQuota: 0.5
    }
};

client.CreateContainer(createRequest, (error, response) => {
    if (error) {
        console.error('Erro ao criar container:', error);
        return;
    }
    console.log('Container criado:', response.containerId);

    // Iniciar container
    const startRequest = {
        containerId: response.containerId
    };

    client.StartContainer(startRequest, (error, response) => {
        if (error) {
            console.error('Erro ao iniciar container:', error);
            return;
        }
        console.log('Container iniciado:', response.success);
    });
});

// Stream de logs
const logsRequest = {
    containerId: 'container-123',
    follow: true
};

const logsStream = client.StreamLogs(logsRequest);
logsStream.on('data', (logEntry) => {
    console.log(`[${logEntry.timestamp}] ${logEntry.level}: ${logEntry.message}`);
});
logsStream.on('error', (error) => {
    console.error('Erro no stream de logs:', error);
});
```

## Configuração do Servidor

### Arquivo de Configuração

```yaml
# polis-grpc.yaml
server:
  host: "0.0.0.0"
  port: 9090
  max_connections: 1000
  keep_alive_time: 30s
  keep_alive_timeout: 5s
  max_message_size: 4194304  # 4MB

tls:
  enabled: false
  cert_file: "/etc/polis/server.crt"
  key_file: "/etc/polis/server.key"

logging:
  level: "info"
  format: "json"

rate_limiting:
  enabled: true
  requests_per_minute: 1000
```

### Iniciar Servidor

```bash
# Usando configuração padrão
polis-grpc

# Usando arquivo de configuração
polis-grpc --config polis-grpc.yaml

# Com TLS
polis-grpc --tls --cert server.crt --key server.key
```

## Tratamento de Erros

### Códigos de Status gRPC

- `OK` - Operação realizada com sucesso
- `INVALID_ARGUMENT` - Argumentos inválidos
- `NOT_FOUND` - Recurso não encontrado
- `ALREADY_EXISTS` - Recurso já existe
- `PERMISSION_DENIED` - Permissão negada
- `RESOURCE_EXHAUSTED` - Recursos esgotados
- `UNAVAILABLE` - Serviço indisponível
- `INTERNAL` - Erro interno do servidor

### Exemplo de Tratamento de Erro

```python
import grpc
from grpc import StatusCode

try:
    response = container_client.CreateContainer(request)
except grpc.RpcError as e:
    if e.code() == StatusCode.INVALID_ARGUMENT:
        print("Argumentos inválidos:", e.details())
    elif e.code() == StatusCode.ALREADY_EXISTS:
        print("Container já existe:", e.details())
    elif e.code() == StatusCode.RESOURCE_EXHAUSTED:
        print("Recursos esgotados:", e.details())
    else:
        print("Erro gRPC:", e.code(), e.details())
```

## Performance e Otimizações

### Configurações Recomendadas

1. **Keep-Alive:** Habilitado para conexões persistentes
2. **Compression:** Gzip para reduzir tráfego de rede
3. **Connection Pooling:** Reutilizar conexões
4. **Batch Operations:** Usar operações em lote quando possível
5. **Streaming:** Usar streams para dados em tempo real

### Métricas de Performance

- **Latência:** < 1ms para operações locais
- **Throughput:** > 10,000 operações/segundo
- **Concurrent Connections:** > 1,000 conexões simultâneas
- **Memory Usage:** < 100MB por conexão ativa

## Segurança

### Autenticação e Autorização

```protobuf
// Headers de autenticação
message AuthContext {
  string user_id = 1;
  repeated string roles = 2;
  map<string, string> permissions = 3;
}
```

### TLS/SSL

```bash
# Gerar certificados
openssl req -x509 -newkey rsa:4096 -keyout server.key -out server.crt -days 365 -nodes

# Iniciar servidor com TLS
polis-grpc --tls --cert server.crt --key server.key
```

### Rate Limiting

```yaml
rate_limiting:
  enabled: true
  global_limit: 1000  # requisições por minuto
  per_user_limit: 100
  burst_limit: 200
```

## Monitoramento

### Métricas Expostas

- Número de conexões ativas
- Requisições por segundo
- Latência média
- Taxa de erro
- Uso de memória
- Uso de CPU

### Health Checks

```bash
# Verificar saúde do serviço
grpc_health_probe -addr=localhost:9090

# Health check customizado
curl http://localhost:8080/health
```

## Changelog

### v1.0.0 (2024-01-15)
- Serviços básicos (Container, Image, System)
- Streaming de logs e métricas
- Suporte a TLS
- Rate limiting
- Health checks

