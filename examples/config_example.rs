use polis_core::{PolisConfig, Logger, LogLevel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Criar configuração padrão
    let mut config = PolisConfig::default();
    
    // Validar configuração
    config.validate()?;
    
    // Salvar configuração em diferentes formatos
    config.save_to_file("config.yaml")?;
    config.save_to_file("config.toml")?;
    config.save_to_file("config.json")?;
    
    // Carregar configuração de arquivo
    let loaded_config = PolisConfig::load_from_file("config.yaml")?;
    
    // Inicializar logger
    let logger = Logger::new(LogLevel::Info, None);
    logger.init()?;
    
    // Usar logger
    polis_core::log_container_created("123e4567-e89b-12d3-a456-426614174000", "meu-container");
    polis_core::log_image_pulled("alpine", "latest");
    
    println!("Configuração carregada com sucesso!");
    println!("Runtime root: {:?}", loaded_config.runtime.root_dir);
    println!("API REST port: {}", loaded_config.api.rest_port);
    println!("API gRPC port: {}", loaded_config.api.grpc_port);
    
    Ok(())
}

