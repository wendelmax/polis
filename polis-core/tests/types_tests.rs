use chrono::Utc;
use polis_core::{
    Container, ContainerId, ContainerStatus, Image, ImageConfig, ImageId, NetworkMode, PortMapping,
    Protocol, ResourceLimits, VolumeMode, VolumeMount,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_container_id() {
    let id1 = ContainerId::new();
    let id2 = ContainerId::new();

    // IDs should be different
    assert_ne!(id1.0, id2.0);

    // Test string conversion
    let id_string = id1.0.to_string();
    let parsed_id = ContainerId::from_string(&id_string).unwrap();
    assert_eq!(id1.0, parsed_id.0);

    // Test invalid UUID
    assert!(ContainerId::from_string("invalid-uuid").is_err());
}

#[test]
fn test_image_id() {
    let id1 = ImageId::from_string("alpine:latest");
    let id2 = ImageId::from_string("ubuntu:20.04");

    assert_eq!(id1.0, "alpine:latest");
    assert_eq!(id2.0, "ubuntu:20.04");
}

#[test]
fn test_container_status() {
    let status = ContainerStatus::Created;
    assert_eq!(format!("{:?}", status), "Created");

    let status = ContainerStatus::Running;
    assert_eq!(format!("{:?}", status), "Running");

    let status = ContainerStatus::Stopped;
    assert_eq!(format!("{:?}", status), "Stopped");
}

#[test]
fn test_container() {
    let container = Container {
        id: ContainerId::new(),
        name: "test-container".to_string(),
        image: ImageId::from_string("alpine:latest"),
        status: ContainerStatus::Created,
        created_at: Utc::now(),
        started_at: None,
        finished_at: None,
        exit_code: None,
        command: vec!["sh".to_string()],
        working_dir: PathBuf::from("/"),
        environment: HashMap::new(),
        labels: HashMap::new(),
        resource_limits: ResourceLimits::default(),
        network_mode: NetworkMode::default(),
        ports: Vec::new(),
        volumes: Vec::new(),
    };

    assert_eq!(container.name, "test-container");
    assert_eq!(container.image.0, "alpine:latest");
    assert_eq!(container.status, ContainerStatus::Created);
    assert!(container.started_at.is_none());
    assert!(container.finished_at.is_none());
    assert!(container.exit_code.is_none());
}

#[test]
fn test_image() {
    let image = Image {
        id: ImageId::from_string("alpine:latest"),
        name: "alpine".to_string(),
        tag: "latest".to_string(),
        digest: "sha256:abc123".to_string(),
        size: 1024 * 1024, // 1MB
        created_at: Utc::now(),
        architecture: "amd64".to_string(),
        os: "linux".to_string(),
        layers: vec!["layer1".to_string(), "layer2".to_string()],
        config: ImageConfig {
            entrypoint: Some(vec!["/bin/sh".to_string()]),
            cmd: Some(vec!["-c".to_string()]),
            env: Some(vec![
                "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
            ]),
            working_dir: Some("/".to_string()),
            exposed_ports: Some(HashMap::new()),
            volumes: Some(HashMap::new()),
            labels: Some(HashMap::new()),
        },
    };

    assert_eq!(image.name, "alpine");
    assert_eq!(image.tag, "latest");
    assert_eq!(image.digest, "sha256:abc123");
    assert_eq!(image.size, 1024 * 1024);
    assert_eq!(image.architecture, "amd64");
    assert_eq!(image.os, "linux");
    assert_eq!(image.layers.len(), 2);
}

#[test]
fn test_resource_limits() {
    let limits = ResourceLimits {
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        memory_swap: Some(1024 * 1024 * 1024), // 1GB
        cpu_quota: Some(0.5),                  // 50% CPU
        cpu_period: Some(100000),              // 100ms
        pids_limit: Some(100),
        disk_quota: Some(5 * 1024 * 1024 * 1024), // 5GB
    };

    assert_eq!(limits.memory_limit, Some(512 * 1024 * 1024));
    assert_eq!(limits.memory_swap, Some(1024 * 1024 * 1024));
    assert_eq!(limits.cpu_quota, Some(0.5));
    assert_eq!(limits.cpu_period, Some(100000));
    assert_eq!(limits.pids_limit, Some(100));
    assert_eq!(limits.disk_quota, Some(5 * 1024 * 1024 * 1024));
}

#[test]
fn test_network_mode() {
    let bridge_mode = NetworkMode::Bridge;
    assert_eq!(format!("{:?}", bridge_mode), "Bridge");

    let host_mode = NetworkMode::Host;
    assert_eq!(format!("{:?}", host_mode), "Host");

    let none_mode = NetworkMode::None;
    assert_eq!(format!("{:?}", none_mode), "None");
}

#[test]
fn test_port_mapping() {
    let port_mapping = PortMapping {
        host_port: 8080,
        container_port: 80,
        protocol: Protocol::Tcp,
        host_ip: None,
    };

    assert_eq!(port_mapping.host_port, 8080);
    assert_eq!(port_mapping.container_port, 80);
    assert_eq!(port_mapping.protocol, Protocol::Tcp);
}

#[test]
fn test_volume_mount() {
    let volume_mount = VolumeMount {
        source: "/host/path".to_string(),
        destination: PathBuf::from("/container/path"),
        mode: VolumeMode::Bind,
        read_only: false,
    };

    assert_eq!(volume_mount.source, "/host/path");
    assert_eq!(volume_mount.destination, PathBuf::from("/container/path"));
    assert_eq!(volume_mount.mode, VolumeMode::Bind);
    assert!(!volume_mount.read_only);
}

#[test]
fn test_serialization() {
    let container = Container {
        id: ContainerId::new(),
        name: "test-serialization".to_string(),
        image: ImageId::from_string("alpine:latest"),
        status: ContainerStatus::Created,
        created_at: Utc::now(),
        started_at: None,
        finished_at: None,
        exit_code: None,
        command: vec!["sh".to_string()],
        working_dir: PathBuf::from("/"),
        environment: HashMap::new(),
        labels: HashMap::new(),
        resource_limits: ResourceLimits::default(),
        network_mode: NetworkMode::default(),
        ports: Vec::new(),
        volumes: Vec::new(),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&container).unwrap();
    let parsed_container: Container = serde_json::from_str(&json).unwrap();
    assert_eq!(container.name, parsed_container.name);
    assert_eq!(container.image.0, parsed_container.image.0);

    // Test YAML serialization
    let yaml = serde_yaml::to_string(&container).unwrap();
    let parsed_container: Container = serde_yaml::from_str(&yaml).unwrap();
    assert_eq!(container.name, parsed_container.name);
    assert_eq!(container.image.0, parsed_container.image.0);
}
