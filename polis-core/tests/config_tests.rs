use polis_core::{
    ApiConfig, LogLevel, NetworkConfig, PolisConfig, RuntimeConfig, SecurityConfig, StorageConfig,
};

#[test]
fn test_default_config() {
    let config = PolisConfig::default();

    // Test runtime config
    assert_eq!(config.runtime.max_containers, 100);
    assert_eq!(config.runtime.container_timeout, 30);
    assert_eq!(config.runtime.log_level, LogLevel::Info);

    // Test storage config
    assert!(config.storage.root_dir.to_string_lossy().contains("polis"));

    // Test network config
    assert_eq!(config.network.bridge_name, "polis0");
    assert_eq!(config.network.subnet, Some("172.17.0.0/16".to_string()));

    // Test security config
    assert_eq!(config.security.drop_capabilities.len(), 1);
    assert!(!config.security.read_only_rootfs);

    // Test API config
    assert_eq!(config.api.rest_port, 8080);
    assert_eq!(config.api.grpc_port, 9090);
}

#[test]
fn test_config_validation() {
    let mut config = PolisConfig::default();

    // Valid config should pass
    assert!(config.validate().is_ok());

    // Invalid max_containers should fail
    config.runtime.max_containers = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid timeout
    config.runtime.max_containers = 100;
    config.runtime.container_timeout = 0;
    assert!(config.validate().is_err());

    // Reset and test invalid ports
    config.runtime.container_timeout = 30;
    config.api.rest_port = 0;
    assert!(config.validate().is_err());

    // Reset and test same ports
    config.api.rest_port = 8080;
    config.api.grpc_port = 8080;
    assert!(config.validate().is_err());
}

#[test]
fn test_config_serialization() {
    let config = PolisConfig::default();

    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let parsed_config: PolisConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(
        config.runtime.max_containers,
        parsed_config.runtime.max_containers
    );

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&config).unwrap();
    let parsed_config: PolisConfig = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(
        config.runtime.max_containers,
        parsed_config.runtime.max_containers
    );

    // Test TOML serialization
    let toml = toml::to_string(&config).unwrap();
    let parsed_config: PolisConfig = toml::from_str(&toml).unwrap();
    assert_eq!(
        config.runtime.max_containers,
        parsed_config.runtime.max_containers
    );
}

#[test]
fn test_runtime_config() {
    let runtime_config = RuntimeConfig::default();

    assert_eq!(runtime_config.max_containers, 100);
    assert_eq!(runtime_config.container_timeout, 30);
    assert_eq!(runtime_config.log_level, LogLevel::Info);
    assert!(runtime_config.root_dir.to_string_lossy().contains("polis"));
}

#[test]
fn test_storage_config() {
    let storage_config = StorageConfig::default();

    assert!(storage_config.root_dir.to_string_lossy().contains("polis"));
    assert_eq!(storage_config.max_size, None);
}

#[test]
fn test_network_config() {
    let network_config = NetworkConfig::default();

    assert_eq!(network_config.bridge_name, "polis0");
    assert_eq!(network_config.subnet, Some("172.17.0.0/16".to_string()));
    assert_eq!(network_config.gateway, Some("172.17.0.1".to_string()));
}

#[test]
fn test_security_config() {
    let security_config = SecurityConfig::default();

    assert_eq!(security_config.drop_capabilities.len(), 1);
    assert!(!security_config.read_only_rootfs);
}

#[test]
fn test_api_config() {
    let api_config = ApiConfig::default();

    assert_eq!(api_config.rest_port, 8080);
    assert_eq!(api_config.grpc_port, 9090);
    assert_eq!(api_config.host, "0.0.0.0");
    assert!(api_config.enable_cors);
}

#[test]
fn test_log_level() {
    assert_eq!(LogLevel::Error as u8, 0);
    assert_eq!(LogLevel::Warn as u8, 1);
    assert_eq!(LogLevel::Info as u8, 2);
    assert_eq!(LogLevel::Debug as u8, 3);
    assert_eq!(LogLevel::Trace as u8, 4);
}
