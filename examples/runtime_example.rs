use polis_core::{PolisConfig, Logger, LogLevel};
use polis_runtime::{PolisRuntime, ContainerRuntime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar configuração
    let config = PolisConfig::default();
    
    // Inicializar logger
    let logger = Logger::new(LogLevel::Info, None);
    logger.init()?;
    
    // Criar runtime
    let runtime = PolisRuntime::new(config);
    runtime.initialize().await?;
    
    println!(" Polis Runtime Example");
    println!("========================");
    
    // Criar container
    println!("\n Criando container...");
    let container_id = runtime.create_container(
        "meu-container".to_string(),
        "alpine:latest".to_string(),
        vec!["sh".to_string(), "-c".to_string(), "echo 'Hello from Polis!'".to_string()]
    ).await?;
    
    println!(" Container criado com ID: {}", container_id.0);
    
    // Listar containers
    println!("\n� Listando containers...");
    let containers = runtime.list_containers().await?;
    for container in containers {
        println!("  - {} ({}) - {:?}", container.name, container.id.0, container.status);
    }
    
    // Iniciar container
    println!("\n▶  Iniciando container...");
    runtime.start_container(container_id.clone()).await?;
    println!(" Container iniciado");
    
    // Listar containers novamente
    println!("\n� Listando containers após start...");
    let containers = runtime.list_containers().await?;
    for container in containers {
        println!("  - {} ({}) - {:?}", container.name, container.id.0, container.status);
    }
    
    // Parar container
    println!("\n⏹  Parando container...");
    runtime.stop_container(container_id.clone()).await?;
    println!(" Container parado");
    
    // Listar containers final
    println!("\n� Listando containers após stop...");
    let containers = runtime.list_containers().await?;
    for container in containers {
        println!("  - {} ({}) - {:?}", container.name, container.id.0, container.status);
    }
    
    // Remover container
    println!("\n  Removendo container...");
    runtime.remove_container(container_id).await?;
    println!(" Container removido");
    
    // Listar containers final
    println!("\n� Listando containers após remoção...");
    let containers = runtime.list_containers().await?;
    if containers.is_empty() {
        println!("  Nenhum container encontrado");
    } else {
        for container in containers {
            println!("  - {} ({}) - {:?}", container.name, container.id.0, container.status);
        }
    }
    
    println!("\n Exemplo concluído com sucesso!");
    
    Ok(())
}

