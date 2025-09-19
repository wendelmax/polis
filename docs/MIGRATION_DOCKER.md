# Guia de Migração do Docker para Polis

Este guia ajuda você a migrar do Docker para o Polis Container Runtime, incluindo comandos equivalentes, scripts de migração e melhores práticas.

## 🎯 Visão Geral

### Por que Migrar?

- **Performance**: Polis é significativamente mais rápido
- **Segurança**: Isolamento mais robusto
- **Simplicidade**: Interface mais intuitiva
- **Compatibilidade**: Suporte completo ao padrão OCI
- **Modularidade**: Arquitetura mais flexível

### Compatibilidade

- ✅ **Imagens OCI**: 100% compatível
- ✅ **Dockerfiles**: Suporte completo
- ✅ **Docker Compose**: Conversão automática
- ✅ **Registries**: Docker Hub, ECR, GCR, etc.
- ✅ **Volumes**: Compatibilidade total
- ✅ **Redes**: Funcionalidades equivalentes

## 📋 Mapeamento de Comandos

### Comandos Básicos

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker run` | `polis container create` + `polis container start` | Criar e executar container |
| `docker ps` | `polis container list` | Listar containers |
| `docker images` | `polis image list` | Listar imagens |
| `docker pull` | `polis image pull` | Baixar imagem |
| `docker push` | `polis image push` | Enviar imagem |
| `docker build` | `polis build` | Build de imagem |
| `docker logs` | `polis container logs` | Ver logs |
| `docker exec` | `polis container exec` | Executar comando |
| `docker stop` | `polis container stop` | Parar container |
| `docker rm` | `polis container remove` | Remover container |
| `docker rmi` | `polis image remove` | Remover imagem |

### Comandos de Rede

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker network create` | `polis network create` | Criar rede |
| `docker network ls` | `polis network list` | Listar redes |
| `docker network connect` | `polis network connect` | Conectar à rede |
| `docker network disconnect` | `polis network disconnect` | Desconectar da rede |
| `docker network rm` | `polis network remove` | Remover rede |

### Comandos de Volume

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker volume create` | `polis volume create` | Criar volume |
| `docker volume ls` | `polis volume list` | Listar volumes |
| `docker volume inspect` | `polis volume inspect` | Inspecionar volume |
| `docker volume rm` | `polis volume remove` | Remover volume |

### Comandos de Sistema

| Docker | Polis | Descrição |
|--------|-------|-----------|
| `docker info` | `polis system info` | Informações do sistema |
| `docker version` | `polis --version` | Versão |
| `docker system prune` | `polis system cleanup` | Limpeza do sistema |
| `docker stats` | `polis stats` | Estatísticas |

## 🔄 Scripts de Migração

### Script de Migração Automática

```bash
#!/bin/bash
# migrate-docker-to-polis.sh

echo "=== Migração do Docker para Polis ==="

# Verificar se o Polis está instalado
if ! command -v polis &> /dev/null; then
    echo "❌ Polis não está instalado. Instale primeiro."
    exit 1
fi

# Migrar containers
echo "1. Migrando containers..."
docker ps --format "table {{.Names}}\t{{.Image}}\t{{.Ports}}" | tail -n +2 | while read name image ports; do
    if [ ! -z "$name" ]; then
        echo "Migrando container: $name"
        
        # Extrair variáveis de ambiente
        env_vars=$(docker inspect $name --format '{{range .Config.Env}}{{println .}}{{end}}' | tr '\n' ' ')
        
        # Criar container no Polis
        polis container create --name "$name" --image "$image" --port "$ports" --env $env_vars
    fi
done

# Migrar imagens
echo "2. Migrando imagens..."
docker images --format "table {{.Repository}}\t{{.Tag}}" | tail -n +2 | while read repo tag; do
    if [ ! -z "$repo" ]; then
        echo "Migrando imagem: $repo:$tag"
        polis image pull "$repo:$tag"
    fi
done

# Migrar volumes
echo "3. Migrando volumes..."
docker volume ls --format "{{.Name}}" | while read volume; do
    if [ ! -z "$volume" ]; then
        echo "Migrando volume: $volume"
        polis volume create --name "$volume"
    fi
done

# Migrar redes
echo "4. Migrando redes..."
docker network ls --format "{{.Name}}" | grep -v "bridge\|host\|none" | while read network; do
    if [ ! -z "$network" ]; then
        echo "Migrando rede: $network"
        polis network create --name "$network"
    fi
done

echo "✅ Migração concluída!"
```

### Script de Conversão Docker Compose

```bash
#!/bin/bash
# convert-compose-to-polis.sh

if [ $# -eq 0 ]; then
    echo "Uso: $0 <docker-compose.yml>"
    exit 1
fi

compose_file=$1
output_file="polis-deploy.yaml"

echo "Convertendo $compose_file para $output_file..."

# Converter serviços
echo "services:" > $output_file
echo "  polis:" >> $output_file
echo "    runtime: polis" >> $output_file
echo "" >> $output_file

# Extrair serviços do docker-compose.yml
yq eval '.services | keys[]' $compose_file | while read service; do
    echo "Convertendo serviço: $service"
    
    # Extrair configuração do serviço
    image=$(yq eval ".services.$service.image" $compose_file)
    ports=$(yq eval ".services.$service.ports[]" $compose_file 2>/dev/null || echo "")
    environment=$(yq eval ".services.$service.environment" $compose_file 2>/dev/null || echo "")
    volumes=$(yq eval ".services.$service.volumes[]" $compose_file 2>/dev/null || echo "")
    
    # Gerar comando polis
    cmd="polis deploy create --name $service --image $image"
    
    if [ ! -z "$ports" ]; then
        cmd="$cmd --port $ports"
    fi
    
    if [ ! -z "$environment" ]; then
        cmd="$cmd --env $environment"
    fi
    
    if [ ! -z "$volumes" ]; then
        cmd="$cmd --volume $volumes"
    fi
    
    echo "  $service:" >> $output_file
    echo "    command: \"$cmd\"" >> $output_file
    echo "" >> $output_file
done

echo "✅ Conversão concluída! Arquivo gerado: $output_file"
```

## 🐳 Exemplos de Migração

### Exemplo 1: Aplicação Web Simples

#### Docker
```bash
# Criar e executar container
docker run -d --name nginx -p 8080:80 nginx:alpine

# Ver logs
docker logs nginx

# Parar e remover
docker stop nginx
docker rm nginx
```

#### Polis
```bash
# Criar container
polis container create --name nginx --image nginx:alpine --port 8080:80

# Executar container
polis container start nginx

# Ver logs
polis container logs nginx

# Parar e remover
polis container stop nginx
polis container remove nginx
```

### Exemplo 2: Aplicação com Banco de Dados

#### Docker
```bash
# Criar rede
docker network create app-net

# Criar banco de dados
docker run -d --name db --network app-net \
  -e POSTGRES_DB=myapp -e POSTGRES_PASSWORD=secret \
  postgres:13

# Criar aplicação
docker run -d --name app --network app-net \
  -p 3000:3000 -e DATABASE_URL=postgres://db:5432/myapp \
  node:16
```

#### Polis
```bash
# Criar rede
polis network create --name app-net --subnet 172.20.0.0/16

# Criar banco de dados
polis container create --name db --image postgres:13 \
  --network app-net \
  --env POSTGRES_DB=myapp \
  --env POSTGRES_PASSWORD=secret

# Criar aplicação
polis container create --name app --image node:16 \
  --network app-net \
  --port 3000:3000 \
  --env DATABASE_URL=postgres://db:5432/myapp

# Executar containers
polis container start db
polis container start app
```

### Exemplo 3: Docker Compose

#### docker-compose.yml
```yaml
version: '3.8'
services:
  web:
    image: nginx:alpine
    ports:
      - "8080:80"
    environment:
      - NGINX_HOST=localhost
    volumes:
      - ./html:/usr/share/nginx/html
  
  db:
    image: postgres:13
    environment:
      - POSTGRES_DB=myapp
      - POSTGRES_PASSWORD=secret
    volumes:
      - db-data:/var/lib/postgresql/data

volumes:
  db-data:
```

#### Polis Deploy
```bash
# Deploy da aplicação web
polis deploy create --name web --image nginx:alpine \
  --replicas 1 \
  --port 8080:80 \
  --env NGINX_HOST=localhost \
  --volume ./html:/usr/share/nginx/html

# Deploy do banco de dados
polis deploy create --name db --image postgres:13 \
  --replicas 1 \
  --env POSTGRES_DB=myapp \
  --env POSTGRES_PASSWORD=secret \
  --volume db-data:/var/lib/postgresql/data
```

## 🔧 Configuração de Migração

### Configurar Registries

```bash
# Migrar configuração do Docker
cp ~/.docker/config.json ~/.polis/registries.json

# Ou configurar manualmente
polis registry add docker.io --username your-username --password your-token
polis registry add registry.example.com --username admin --password secret
```

### Migrar Volumes

```bash
# Listar volumes do Docker
docker volume ls

# Criar volumes equivalentes no Polis
for volume in $(docker volume ls --format "{{.Name}}"); do
    polis volume create --name "$volume"
done
```

### Migrar Redes

```bash
# Listar redes do Docker
docker network ls

# Criar redes equivalentes no Polis
for network in $(docker network ls --format "{{.Name}}" | grep -v "bridge\|host\|none"); do
    polis network create --name "$network"
done
```

## 🚀 Otimizações Pós-Migração

### Performance

```bash
# Configurar otimizações de performance
polis config set runtime.optimization.enabled true
polis config set runtime.optimization.memory_limit 512m
polis config set runtime.optimization.cpu_limit 0.5
```

### Segurança

```bash
# Configurar perfil de segurança
polis config set security.default_profile apparmor:docker-default
polis config set security.capabilities.drop ALL
polis config set security.capabilities.add NET_BIND_SERVICE
```

### Monitoramento

```bash
# Configurar monitoramento
polis config set monitoring.enabled true
polis config set monitoring.metrics_interval 30s
polis config set monitoring.log_level info
```

## 🧪 Validação da Migração

### Script de Validação

```bash
#!/bin/bash
# validate-migration.sh

echo "=== Validação da Migração ==="

# Verificar containers
echo "1. Verificando containers..."
docker ps --format "{{.Names}}" | while read container; do
    if polis container list | grep -q "$container"; then
        echo "✅ Container $container migrado com sucesso"
    else
        echo "❌ Container $container não foi migrado"
    fi
done

# Verificar imagens
echo "2. Verificando imagens..."
docker images --format "{{.Repository}}:{{.Tag}}" | while read image; do
    if polis image list | grep -q "$image"; then
        echo "✅ Imagem $image migrada com sucesso"
    else
        echo "❌ Imagem $image não foi migrada"
    fi
done

# Verificar volumes
echo "3. Verificando volumes..."
docker volume ls --format "{{.Name}}" | while read volume; do
    if polis volume list | grep -q "$volume"; then
        echo "✅ Volume $volume migrado com sucesso"
    else
        echo "❌ Volume $volume não foi migrado"
    fi
done

echo "=== Validação Concluída ==="
```

## 🔄 Rollback

### Voltar para Docker

```bash
# Parar Polis
polis system stop

# Iniciar Docker
sudo systemctl start docker

# Migrar containers de volta
polis container list --format "{{.Names}}" | while read container; do
    image=$(polis container inspect $container --format "{{.Image}}")
    ports=$(polis container inspect $container --format "{{.Ports}}")
    
    docker run -d --name "$container" --port "$ports" "$image"
done
```

## 📚 Recursos Adicionais

### Documentação
- [Guia de Instalação](INSTALLATION.md)
- [Tutorial Completo](TUTORIAL.md)
- [Referência da API](API_REST.md)

### Ferramentas
- **Docker2Polis**: Script de migração automática
- **Compose2Polis**: Conversor de Docker Compose
- **Polis Migration Tool**: Ferramenta oficial de migração

### Suporte
- **GitHub Issues**: [github.com/polis/polis/issues](https://github.com/polis/polis/issues)
- **Discord**: [discord.gg/polis](https://discord.gg/polis)
- **Stack Overflow**: [stackoverflow.com/tags/polis](https://stackoverflow.com/tags/polis)

## 🎯 Próximos Passos

### Após a Migração

1. **Testar Aplicações**: Verificar se todas as aplicações funcionam
2. **Configurar Monitoramento**: Implementar monitoramento completo
3. **Otimizar Performance**: Aplicar otimizações específicas
4. **Treinar Equipe**: Capacitar equipe no uso do Polis
5. **Documentar Processo**: Documentar o processo de migração

### Melhores Práticas

- **Migração Gradual**: Migre aplicação por aplicação
- **Testes Extensivos**: Teste cada migração antes de prosseguir
- **Backup**: Sempre faça backup antes da migração
- **Monitoramento**: Monitore a performance após a migração
- **Documentação**: Documente todas as mudanças

---

**Última atualização**: Janeiro 2025  
**Versão**: 1.0.0  
**Status**: Ativa e mantida

**Polis** - Container Runtime moderno, seguro e eficiente. Feito com ❤ no Brasil.
