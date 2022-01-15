use clap::{Parser, Subcommand};
use polis_core::{ContainerId, PolisConfig};
use polis_image::ImageManager;
use polis_runtime::{ContainerRuntime, PolisRuntime};
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "polis")]
#[command(about = "Polis - Container Runtime and Orchestration Platform")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Container management
    Container {
        #[command(subcommand)]
        action: ContainerCommands,
    },
    /// Image management
    Image {
        #[command(subcommand)]
        action: ImageCommands,
    },
    /// System information
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },
}

#[derive(Subcommand)]
enum ContainerCommands {
    /// Create a new container
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        image: String,
        #[arg(short, long)]
        command: Option<String>,
    },
    /// Start a container
    Start { name: String },
    /// Stop a container
    Stop { name: String },
    /// List containers
    List,
    /// Remove a container
    Remove { name: String },
    /// Pause a container
    Pause { name: String },
    /// Unpause a container
    Unpause { name: String },
}

#[derive(Subcommand)]
enum ImageCommands {
    /// Pull an image
    Pull { name: String },
    /// List images
    List,
    /// Remove an image
    Remove { name: String },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Show system information
    Info,
    /// Show version
    Version,
}

struct CliState {
    runtime: PolisRuntime,
    image_manager: ImageManager,
    container_names: HashMap<String, ContainerId>,
}

impl CliState {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = PolisConfig::default();
        let runtime = PolisRuntime::new(config.clone());
        runtime.initialize().await?;

        let image_cache_dir = config.storage.root_dir.join("images");
        let image_manager = ImageManager::new(image_cache_dir);

        Ok(Self {
            runtime,
            image_manager,
            container_names: HashMap::new(),
        })
    }

    async fn find_container_by_name(&self, name: &str) -> Option<ContainerId> {
        self.container_names.get(name).cloned()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let mut state = CliState::new().await?;

    match cli.command {
        Commands::Container { action } => match action {
            ContainerCommands::Create {
                name,
                image,
                command,
            } => {
                let command_vec = if let Some(cmd) = command {
                    cmd.split_whitespace().map(|s| s.to_string()).collect()
                } else {
                    vec!["sh".to_string()]
                };

                let container_id = state
                    .runtime
                    .create_container(name.clone(), image, command_vec)
                    .await?;
                state.container_names.insert(name.clone(), container_id);
                println!("Container '{}' criado com sucesso", name);
            }
            ContainerCommands::Start { name } => {
                if let Some(container_id) = state.find_container_by_name(&name).await {
                    state.runtime.start_container(container_id).await?;
                    println!("Container '{}' iniciado", name);
                } else {
                    println!("Container '{}' não encontrado", name);
                }
            }
            ContainerCommands::Stop { name } => {
                if let Some(container_id) = state.find_container_by_name(&name).await {
                    state.runtime.stop_container(container_id).await?;
                    println!("Container '{}' parado", name);
                } else {
                    println!("Container '{}' não encontrado", name);
                }
            }
            ContainerCommands::List => {
                let containers = state.runtime.list_containers().await?;
                if containers.is_empty() {
                    println!("Nenhum container encontrado");
                } else {
                    println!(
                        "{:<20} {:<20} {:<15} {:<20}",
                        "ID", "Nome", "Status", "Imagem"
                    );
                    println!("{}", "-".repeat(75));
                    for container in containers {
                        println!(
                            "{:<20} {:<20} {:<15} {:<20}",
                            container.id.0.to_string()[..8].to_string(),
                            container.name,
                            format!("{:?}", container.status),
                            container.image.0
                        );
                    }
                }
            }
            ContainerCommands::Remove { name } => {
                if let Some(container_id) = state.find_container_by_name(&name).await {
                    state.runtime.remove_container(container_id).await?;
                    state.container_names.remove(&name);
                    println!("Container '{}' removido", name);
                } else {
                    println!("Container '{}' não encontrado", name);
                }
            }
            ContainerCommands::Pause { name } => {
                if let Some(container_id) = state.find_container_by_name(&name).await {
                    state.runtime.pause_container(container_id).await?;
                    println!("Container '{}' pausado", name);
                } else {
                    println!("Container '{}' não encontrado", name);
                }
            }
            ContainerCommands::Unpause { name } => {
                if let Some(container_id) = state.find_container_by_name(&name).await {
                    state.runtime.unpause_container(container_id).await?;
                    println!("Container '{}' despausado", name);
                } else {
                    println!("Container '{}' não encontrado", name);
                }
            }
        },
        Commands::Image { action } => {
            match action {
                ImageCommands::Pull { name } => {
                    println!("📥 Baixando imagem '{}'...", name);
                    match state.image_manager.pull(&name).await {
                        Ok(image) => {
                            println!("✅ Imagem '{}' baixada com sucesso", name);
                            println!("  - ID: {}", image.id.0);
                            println!("  - Tamanho: {} bytes", image.size);
                            println!("  - Arquitetura: {}", image.architecture);
                            println!("  - OS: {}", image.os);
                        }
                        Err(e) => {
                            println!("❌ Erro ao baixar imagem: {}", e);
                        }
                    }
                }
                ImageCommands::List => {
                    println!("📋 Listando imagens...");
                    match state.image_manager.list_images().await {
                        Ok(images) => {
                            if images.is_empty() {
                                println!("  Nenhuma imagem encontrada");
                            } else {
                                println!(
                                    "{:<20} {:<20} {:<15} {:<10}",
                                    "ID", "Nome", "Tag", "Tamanho"
                                );
                                println!("{}", "-".repeat(65));
                                for image in images {
                                    println!(
                                        "{:<20} {:<20} {:<15} {:<10}",
                                        image.id.0[..8].to_string(),
                                        image.name,
                                        image.tag,
                                        format!("{} MB", image.size / 1024 / 1024)
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Erro ao listar imagens: {}", e);
                        }
                    }
                }
                ImageCommands::Remove { name } => {
                    println!("🗑️  Removendo imagem '{}'...", name);
                    // TODO: Implementar remoção de imagem por nome
                    println!("❌ Remoção por nome não implementada ainda");
                }
            }
        }
        Commands::System { action } => match action {
            SystemCommands::Info => {
                println!("Polis System Information");
                println!("Version: 0.1.0");
                println!("Runtime: Rust");
                println!("Architecture: {}", std::env::consts::ARCH);
                println!("OS: {}", std::env::consts::OS);
            }
            SystemCommands::Version => {
                println!("polis version 0.1.0");
            }
        },
    }

    Ok(())
}
