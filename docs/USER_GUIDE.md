# Polis User Guide

This guide provides comprehensive instructions for using Polis Container Runtime and Orchestration Platform.

## Table of Contents

- [Getting Started](#getting-started)
- [Basic Concepts](#basic-concepts)
- [Container Management](#container-management)
- [Image Management](#image-management)
- [Networking](#networking)
- [Storage](#storage)
- [Orchestration](#orchestration)
- [Monitoring](#monitoring)
- [Security](#security)
- [APIs](#apis)
- [Configuration](#configuration)
- [Troubleshooting](#troubleshooting)

## Getting Started

### First Steps

1. **Install Polis** (see [Installation Guide](INSTALLATION.md))
2. **Start the Polis server**
3. **Create your first container**
4. **Explore the features**

### Quick Start

```bash
# Start Polis server
polis-cli server start

# Create a container
polis-cli container create --name my-container --image alpine:latest

# List containers
polis-cli container list

# Start the container
polis-cli container start my-container

# Execute commands in the container
polis-cli container exec my-container -- sh -c "echo 'Hello from Polis!'"

# Stop the container
polis-cli container stop my-container

# Remove the container
polis-cli container remove my-container
```

## Basic Concepts

### Containers
Containers are isolated, lightweight, and portable units that package applications and their dependencies.

### Images
Images are read-only templates used to create containers. They contain the application code, runtime, libraries, and system tools.

### Services
Services are logical groupings of containers that work together to provide a complete application.

### Deployments
Deployments manage the lifecycle of containerized applications, including scaling, updates, and rollbacks.

### Networks
Networks enable communication between containers and external systems.

### Volumes
Volumes provide persistent storage for containers.

## Container Management

### Creating Containers

#### Basic Container
```bash
# Create a simple container
polis-cli container create --name web-server --image nginx:latest

# Create with specific command
polis-cli container create --name app --image node:18 --command "npm start"

# Create with environment variables
polis-cli container create --name db --image postgres:15 \
  --env POSTGRES_DB=mydb \
  --env POSTGRES_USER=user \
  --env POSTGRES_PASSWORD=password
```

#### Advanced Container Configuration
```bash
# Create with resource limits
polis-cli container create --name resource-limited --image alpine:latest \
  --cpu-limit 0.5 \
  --memory-limit 512MB \
  --disk-limit 1GB

# Create with port mapping
polis-cli container create --name web --image nginx:latest \
  --port 8080:80 \
  --port 8443:443

# Create with volume mounts
polis-cli container create --name data-app --image alpine:latest \
  --volume /host/data:/container/data \
  --volume my-volume:/app/storage

# Create with network configuration
polis-cli container create --name network-app --image alpine:latest \
  --network custom-network \
  --ip 192.168.1.100
```

### Managing Container Lifecycle

#### Starting and Stopping
```bash
# Start a container
polis-cli container start my-container

# Stop a container
polis-cli container stop my-container

# Restart a container
polis-cli container restart my-container

# Pause a container
polis-cli container pause my-container

# Resume a container
polis-cli container resume my-container
```

#### Container Information
```bash
# List all containers
polis-cli container list

# List running containers
polis-cli container list --status running

# Get container details
polis-cli container inspect my-container

# View container logs
polis-cli container logs my-container

# Follow logs in real-time
polis-cli container logs --follow my-container

# View container statistics
polis-cli container stats my-container
```

#### Executing Commands
```bash
# Execute a command in a running container
polis-cli container exec my-container -- ls -la

# Execute an interactive shell
polis-cli container exec --interactive --tty my-container -- sh

# Execute with specific user
polis-cli container exec --user root my-container -- whoami
```

### Container Configuration

#### Resource Management
```bash
# Set CPU limits
polis-cli container update my-container --cpu-limit 1.0

# Set memory limits
polis-cli container update my-container --memory-limit 1GB

# Set disk limits
polis-cli container update my-container --disk-limit 2GB

# Set network limits
polis-cli container update my-container --network-limit 100Mbps
```

#### Environment Variables
```bash
# Set environment variables
polis-cli container update my-container --env KEY=value

# Set multiple environment variables
polis-cli container update my-container \
  --env NODE_ENV=production \
  --env PORT=3000 \
  --env DEBUG=true

# Remove environment variable
polis-cli container update my-container --env-remove KEY
```

#### Labels and Annotations
```bash
# Add labels
polis-cli container update my-container --label app=web --label version=1.0

# Add annotations
polis-cli container update my-container --annotation description="Web server container"

# Remove labels
polis-cli container update my-container --label-remove app
```

## Image Management

### Working with Images

#### Pulling Images
```bash
# Pull an image
polis-cli image pull alpine:latest

# Pull from specific registry
polis-cli image pull registry.example.com/my-app:v1.0

# Pull with authentication
polis-cli image pull --username myuser --password mypass private-registry.com/app:latest
```

#### Building Images
```bash
# Build from Dockerfile
polis-cli image build --tag my-app:latest .

# Build with specific Dockerfile
polis-cli image build --tag my-app:latest --file Dockerfile.prod .

# Build with build arguments
polis-cli image build --tag my-app:latest \
  --build-arg NODE_ENV=production \
  --build-arg VERSION=1.0.0 .
```

#### Managing Images
```bash
# List images
polis-cli image list

# Inspect image
polis-cli image inspect alpine:latest

# Remove image
polis-cli image remove alpine:latest

# Tag image
polis-cli image tag my-app:latest my-app:v1.0

# Push image
polis-cli image push my-app:latest

# Push to specific registry
polis-cli image push registry.example.com/my-app:latest
```

### Image Security

#### Scanning Images
```bash
# Scan image for vulnerabilities
polis-cli image scan alpine:latest

# Scan with specific policy
polis-cli image scan --policy security-policy.yaml alpine:latest

# Scan and generate report
polis-cli image scan --output report.json alpine:latest
```

#### Image Signing
```bash
# Sign image
polis-cli image sign my-app:latest

# Verify image signature
polis-cli image verify my-app:latest

# List image signatures
polis-cli image signatures my-app:latest
```

## Networking

### Network Management

#### Creating Networks
```bash
# Create a bridge network
polis-cli network create --driver bridge my-network

# Create with custom subnet
polis-cli network create --driver bridge --subnet 192.168.1.0/24 my-network

# Create with custom gateway
polis-cli network create --driver bridge \
  --subnet 192.168.1.0/24 \
  --gateway 192.168.1.1 \
  my-network
```

#### Managing Networks
```bash
# List networks
polis-cli network list

# Inspect network
polis-cli network inspect my-network

# Connect container to network
polis-cli network connect my-network my-container

# Disconnect container from network
polis-cli network disconnect my-network my-container

# Remove network
polis-cli network remove my-network
```

### Port Forwarding

#### Basic Port Forwarding
```bash
# Forward host port to container port
polis-cli port forward 8080:80 my-container

# Forward multiple ports
polis-cli port forward 8080:80 8443:443 my-container

# Forward with specific protocol
polis-cli port forward --protocol tcp 8080:80 my-container
```

#### Advanced Port Configuration
```bash
# Forward with specific host interface
polis-cli port forward --host 0.0.0.0 8080:80 my-container

# Forward with port range
polis-cli port forward 8080-8090:80-90 my-container

# List port forwards
polis-cli port list

# Remove port forward
polis-cli port remove 8080:80 my-container
```

### DNS Configuration

#### DNS Management
```bash
# Add DNS server
polis-cli dns add --server 8.8.8.8

# Add custom DNS record
polis-cli dns add --record "api.example.com A 192.168.1.100"

# List DNS configuration
polis-cli dns list

# Remove DNS server
polis-cli dns remove --server 8.8.8.8
```

## Storage

### Volume Management

#### Creating Volumes
```bash
# Create a volume
polis-cli volume create my-volume

# Create with specific driver
polis-cli volume create --driver local my-volume

# Create with options
polis-cli volume create --driver local \
  --opt type=tmpfs \
  --opt device=tmpfs \
  --opt o=size=100m \
  my-volume
```

#### Managing Volumes
```bash
# List volumes
polis-cli volume list

# Inspect volume
polis-cli volume inspect my-volume

# Mount volume to container
polis-cli container create --name app --image alpine:latest \
  --volume my-volume:/data

# Remove volume
polis-cli volume remove my-volume
```

### Bind Mounts

#### Using Bind Mounts
```bash
# Mount host directory
polis-cli container create --name app --image alpine:latest \
  --bind /host/path:/container/path

# Mount with read-only option
polis-cli container create --name app --image alpine:latest \
  --bind /host/path:/container/path:ro

# Mount with specific options
polis-cli container create --name app --image alpine:latest \
  --bind /host/path:/container/path:rw,noexec,nosuid
```

## Orchestration

### Service Discovery

#### Registering Services
```bash
# Register a service
polis-cli service register --name web-service \
  --port 8080 \
  --protocol http \
  --health-check /health

# Register with multiple endpoints
polis-cli service register --name web-service \
  --endpoint 192.168.1.10:8080 \
  --endpoint 192.168.1.11:8080 \
  --endpoint 192.168.1.12:8080
```

#### Service Management
```bash
# List services
polis-cli service list

# Get service details
polis-cli service inspect web-service

# Update service
polis-cli service update web-service --port 8081

# Remove service
polis-cli service remove web-service
```

### Load Balancing

#### Load Balancer Configuration
```bash
# Create load balancer
polis-cli loadbalancer create --name web-lb \
  --algorithm round-robin \
  --service web-service

# Configure load balancer
polis-cli loadbalancer configure web-lb \
  --algorithm least-connections \
  --sticky-sessions \
  --health-check
```

#### Load Balancer Management
```bash
# List load balancers
polis-cli loadbalancer list

# Get load balancer status
polis-cli loadbalancer status web-lb

# Update load balancer
polis-cli loadbalancer update web-lb --algorithm weighted-round-robin

# Remove load balancer
polis-cli loadbalancer remove web-lb
```

### Auto Scaling

#### Scaling Policies
```bash
# Create scaling policy
polis-cli scaling policy create --name web-scaling \
  --deployment web-deployment \
  --min-replicas 1 \
  --max-replicas 10 \
  --target-cpu 70 \
  --target-memory 80

# Update scaling policy
polis-cli scaling policy update web-scaling \
  --target-cpu 80 \
  --target-memory 85
```

#### Deployment Management
```bash
# Create deployment
polis-cli deployment create --name web-deployment \
  --image nginx:latest \
  --replicas 3 \
  --port 80

# Scale deployment
polis-cli deployment scale web-deployment --replicas 5

# Update deployment
polis-cli deployment update web-deployment --image nginx:1.21

# Rollback deployment
polis-cli deployment rollback web-deployment
```

## Monitoring

### Health Monitoring

#### Health Checks
```bash
# Create health check
polis-cli health check create --name web-health \
  --target web-service \
  --type http \
  --path /health \
  --interval 30s \
  --timeout 5s

# List health checks
polis-cli health check list

# Get health check status
polis-cli health check status web-health
```

#### Metrics Collection
```bash
# View container metrics
polis-cli metrics container my-container

# View system metrics
polis-cli metrics system

# View service metrics
polis-cli metrics service web-service

# Export metrics
polis-cli metrics export --format prometheus --output metrics.txt
```

### Logging

#### Log Management
```bash
# View container logs
polis-cli logs my-container

# View logs with filters
polis-cli logs my-container --since 1h --until now

# View logs from multiple containers
polis-cli logs --service web-service

# Export logs
polis-cli logs export --output logs.json
```

## Security

### Security Policies

#### AppArmor Profiles
```bash
# Create AppArmor profile
polis-cli security apparmor create --name web-profile \
  --profile apparmor-web.conf

# Apply AppArmor profile
polis-cli container update my-container --apparmor web-profile
```

#### SELinux Contexts
```bash
# Set SELinux context
polis-cli container update my-container \
  --selinux-context system_u:system_r:container_t:s0
```

#### Capabilities
```bash
# Add capabilities
polis-cli container update my-container --cap-add NET_ADMIN

# Remove capabilities
polis-cli container update my-container --cap-drop ALL
```

### Network Security

#### Firewall Rules
```bash
# Create firewall rule
polis-cli firewall rule create --name allow-http \
  --action allow \
  --protocol tcp \
  --port 80 \
  --source 0.0.0.0/0

# Apply firewall rule
polis-cli firewall apply --container my-container --rule allow-http
```

## APIs

### REST API

#### Basic Usage
```bash
# Start API server
polis-cli api start --rest-port 8080

# Test API endpoint
curl http://localhost:8080/health

# List containers via API
curl http://localhost:8080/containers

# Create container via API
curl -X POST http://localhost:8080/containers \
  -H "Content-Type: application/json" \
  -d '{"name":"api-container","image":"alpine:latest"}'
```

#### Authentication
```bash
# Get authentication token
TOKEN=$(polis-cli auth login --username admin --password admin)

# Use token in API calls
curl -H "Authorization: Bearer $TOKEN" http://localhost:8080/containers
```

### gRPC API

#### Using gRPC Client
```bash
# Start gRPC server
polis-cli api start --grpc-port 50051

# Use grpcurl to test gRPC
grpcurl -plaintext localhost:50051 list

# Call gRPC method
grpcurl -plaintext -d '{"name":"grpc-container","image":"alpine:latest"}' \
  localhost:50051 polis.ContainerService/CreateContainer
```

## Configuration

### Configuration Management

#### View Configuration
```bash
# Show current configuration
polis-cli config show

# Show specific configuration section
polis-cli config show --section runtime

# Show configuration file location
polis-cli config show --file
```

#### Update Configuration
```bash
# Update configuration
polis-cli config set runtime.root_dir /var/lib/polis

# Update multiple settings
polis-cli config set \
  runtime.root_dir /var/lib/polis \
  api.rest_port 8080 \
  api.grpc_port 50051

# Reset configuration
polis-cli config reset
```

#### Configuration Files
```bash
# Edit configuration file
polis-cli config edit

# Validate configuration
polis-cli config validate

# Export configuration
polis-cli config export --output config.yaml
```

### Environment Variables

#### Setting Environment Variables
```bash
# Set environment variable
export POLIS_LOG_LEVEL=debug

# Set multiple environment variables
export POLIS_LOG_LEVEL=debug
export POLIS_CONFIG_FILE=/etc/polis/config.yaml
export POLIS_DATA_DIR=/var/lib/polis
```

## Troubleshooting

### Common Issues

#### Container Issues
```bash
# Container won't start
polis-cli container logs my-container
polis-cli container inspect my-container

# Container is consuming too many resources
polis-cli container stats my-container
polis-cli container update my-container --cpu-limit 0.5 --memory-limit 512MB
```

#### Network Issues
```bash
# Container can't reach external network
polis-cli network inspect my-network
polis-cli container inspect my-container

# Port forwarding not working
polis-cli port list
polis-cli port forward --list
```

#### Performance Issues
```bash
# Check system resources
polis-cli system info
polis-cli metrics system

# Check container performance
polis-cli container stats my-container
polis-cli container top my-container
```

### Debugging

#### Enable Debug Logging
```bash
# Enable debug logging
export RUST_LOG=debug
polis-cli server start

# Enable specific module logging
export RUST_LOG=polis_runtime=debug,polis_api=info
polis-cli server start
```

#### Collect Debug Information
```bash
# Collect system information
polis-cli debug collect --output debug-info.tar.gz

# Collect container information
polis-cli debug collect --container my-container --output container-debug.tar.gz
```

### Getting Help

#### Documentation
- [API Reference](API_REFERENCE.md)
- [Configuration Reference](CONFIGURATION.md)
- [Troubleshooting Guide](TROUBLESHOOTING.md)

#### Community Support
- [GitHub Issues](https://github.com/polis-project/polis/issues)
- [Discord Community](https://discord.gg/polis)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/polis)

#### Professional Support
- [Enterprise Support](https://polis.dev/support)
- [Training](https://polis.dev/training)
- [Consulting](https://polis.dev/consulting)

---

**Note**: This user guide is regularly updated. For the latest information, always refer to the [official documentation](https://docs.polis.dev).
