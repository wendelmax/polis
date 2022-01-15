# Tutorial 4: Criando e Executando Containers

## 📖 Objetivos
- Criar containers com o Polis
- Executar comandos em containers
- Gerenciar o estado dos containers
- Entender os diferentes modos de execução

## ⏱️ Duração
- **Estimada**: 20 minutos
- **Nível**: Iniciante

## 📋 Pré-requisitos
- Polis instalado e configurado
- Tutorial 1 concluído
- Conhecimento básico de containers

## 🎬 Roteiro do Vídeo

### Introdução (0:00 - 2:00)
- O que são containers
- Diferenças entre containers e VMs
- Vantagens dos containers
- O que será coberto

### Conceitos Básicos (2:00 - 5:00)
- Container vs Image
- Namespaces e isolamento
- Cgroups e limites de recursos
- Lifecycle de um container

### Criando o Primeiro Container (5:00 - 10:00)
```bash
# Listar imagens disponíveis
polis image list

# Baixar uma imagem
polis image pull alpine:latest

# Criar um container
polis container create \
  --name my-first-container \
  --image alpine:latest \
  --command "echo" "Hello, Polis!"

# Verificar container criado
polis container list
```

### Executando Containers (10:00 - 15:00)
```bash
# Iniciar o container
polis container start my-first-container

# Ver logs do container
polis container logs my-first-container

# Executar comando interativo
polis container create \
  --name interactive-container \
  --image alpine:latest \
  --interactive \
  --tty

polis container start interactive-container
polis container exec interactive-container sh
```

### Gerenciando Containers (15:00 - 18:00)
```bash
# Ver status dos containers
polis container list --all

# Parar um container
polis container stop my-first-container

# Remover um container
polis container remove my-first-container

# Limpar containers parados
polis container prune
```

### Demonstração Prática (18:00 - 20:00)
```bash
# Exemplo prático: servidor web
polis container create \
  --name web-server \
  --image nginx:alpine \
  --port 8080:80 \
  --detach

polis container start web-server

# Testar o servidor
curl http://localhost:8080
```

## 💻 Código de Exemplo

### Script de Demonstração
```bash
#!/bin/bash
# container_demo.sh

echo "🚀 Demonstração de Containers com Polis"

# 1. Baixar imagem
echo "📥 Baixando imagem Alpine..."
polis image pull alpine:latest

# 2. Container simples
echo "📦 Criando container simples..."
polis container create \
  --name demo-simple \
  --image alpine:latest \
  --command "echo" "Hello from Polis!"

polis container start demo-simple
polis container logs demo-simple

# 3. Container interativo
echo "🖥️ Criando container interativo..."
polis container create \
  --name demo-interactive \
  --image alpine:latest \
  --interactive \
  --tty \
  --command "sh"

polis container start demo-interactive
echo "Container interativo iniciado. Use 'polis container exec demo-interactive sh' para acessar"

# 4. Container com portas
echo "🌐 Criando servidor web..."
polis container create \
  --name demo-web \
  --image nginx:alpine \
  --port 8080:80 \
  --detach

polis container start demo-web
echo "Servidor web disponível em http://localhost:8080"

# 5. Limpeza
echo "🧹 Limpando containers..."
polis container stop demo-simple demo-interactive demo-web
polis container remove demo-simple demo-interactive demo-web

echo "✅ Demonstração concluída!"
```

### Configurações Avançadas
```bash
# Container com variáveis de ambiente
polis container create \
  --name app-with-env \
  --image node:alpine \
  --env NODE_ENV=production \
  --env PORT=3000 \
  --command "node" "app.js"

# Container com volumes
polis container create \
  --name app-with-volume \
  --image postgres:13 \
  --volume /data/postgres:/var/lib/postgresql/data \
  --env POSTGRES_PASSWORD=secret

# Container com limites de recursos
polis container create \
  --name limited-container \
  --image alpine:latest \
  --memory 512m \
  --cpus 0.5 \
  --command "stress" "--cpu" "1"
```

### Exemplo com Múltiplos Containers
```bash
#!/bin/bash
# multi_container_demo.sh

echo "🏗️ Demonstração de Múltiplos Containers"

# 1. Banco de dados
echo "🗄️ Iniciando banco de dados..."
polis container create \
  --name database \
  --image postgres:13 \
  --env POSTGRES_DB=myapp \
  --env POSTGRES_USER=user \
  --env POSTGRES_PASSWORD=password \
  --volume postgres_data:/var/lib/postgresql/data

polis container start database

# 2. Aplicação
echo "🚀 Iniciando aplicação..."
polis container create \
  --name app \
  --image node:alpine \
  --env DATABASE_URL=postgres://user:password@database:5432/myapp \
  --port 3000:3000 \
  --link database:db

polis container start app

# 3. Proxy reverso
echo "🔄 Iniciando proxy reverso..."
polis container create \
  --name proxy \
  --image nginx:alpine \
  --port 80:80 \
  --link app:backend

polis container start proxy

echo "✅ Stack completo iniciado!"
echo "Aplicação disponível em http://localhost"
```

## 🔗 Recursos Adicionais
- [Documentação de Containers](../CONTAINERS.md)
- [Referência de Comandos](../COMANDOS.md)
- [Exemplos Avançados](../EXEMPLOS.md)
- [Troubleshooting](../TROUBLESHOOTING.md)

## ❓ Exercícios
1. **Container Básico**: Crie um container que exiba "Hello, World!"
2. **Container Interativo**: Crie um container interativo e explore o sistema
3. **Servidor Web**: Crie um container com nginx e acesse via browser
4. **Múltiplos Containers**: Crie uma stack com banco de dados e aplicação

## 🎯 Próximos Tutoriais
- [Tutorial 5: Gerenciando o Lifecycle de Containers](05-lifecycle-containers.md)
- [Tutorial 6: Configurando Recursos e Limites](06-recursos-limites.md)

## 📝 Notas do Instrutor

### Pontos Importantes
- Explicar a diferença entre `create` e `start`
- Mostrar como usar `--detach` para containers em background
- Enfatizar a importância de limpar containers não utilizados
- Demonstrar diferentes modos de execução

### Dicas de Apresentação
- Usar containers pequenos (Alpine) para demonstrações rápidas
- Mostrar a saída de cada comando
- Explicar o que acontece em cada etapa
- Usar cores diferentes para diferentes tipos de comandos

### Possíveis Problemas
- **Imagem não encontrada**: Verificar se a imagem foi baixada
- **Porta ocupada**: Usar portas diferentes
- **Permissões**: Verificar se o usuário tem permissões necessárias
- **Recursos**: Verificar se há recursos suficientes

### Tempo Sugerido por Seção
- Introdução: 2 min
- Conceitos: 3 min
- Criação: 5 min
- Execução: 5 min
- Gerenciamento: 3 min
- Demonstração: 2 min
- **Total**: 20 min
