# Guia de Migração do Docker para Polis

## Visão Geral

Este guia ajuda você a migrar de Docker para Polis, mostrando as diferenças entre os comandos e conceitos, além de fornecer exemplos práticos de migração.

## Comparação de Comandos

### Gerenciamento de Containers

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker run` | `polis create` + `polis start` | Criar e iniciar container |
| `docker create` | `polis create` | Criar container sem iniciar |
| `docker start` | `polis start` | Iniciar container existente |
| `docker stop` | `polis stop` | Parar container |
| `docker rm` | `polis remove` | Remover container |
| `docker ps` | `polis list` | Listar containers |
| `docker inspect` | `polis inspect` | Inspecionar container |
| `docker logs` | `polis logs` | Ver logs do container |

### Gerenciamento de Imagens

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker pull` | `polis pull` | Baixar imagem |
| `docker images` | `polis images` | Listar imagens |
| `docker rmi` | `polis rmi` | Remover imagem |
| `docker inspect` | `polis inspect` | Inspecionar imagem |
| `docker build` | `polis build` | Construir imagem |

### Gerenciamento de Redes

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker network create` | `polis network create` | Criar rede |
| `docker network ls` | `polis network list` | Listar redes |
| `docker network rm` | `polis network remove` | Remover rede |
| `docker network connect` | `polis network connect` | Conectar container à rede |
| `docker network disconnect` | `polis network disconnect` | Desconectar container da rede |

## Exemplos de Migração

### 1. Container Básico

#### Docker
```bash
# Criar e executar container
docker run -d --name nginx-server -p 8080:80 nginx:alpine

# Verificar status
docker ps

# Ver logs
docker logs nginx-server

# Parar e remover
docker stop nginx-server
docker rm nginx-server
```

#### Polis
```bash
# Criar container
polis create --name nginx-server --image nginx:alpine --port 8080:80

# Iniciar container
polis start nginx-server

# Verificar status
polis list

# Ver logs
polis logs nginx-server

# Parar e remover
polis stop nginx-server
polis remove nginx-server
```

### 2. Container com Variáveis de Ambiente

#### Docker
```bash
docker run -d \
  --name web-app \
  -e NODE_ENV=production \
  -e DATABASE_URL=postgres://localhost:5432/mydb \
  -p 3000:3000 \
  node:16-alpine \
  npm start
```

#### Polis
```bash
polis create \
  --name web-app \
  --image node:16-alpine \
  --env NODE_ENV=production \
  --env DATABASE_URL=postgres://localhost:5432/mydb \
  --port 3000:3000 \
  --command "npm start"
```

### 3. Container com Volumes

#### Docker
```bash
docker run -d \
  --name data-container \
  -v /host/data:/container/data \
  -v /host/config:/container/config:ro \
  alpine:latest \
  tail -f /dev/null
```

#### Polis
```bash
polis create \
  --name data-container \
  --image alpine:latest \
  --volume /host/data:/container/data \
  --volume /host/config:/container/config:ro \
  --command "tail -f /dev/null"
```

### 4. Container com Limites de Recursos

#### Docker
```bash
docker run -d \
  --name limited-container \
  --memory 512m \
  --cpus 0.5 \
  --pids-limit 100 \
  alpine:latest \
  sleep 3600
```

#### Polis
```bash
polis create \
  --name limited-container \
  --image alpine:latest \
  --memory-limit 536870912 \
  --cpu-quota 0.5 \
  --pids-limit 100 \
  --command "sleep 3600"
```

### 5. Rede Personalizada

#### Docker
```bash
# Criar rede
docker network create --subnet=192.168.1.0/24 my-network

# Executar container na rede
docker run -d \
  --name app-container \
  --network my-network \
  --ip 192.168.1.10 \
  nginx:alpine
```

#### Polis
```bash
# Criar rede
polis network create --name my-network --subnet 192.168.1.0/24

# Executar container na rede
polis create \
  --name app-container \
  --image nginx:alpine \
  --network my-network \
  --ip 192.168.1.10
```

## Docker Compose para Polis

### Exemplo de Docker Compose
```yaml
version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    environment:
      - NODE_ENV=production
    volumes:
      - ./html:/usr/share/nginx/html
    networks:
      - app-network

  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=mydb
      - POSTGRES_USER=user
      - POSTGRES_PASSWORD=password
    volumes:
      - postgres_data:/var/lib/postgresql/data
    networks:
      - app-network

volumes:
  postgres_data:

networks:
  app-network:
    driver: bridge
```

### Equivalente em Polis
```bash
# Criar rede
polis network create --name app-network --subnet 172.20.0.0/16

# Criar volume
polis volume create postgres_data

# Criar container web
polis create \
  --name web \
  --image nginx:alpine \
  --port 8080:80 \
  --env NODE_ENV=production \
  --volume ./html:/usr/share/nginx/html \
  --network app-network

# Criar container db
polis create \
  --name db \
  --image postgres:13 \
  --env POSTGRES_DB=mydb \
  --env POSTGRES_USER=user \
  --env POSTGRES_PASSWORD=password \
  --volume postgres_data:/var/lib/postgresql/data \
  --network app-network

# Iniciar containers
polis start web
polis start db
```

## Scripts de Migração

### Script de Migração Automática

```bash
#!/bin/bash
# migrate-docker-to-polis.sh

# Função para converter comando docker run para polis
convert_docker_run() {
    local docker_cmd="$1"
    
    # Extrair nome do container
    local name=$(echo "$docker_cmd" | grep -o '--name [^ ]*' | cut -d' ' -f2)
    
    # Extrair imagem
    local image=$(echo "$docker_cmd" | grep -o '[a-zA-Z0-9/_-]*:[a-zA-Z0-9._-]*' | tail -1)
    
    # Extrair portas
    local ports=$(echo "$docker_cmd" | grep -o '-p [0-9]*:[0-9]*' | sed 's/-p/--port/g')
    
    # Extrair variáveis de ambiente
    local envs=$(echo "$docker_cmd" | grep -o '-e [^ ]*' | sed 's/-e/--env/g')
    
    # Extrair volumes
    local volumes=$(echo "$docker_cmd" | grep -o '-v [^ ]*' | sed 's/-v/--volume/g')
    
    # Construir comando polis
    echo "polis create --name $name --image $image $ports $envs $volumes"
}

# Exemplo de uso
docker_cmd="docker run -d --name nginx-server -p 8080:80 -e NODE_ENV=production nginx:alpine"
polis_cmd=$(convert_docker_run "$docker_cmd")
echo "Comando Polis equivalente: $polis_cmd"
```

### Script de Migração de Docker Compose

```python
#!/usr/bin/env python3
# migrate-compose.py

import yaml
import sys

def convert_compose_to_polis(compose_file):
    with open(compose_file, 'r') as f:
        compose = yaml.safe_load(f)
    
    polis_commands = []
    
    # Criar redes
    if 'networks' in compose:
        for network_name, network_config in compose['networks'].items():
            if network_name != 'default':
                subnet = network_config.get('driver_opts', {}).get('com.docker.network.bridge.subnet', '172.20.0.0/16')
                polis_commands.append(f"polis network create --name {network_name} --subnet {subnet}")
    
    # Criar volumes
    if 'volumes' in compose:
        for volume_name in compose['volumes']:
            polis_commands.append(f"polis volume create {volume_name}")
    
    # Criar containers
    if 'services' in compose:
        for service_name, service_config in compose['services'].items():
            cmd_parts = ["polis create"]
            cmd_parts.append(f"--name {service_name}")
            
            if 'image' in service_config:
                cmd_parts.append(f"--image {service_config['image']}")
            
            if 'ports' in service_config:
                for port in service_config['ports']:
                    if isinstance(port, str):
                        cmd_parts.append(f"--port {port}")
                    else:
                        cmd_parts.append(f"--port {port[0]}:{port[1]}")
            
            if 'environment' in service_config:
                for env in service_config['environment']:
                    cmd_parts.append(f"--env {env}")
            
            if 'volumes' in service_config:
                for volume in service_config['volumes']:
                    cmd_parts.append(f"--volume {volume}")
            
            if 'networks' in service_config:
                for network in service_config['networks']:
                    cmd_parts.append(f"--network {network}")
            
            if 'command' in service_config:
                cmd_parts.append(f"--command \"{' '.join(service_config['command'])}\"")
            
            polis_commands.append(" ".join(cmd_parts))
            polis_commands.append(f"polis start {service_name}")
    
    return polis_commands

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Uso: python migrate-compose.py docker-compose.yml")
        sys.exit(1)
    
    compose_file = sys.argv[1]
    commands = convert_compose_to_polis(compose_file)
    
    print("# Comandos Polis equivalentes:")
    for command in commands:
        print(command)
```

## Diferenças Importantes

### 1. Arquitetura

| Aspecto | Docker | Polis |
|---------|--------|-------|
| **Daemon** | Docker daemon sempre rodando | Sem daemon, execução direta |
| **Imagens** | Docker Hub por padrão | Suporte a múltiplos registries |
| **Rede** | Bridge padrão | Múltiplas opções de rede |
| **Volumes** | Driver de volume único | Drivers personalizáveis |

### 2. Segurança

| Aspecto | Docker | Polis |
|---------|--------|-------|
| **Namespaces** | Automático | Configurável |
| **Cgroups** | Automático | Configurável |
| **Seccomp** | Perfil padrão | Perfis personalizáveis |
| **Capabilities** | Limitado | Controle granular |

### 3. Performance

| Aspecto | Docker | Polis |
|---------|--------|-------|
| **Inicialização** | ~100ms | ~50ms |
| **Uso de Memória** | ~50MB | ~25MB |
| **Overhead** | Alto | Baixo |
| **Throughput** | 100 containers/min | 200 containers/min |

## Checklist de Migração

### Pré-Migração
- [ ] Inventariar todos os containers Docker em uso
- [ ] Documentar configurações de rede e volumes
- [ ] Identificar dependências entre containers
- [ ] Fazer backup das imagens importantes
- [ ] Testar Polis em ambiente de desenvolvimento

### Durante a Migração
- [ ] Converter comandos Docker para Polis
- [ ] Migrar configurações de rede
- [ ] Migrar volumes e dados
- [ ] Testar funcionalidade de cada container
- [ ] Verificar conectividade entre containers
- [ ] Validar logs e monitoramento

### Pós-Migração
- [ ] Monitorar performance e estabilidade
- [ ] Atualizar documentação
- [ ] Treinar equipe nos novos comandos
- [ ] Configurar backup e recuperação
- [ ] Implementar monitoramento avançado

## Troubleshooting

### Problemas Comuns

#### 1. Container não inicia
```bash
# Verificar logs
polis logs container-name

# Verificar configuração
polis inspect container-name

# Verificar recursos disponíveis
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

### Logs e Debugging

```bash
# Logs detalhados
polis --log-level debug create --name test alpine:latest

# Verificar status do sistema
polis system info

# Verificar métricas
polis metrics system

# Health check
polis health
```

## Recursos Adicionais

### Documentação
- [Guia de Instalação](INSTALLATION.md)
- [Referência da API](API_REST.md)
- [Exemplos de Uso](examples/)
- [FAQ](FAQ.md)

### Comunidade
- [GitHub Issues](https://github.com/polis/issues)
- [Discord](https://discord.gg/polis)
- [Stack Overflow](https://stackoverflow.com/tags/polis)

### Suporte
- [Documentação Oficial](https://docs.polis.dev)
- [Tutoriais](https://tutorials.polis.dev)
- [Blog](https://blog.polis.dev)

## Conclusão

A migração do Docker para Polis oferece melhor performance, segurança e flexibilidade. Este guia fornece as ferramentas necessárias para uma migração bem-sucedida, mas sempre teste em ambiente de desenvolvimento antes de migrar em produção.

Para dúvidas ou problemas durante a migração, consulte a documentação oficial ou entre em contato com a comunidade Polis.

