use clap::{Parser, Subcommand};
use polis_core::{ContainerId, PolisConfig};
use polis_image::{ImageManager, ImageSearchManager, ImageCleanupManager, SearchOptions, CleanupOptions};
use polis_runtime::{ContainerRuntime, PolisRuntime};
use polis_stats::{ContainerStatsCollector, ContainerStatsSummary};
use polis_build::{ImageBuilder, BuildContext, BuildOptions};
use polis_network::{BridgeManager, IpamManager, DnsManager, FirewallManager, PortForwardingManager};
use polis_storage::{VolumeManager, VolumeDriver, MountOptions};
use polis_orchestrator::{
    Orchestrator, OrchestratorConfig, DeploymentSpec, PortSpec, HealthCheckSpec,
    ScalingPolicySpec, ResourceSpec, DeploymentStatusResult, DeploymentStatusType
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

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
    /// Registry configuration
    Registry {
        #[command(subcommand)]
        action: RegistryCommands,
    },
    /// Container statistics and monitoring
    Stats {
        #[command(subcommand)]
        action: StatsCommands,
    },
    /// Network management
    Network {
        #[command(subcommand)]
        action: NetworkCommands,
    },
    /// Volume management
    Volume {
        #[command(subcommand)]
        action: VolumeCommands,
    },
    /// Orchestration and deployment
    Deploy {
        #[command(subcommand)]
        action: DeployCommands,
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
    /// Build an image from Dockerfile
    Build {
        #[arg(short, long)]
        path: String,
        #[arg(short, long)]
        tag: Option<String>,
        #[arg(long)]
        no_cache: bool,
    },
    /// Search for images
    Search {
        query: String,
        #[arg(short, long)]
        limit: Option<usize>,
        #[arg(long)]
        official: bool,
    },
    /// Clean up images
    Cleanup {
        #[arg(long)]
        force: bool,
        #[arg(long)]
        dangling: bool,
        #[arg(long)]
        untagged: bool,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Subcommand)]
enum SystemCommands {
    /// Show system information
    Info,
    /// Show version
    Version,
}

#[derive(Subcommand)]
enum RegistryCommands {
    /// List configured registries
    List,
    /// Show registry configuration
    Show,
    /// Add a new registry
    Add {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        location: String,
        #[arg(short, long)]
        mirror: Option<String>,
        #[arg(long)]
        insecure: bool,
    },
    /// Remove a registry
    Remove {
        #[arg(short, long)]
        name: String,
    },
    /// Block/unblock a registry
    Block {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        unblock: bool,
    },
    /// Initialize default configuration
    Init,
}

#[derive(Subcommand)]
enum StatsCommands {
    /// Show container statistics
    Show {
        #[arg(short, long)]
        container: Option<String>,
        #[arg(long)]
        follow: bool,
        #[arg(short, long, default_value = "5")]
        interval: u64,
    },
    /// List all container statistics
    List,
    /// Show statistics summary
    Summary,
    /// Start monitoring a container
    Start {
        #[arg(short, long)]
        container: String,
    },
    /// Stop monitoring a container
    Stop {
        #[arg(short, long)]
        container: String,
    },
}

#[derive(Subcommand)]
enum NetworkCommands {
    /// Create a network bridge
    CreateBridge {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        ip: String,
        #[arg(short, long)]
        subnet: String,
        #[arg(long, default_value = "1500")]
        mtu: u16,
    },
    /// List network bridges
    ListBridges,
    /// Create IP pool
    CreatePool {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        subnet: String,
        #[arg(short, long)]
        gateway: String,
    },
    /// List IP pools
    ListPools,
    /// Add DNS record
    AddDns {
        #[arg(short, long)]
        zone: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        record_type: String,
        #[arg(short, long)]
        value: String,
        #[arg(long, default_value = "300")]
        ttl: u32,
    },
    /// List DNS records
    ListDns {
        #[arg(short, long)]
        zone: Option<String>,
    },
    /// Add firewall rule
    AddFirewall {
        #[arg(short, long)]
        action: String,
        #[arg(short, long)]
        protocol: String,
        #[arg(long)]
        source_ip: Option<String>,
        #[arg(long)]
        source_port: Option<u16>,
        #[arg(long)]
        dest_ip: Option<String>,
        #[arg(long)]
        dest_port: Option<u16>,
        #[arg(long)]
        comment: Option<String>,
    },
    /// List firewall rules
    ListFirewall,
    /// Add port forwarding
    AddPortForward {
        #[arg(short, long)]
        host_ip: String,
        #[arg(short, long)]
        host_port: u16,
        #[arg(short, long)]
        container_ip: String,
        #[arg(short, long)]
        container_port: u16,
        #[arg(short, long)]
        protocol: String,
        #[arg(long)]
        comment: Option<String>,
    },
    /// List port forwarding rules
    ListPortForward,
}

#[derive(Subcommand)]
enum VolumeCommands {
    /// Create a volume
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long, default_value = "local")]
        driver: String,
        #[arg(long)]
        label: Vec<String>,
    },
    /// List volumes
    List,
    /// Remove a volume
    Remove {
        #[arg(short, long)]
        name: String,
        #[arg(long)]
        force: bool,
    },
    /// Mount a volume
    Mount {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        target: String,
        #[arg(long)]
        read_only: bool,
    },
    /// Unmount a volume
    Unmount {
        #[arg(short, long)]
        name: String,
    },
    /// Show volume information
    Inspect {
        #[arg(short, long)]
        name: String,
    },
    /// Prune unused volumes
    Prune {
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
enum DeployCommands {
    /// Deploy a new service
    Create {
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        image: String,
        #[arg(long, default_value = "default")]
        namespace: String,
        #[arg(short, long, default_value = "1")]
        replicas: u32,
        #[arg(short, long)]
        port: Option<u16>,
        #[arg(long)]
        health_path: Option<String>,
        #[arg(long)]
        min_replicas: Option<u32>,
        #[arg(long)]
        max_replicas: Option<u32>,
        #[arg(long)]
        target_cpu: Option<f64>,
        #[arg(long)]
        target_memory: Option<f64>,
    },
    /// List deployments
    List {
        #[arg(short, long)]
        namespace: Option<String>,
    },
    /// Get deployment status
    Status {
        #[arg(short, long)]
        name: String,
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// Scale a deployment
    Scale {
        #[arg(short, long)]
        name: String,
        #[arg(long, default_value = "default")]
        namespace: String,
        #[arg(short, long)]
        replicas: u32,
    },
    /// Delete a deployment
    Delete {
        #[arg(short, long)]
        name: String,
        #[arg(long, default_value = "default")]
        namespace: String,
    },
    /// Show orchestrator statistics
    Stats,
}

struct CliState {
    runtime: PolisRuntime,
    image_manager: ImageManager,
    stats_collector: ContainerStatsCollector,
    search_manager: ImageSearchManager,
    cleanup_manager: ImageCleanupManager,
    bridge_manager: BridgeManager,
    ipam_manager: IpamManager,
    dns_manager: DnsManager,
    firewall_manager: FirewallManager,
    port_forwarding_manager: PortForwardingManager,
    volume_manager: VolumeManager,
    orchestrator: Orchestrator,
    container_names: HashMap<String, ContainerId>,
}

impl CliState {
    async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = PolisConfig::default();
        let runtime = PolisRuntime::new(config.clone());
        runtime.initialize().await?;

        let image_cache_dir = config.storage.root_dir.join("images");
        // Use environment variable or default to no authentication
        let docker_hub_token = std::env::var("DOCKER_HUB_TOKEN").unwrap_or_default();
        let image_manager = if docker_hub_token.is_empty() {
            ImageManager::new(image_cache_dir.clone())
        } else {
            ImageManager::with_docker_hub_token(image_cache_dir.clone(), docker_hub_token)
        };

        let stats_collector = ContainerStatsCollector::default();
        let search_manager = ImageSearchManager::new(vec!["docker.io".to_string(), "quay.io".to_string()]);
        let cleanup_manager = ImageCleanupManager::new(image_cache_dir.clone())?;

        // Initialize network managers
        let bridge_manager = BridgeManager::new();
        let ipam_manager = IpamManager::new();
        let dns_manager = DnsManager::new();
        let firewall_manager = FirewallManager::new();
        let port_forwarding_manager = PortForwardingManager::new();

        // Initialize volume manager
        let volume_dir = config.storage.root_dir.join("volumes");
        let volume_manager = VolumeManager::new(volume_dir).await?;

        // Initialize orchestrator with persistent state
        let orchestrator_config = if std::path::Path::new("config/orchestrator.yaml").exists() {
            let content = std::fs::read_to_string("config/orchestrator.yaml")?;
            serde_yaml::from_str(&content).unwrap_or_else(|_| OrchestratorConfig::default())
        } else {
            OrchestratorConfig::default()
        };
        let orchestrator = Orchestrator::new(orchestrator_config).await?;

        Ok(Self {
            runtime,
            image_manager,
            stats_collector,
            search_manager,
            cleanup_manager,
            bridge_manager,
            ipam_manager,
            dns_manager,
            firewall_manager,
            port_forwarding_manager,
            volume_manager,
            orchestrator,
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
                    println!(" Baixando imagem '{}'...", name);
                    match state.image_manager.pull(&name).await {
                        Ok(image) => {
                            println!(" Imagem '{}' baixada com sucesso", name);
                            println!("  - ID: {}", image.id.0);
                            println!("  - Tamanho: {} bytes", image.size);
                            println!("  - Arquitetura: {}", image.architecture);
                            println!("  - OS: {}", image.os);
                        }
                        Err(e) => {
                            println!(" Erro ao baixar imagem: {}", e);
                        }
                    }
                }
                ImageCommands::List => {
                    println!("� Listando imagens...");
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
                                    let id_short = if image.id.0.len() >= 8 {
                                        &image.id.0[..8]
                                    } else {
                                        &image.id.0
                                    };
                                    println!(
                                        "{:<20} {:<20} {:<15} {:<10}",
                                        id_short,
                                        image.name,
                                        image.tag,
                                        format!("{} MB", image.size / 1024 / 1024)
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!(" Erro ao listar imagens: {}", e);
                        }
                    }
                }
                ImageCommands::Remove { name } => {
                    println!("  Removendo imagem '{}'...", name);
                    // TODO: Implementar remoção de imagem por nome
                    println!(" Remoção por nome não implementada ainda");
                }
                ImageCommands::Build { path, tag, no_cache } => {
                    println!("  Construindo imagem a partir de '{}'...", path);
                    
                    let build_path = std::path::PathBuf::from(&path);
                    let context = match BuildContext::new(build_path) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            println!("  Erro ao criar contexto de build: {}", e);
                            return Ok(());
                        }
                    };

                    let dockerfile_path = context.get_dockerfile()
                        .ok_or("Dockerfile não encontrado")?;
                    
                    let dockerfile = match polis_build::Dockerfile::from_file(dockerfile_path) {
                        Ok(df) => df,
                        Err(e) => {
                            println!("  Erro ao parsear Dockerfile: {}", e);
                            return Ok(());
                        }
                    };

                    let build_options = BuildOptions {
                        tag: tag.clone(),
                        no_cache: no_cache,
                        pull: false,
                        build_args: HashMap::new(),
                        target: None,
                        platform: None,
                        progress: true,
                    };

                    let build_dir = std::path::PathBuf::from("./build");
                    let mut builder = match ImageBuilder::new(build_dir) {
                        Ok(b) => b,
                        Err(e) => {
                            println!("  Erro ao criar builder: {}", e);
                            return Ok(());
                        }
                    };

                    match builder.build_image(context, dockerfile, build_options).await {
                        Ok(image_id) => {
                            println!("  Imagem construída com sucesso: {}", image_id.0);
                            if let Some(tag) = tag {
                                println!("  Tag: {}", tag);
                            }
                        }
                        Err(e) => {
                            println!("  Erro ao construir imagem: {}", e);
                        }
                    }
                }
                ImageCommands::Search { query, limit, official } => {
                    println!("  Procurando imagens por '{}'...", query);
                    
                    let search_options = SearchOptions {
                        limit,
                        official_only: official,
                        trusted_only: false,
                        automated_only: false,
                        min_stars: None,
                        registry: None,
                    };

                    match state.search_manager.search_images(&query, search_options).await {
                        Ok(results) => {
                            if results.is_empty() {
                                println!("  Nenhuma imagem encontrada");
                            } else {
                                println!("  {:<30} {:<10} {:<8} {:<15} {:<20}", "NOME", "ESTRELAS", "OFICIAL", "TAMANHO", "ATUALIZADO");
                                println!("  {}", "-".repeat(90));
                                for result in results {
                                    let size_str = if let Some(size) = result.size {
                                        format!("{:.1} MB", size as f64 / 1024.0 / 1024.0)
                                    } else {
                                        "N/A".to_string()
                                    };
                                    
                                    let updated_str = if let Some(updated) = result.last_updated {
                                        updated.format("%Y-%m-%d").to_string()
                                    } else {
                                        "N/A".to_string()
                                    };

                                    println!("  {:<30} {:<10} {:<8} {:<15} {:<20}", 
                                        result.name, 
                                        result.stars, 
                                        if result.official { "Sim" } else { "Não" },
                                        size_str,
                                        updated_str
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            println!("  Erro ao procurar imagens: {}", e);
                        }
                    }
                }
                ImageCommands::Cleanup { force, dangling, untagged, dry_run } => {
                    println!("  Limpando imagens...");
                    
                    let cleanup_options = CleanupOptions {
                        force,
                        remove_untagged: untagged,
                        remove_dangling: dangling,
                        older_than: None,
                        keep_latest: true,
                        dry_run,
                    };

                    match state.cleanup_manager.cleanup_images(cleanup_options).await {
                        Ok(stats) => {
                            if dry_run {
                                println!("  [DRY RUN] Seriam removidas:");
                            } else {
                                println!("  Limpeza concluída:");
                            }
                            println!("    - Imagens removidas: {}", stats.images_removed);
                            println!("    - Espaço liberado: {:.2} MB", stats.space_freed as f64 / 1024.0 / 1024.0);
                            println!("    - Layers removidos: {}", stats.layers_removed);
                            println!("    - Dangling removidos: {}", stats.dangling_removed);
                            println!("    - Untagged removidos: {}", stats.untagged_removed);
                        }
                        Err(e) => {
                            println!("  Erro ao limpar imagens: {}", e);
                        }
                    }
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
        Commands::Registry { action } => {
            use polis_image::RegistryConfig;
            
            match action {
                RegistryCommands::List => {
                    let config = RegistryConfig::load().unwrap_or_default();
                    println!("Configured Registries:");
                    for (name, entry) in &config.registries {
                        let status = if entry.blocked.unwrap_or(false) {
                            "BLOCKED"
                        } else {
                            "ACTIVE"
                        };
                        let mirror_info = if let Some(mirror) = &entry.mirror {
                            format!(" (mirror: {})", mirror)
                        } else {
                            String::new()
                        };
                        println!("  {}: {} - {} {}", name, entry.location, status, mirror_info);
                    }
                }
                RegistryCommands::Show => {
                    let config = RegistryConfig::load().unwrap_or_default();
                    println!("Registry Configuration:");
                    println!("Search registries: {:?}", config.unqualified_search_registries);
                    println!();
                    for (name, entry) in &config.registries {
                        println!("[registries.{}]", name);
                        println!("location = \"{}\"", entry.location);
                        if let Some(mirror) = &entry.mirror {
                            println!("mirror = \"{}\"", mirror);
                        } else {
                            println!("mirror = null");
                        }
                        println!("insecure = {}", entry.insecure.unwrap_or(false));
                        println!("blocked = {}", entry.blocked.unwrap_or(false));
                        println!();
                    }
                }
                RegistryCommands::Add { name, location, mirror, insecure } => {
                    let mut config = RegistryConfig::load().unwrap_or_default();
                    config.registries.insert(name.clone(), polis_image::RegistryEntry {
                        location,
                        mirror,
                        insecure: Some(insecure),
                        blocked: Some(false),
                    });
                    config.save_user_config()?;
                    println!("Registry '{}' added successfully", name);
                }
                RegistryCommands::Remove { name } => {
                    let mut config = RegistryConfig::load().unwrap_or_default();
                    if config.registries.remove(&name).is_some() {
                        config.save_user_config()?;
                        println!("Registry '{}' removed successfully", name);
                    } else {
                        println!("Registry '{}' not found", name);
                    }
                }
                RegistryCommands::Block { name, unblock } => {
                    let mut config = RegistryConfig::load().unwrap_or_default();
                    if let Some(entry) = config.registries.get_mut(&name) {
                        entry.blocked = Some(!unblock);
                        config.save_user_config()?;
                        let action = if unblock { "unblocked" } else { "blocked" };
                        println!("Registry '{}' {} successfully", name, action);
                    } else {
                        println!("Registry '{}' not found", name);
                    }
                }
                RegistryCommands::Init => {
                    let config = RegistryConfig::default();
                    config.save_user_config()?;
                    println!("Default registry configuration initialized");
                    println!("Configuration saved to: {:?}", RegistryConfig::user_config_path());
                }
            }
        },
        Commands::Stats { action } => {
            match action {
                StatsCommands::Show { container, follow, interval } => {
                    if let Some(container_name) = container {
                        if let Some(container_id) = state.find_container_by_name(&container_name).await {
                            state.stats_collector.start_collecting(&container_id.to_string()).await?;
                            
                            if follow {
                                println!("Monitoring container '{}' (press Ctrl+C to stop)...", container_name);
                                let mut interval_timer = tokio::time::interval(tokio::time::Duration::from_secs(interval));
                                
                                loop {
                                    interval_timer.tick().await;
                                    
                                    if let Some(metrics) = state.stats_collector.get_metrics(&container_id.to_string()).await? {
                                        print_stats_table(&metrics);
                                    }
                                }
                            } else {
                                if let Some(metrics) = state.stats_collector.get_metrics(&container_id.to_string()).await? {
                                    print_stats_table(&metrics);
                                } else {
                                    println!("No statistics available for container '{}'", container_name);
                                }
                            }
                        } else {
                            println!("Container '{}' not found", container_name);
                        }
                    } else {
                        println!("Please specify a container name");
                    }
                }
                StatsCommands::List => {
                    let all_metrics = state.stats_collector.get_all_metrics().await?;
                    if all_metrics.is_empty() {
                        println!("No containers being monitored");
                    } else {
                        println!("{:<20} {:<10} {:<15} {:<15} {:<15}", "CONTAINER", "CPU%", "MEMORY", "NET RX", "NET TX");
                        println!("{}", "-".repeat(80));
                        for metrics in all_metrics {
                            println!(
                                "{:<20} {:<10.1} {:<15} {:<15} {:<15}",
                                metrics.container_id,
                                metrics.cpu.usage_percent,
                                format_bytes(metrics.memory.usage),
                                format_bytes(metrics.network.rx_bytes),
                                format_bytes(metrics.network.tx_bytes)
                            );
                        }
                    }
                }
                StatsCommands::Summary => {
                    let summary = state.stats_collector.get_summary().await?;
                    println!("Container Statistics Summary:");
                    println!("  Total containers: {}", summary.total_containers);
                    println!("  Average CPU usage: {:.1}%", summary.avg_cpu_usage);
                    println!("  Total memory usage: {}", format_bytes(summary.total_memory_usage));
                    println!("  Total network RX: {}", format_bytes(summary.total_network_rx));
                    println!("  Total network TX: {}", format_bytes(summary.total_network_tx));
                    println!("  Total disk read: {}", format_bytes(summary.total_disk_read));
                    println!("  Total disk write: {}", format_bytes(summary.total_disk_write));
                    println!("  Total processes: {}", summary.total_processes);
                }
                StatsCommands::Start { container } => {
                    if let Some(container_id) = state.find_container_by_name(&container).await {
                        state.stats_collector.start_collecting(&container_id.to_string()).await?;
                        println!("Started monitoring container '{}'", container);
                    } else {
                        println!("Container '{}' not found", container);
                    }
                }
                StatsCommands::Stop { container } => {
                    if let Some(container_id) = state.find_container_by_name(&container).await {
                        state.stats_collector.stop_collecting(&container_id.to_string()).await?;
                        println!("Stopped monitoring container '{}'", container);
                    } else {
                        println!("Container '{}' not found", container);
                    }
                }
            }
        },
        Commands::Network { action } => {
            match action {
                NetworkCommands::CreateBridge { name, ip, subnet, mtu } => {
                    state.bridge_manager.create_bridge(&name, &ip, &subnet, mtu).await?;
                }
                NetworkCommands::ListBridges => {
                    let bridges = state.bridge_manager.list_bridges().await?;
                    if bridges.is_empty() {
                        println!("No bridges found");
                    } else {
                        println!("{:<20} {:<15} {:<20} {:<8} {:<8}", "NAME", "IP", "SUBNET", "MTU", "ENABLED");
                        println!("{}", "-".repeat(80));
                        for bridge in bridges {
                            println!(
                                "{:<20} {:<15} {:<20} {:<8} {:<8}",
                                bridge.name,
                                bridge.ip,
                                bridge.subnet,
                                bridge.mtu,
                                if bridge.enabled { "Yes" } else { "No" }
                            );
                        }
                    }
                }
                _ => {
                    println!("Network command not implemented yet");
                }
            }
        },
        Commands::Volume { action } => {
            match action {
                VolumeCommands::Create { name, driver, label } => {
                    use polis_storage::VolumeDriver;
                    let driver = match driver.to_lowercase().as_str() {
                        "local" => VolumeDriver::Local,
                        _ => {
                            println!("Only local driver supported for now");
                            return Ok(());
                        }
                    };
                    
                    let mut labels = HashMap::new();
                    for label_pair in label {
                        if let Some((key, value)) = label_pair.split_once('=') {
                            labels.insert(key.to_string(), value.to_string());
                        }
                    }
                    
                    let volume = state.volume_manager.create_volume(&name, driver, HashMap::new(), labels).await?;
                    println!("Volume '{}' created successfully", volume.name);
                }
                VolumeCommands::List => {
                    let volumes = state.volume_manager.list_volumes().await?;
                    if volumes.is_empty() {
                        println!("No volumes found");
                    } else {
                        println!("{:<20} {:<10} {:<20} {:<8}", "NAME", "DRIVER", "MOUNTPOINT", "IN USE");
                        println!("{}", "-".repeat(70));
                        for volume in volumes {
                            println!(
                                "{:<20} {:<10} {:<20} {:<8}",
                                volume.name,
                                format!("{:?}", volume.driver),
                                volume.mountpoint.to_string_lossy(),
                                if volume.in_use { "Yes" } else { "No" }
                            );
                        }
                    }
                }
                _ => {
                    println!("Volume command not implemented yet");
                }
            }
        },
        Commands::Deploy { action } => {
            match action {
                DeployCommands::Create {
                    name, image, namespace, replicas, port, health_path,
                    min_replicas, max_replicas, target_cpu, target_memory
                } => {
                    // Create port specs
                    let mut ports = Vec::new();
                    if let Some(port_num) = port {
                        ports.push(PortSpec {
                            name: "http".to_string(),
                            port: port_num,
                            target_port: port_num,
                            protocol: "TCP".to_string(),
                            expose: true,
                        });
                    }

                    // Create health check spec
                    let health_check = health_path.map(|path| HealthCheckSpec {
                        http_path: Some(path),
                        tcp_port: None,
                        command: None,
                        interval: Duration::from_secs(30),
                        timeout: Duration::from_secs(5),
                        retries: 3,
                    });

                    // Create scaling policy spec
                    let scaling_policy = if min_replicas.is_some() || max_replicas.is_some() || target_cpu.is_some() || target_memory.is_some() {
                        Some(ScalingPolicySpec {
                            min_replicas: min_replicas.unwrap_or(1),
                            max_replicas: max_replicas.unwrap_or(10),
                            target_cpu: target_cpu.unwrap_or(70.0),
                            target_memory: target_memory.unwrap_or(80.0),
                            scale_up_cooldown: Duration::from_secs(300),
                            scale_down_cooldown: Duration::from_secs(300),
                        })
                    } else {
                        None
                    };

                    let spec = DeploymentSpec {
                        name: name.clone(),
                        namespace: namespace.clone(),
                        image: image.clone(),
                        replicas,
                        ports,
                        env_vars: HashMap::new(),
                        labels: HashMap::new(),
                        annotations: HashMap::new(),
                        health_check,
                        scaling_policy,
                        resources: None,
                    };

                    let status = state.orchestrator.deploy(spec).await?;
                    println!("Deployment '{}' created successfully", status.name);
                    println!("  Namespace: {}", status.namespace);
                    println!("  Desired Replicas: {}", status.desired_replicas);
                    println!("  Status: {:?}", status.status);
                }
                DeployCommands::List { namespace } => {
                    let deployments = state.orchestrator.list_deployments(namespace.as_deref()).await?;
                    if deployments.is_empty() {
                        println!("No deployments found");
                    } else {
                        println!("{:<20} {:<15} {:<8} {:<8} {:<8} {:<12}", "NAME", "NAMESPACE", "DESIRED", "CURRENT", "READY", "STATUS");
                        println!("{}", "-".repeat(80));
                        for deployment in deployments {
                            println!(
                                "{:<20} {:<15} {:<8} {:<8} {:<8} {:<12}",
                                deployment.name,
                                deployment.namespace,
                                deployment.desired_replicas,
                                deployment.current_replicas,
                                deployment.ready_replicas,
                                format!("{:?}", deployment.status)
                            );
                        }
                    }
                }
                DeployCommands::Status { name, namespace } => {
                    if let Some(status) = state.orchestrator.get_deployment_status(&name, &namespace).await? {
                        println!("Deployment: {}", status.name);
                        println!("  Namespace: {}", status.namespace);
                        println!("  Desired Replicas: {}", status.desired_replicas);
                        println!("  Current Replicas: {}", status.current_replicas);
                        println!("  Ready Replicas: {}", status.ready_replicas);
                        println!("  Available Replicas: {}", status.available_replicas);
                        println!("  Status: {:?}", status.status);
                        println!("  Created: {}", status.created_at);
                        println!("  Updated: {}", status.updated_at);
                    } else {
                        println!("Deployment '{}' not found in namespace '{}'", name, namespace);
                    }
                }
                DeployCommands::Scale { name, namespace, replicas } => {
                    state.orchestrator.scale_deployment(&name, &namespace, replicas).await?;
                    println!("Deployment '{}' scaled to {} replicas", name, replicas);
                }
                DeployCommands::Delete { name, namespace } => {
                    state.orchestrator.delete_deployment(&name, &namespace).await?;
                    println!("Deployment '{}' deleted successfully", name);
                }
                DeployCommands::Stats => {
                    let stats = state.orchestrator.get_stats().await?;
                    println!("Orchestrator Statistics:");
                    println!("  Total Deployments: {}", stats.total_deployments);
                    println!("  Running Deployments: {}", stats.running_deployments);
                    println!("  Failed Deployments: {}", stats.failed_deployments);
                    println!("  Total Services: {}", stats.total_services);
                    println!("  Total Health Checks: {}", stats.total_health_checks);
                    println!("  Total Replicas: {}", stats.total_replicas);
                    println!("  Auto Scaling: {}", if stats.auto_scaling_enabled { "Enabled" } else { "Disabled" });
                }
            }
        },
    }

    Ok(())
}

/// Print a detailed stats table for a container
fn print_stats_table(metrics: &polis_stats::ContainerMetrics) {
    println!("\n=== Container Statistics: {} ===", metrics.container_id);
    println!("Timestamp: {:?}", metrics.timestamp);
    
    println!("\n--- CPU ---");
    println!("  Usage: {:.1}%", metrics.cpu.usage_percent);
    println!("  Cores: {}", metrics.cpu.cores);
    println!("  User time: {} ns", metrics.cpu.user_time);
    println!("  System time: {} ns", metrics.cpu.system_time);
    println!("  Throttled: {} times", metrics.cpu.throttled_count);
    
    println!("\n--- Memory ---");
    println!("  Usage: {} ({:.1}%)", format_bytes(metrics.memory.usage), metrics.memory.usage_percent);
    println!("  Limit: {}", format_bytes(metrics.memory.limit));
    println!("  Peak: {}", format_bytes(metrics.memory.peak_usage));
    println!("  RSS: {}", format_bytes(metrics.memory.rss));
    println!("  Cache: {}", format_bytes(metrics.memory.cache));
    println!("  Swap: {}", format_bytes(metrics.memory.swap));
    println!("  OOM kills: {}", metrics.memory.oom_kills);
    
    println!("\n--- Network ---");
    println!("  RX: {} ({} packets)", format_bytes(metrics.network.rx_bytes), metrics.network.rx_packets);
    println!("  TX: {} ({} packets)", format_bytes(metrics.network.tx_bytes), metrics.network.tx_packets);
    println!("  RX errors: {}", metrics.network.rx_errors);
    println!("  TX errors: {}", metrics.network.tx_errors);
    println!("  RX dropped: {}", metrics.network.rx_dropped);
    println!("  TX dropped: {}", metrics.network.tx_dropped);
    
    println!("\n--- Disk ---");
    println!("  Read: {} ({} ops)", format_bytes(metrics.disk.read_bytes), metrics.disk.read_ops);
    println!("  Write: {} ({} ops)", format_bytes(metrics.disk.write_bytes), metrics.disk.write_ops);
    println!("  Read time: {} ns", metrics.disk.read_time);
    println!("  Write time: {} ns", metrics.disk.write_time);
    
    println!("\n--- Processes ---");
    println!("  Count: {}", metrics.processes.process_count);
    println!("  Threads: {}", metrics.processes.thread_count);
    println!("  File descriptors: {}", metrics.processes.fd_count);
    println!("  Open files: {}", metrics.processes.open_files);
    println!("  State: {}", metrics.processes.state);
    println!();
}

/// Format bytes into human readable format
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    
    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}
