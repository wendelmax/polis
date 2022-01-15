use polis_core::ResourceLimits;
use polis_security::{
    Capability, CapabilityManager, CgroupManager, NamespaceManager, NamespaceType, SeccompManager,
};
use std::collections::HashSet;
use std::path::PathBuf;

#[tokio::test]
async fn test_namespace_manager_creation() {
    let mut namespace_manager = NamespaceManager::new();

    // Test creating individual namespaces
    let pid_namespace = namespace_manager
        .create_namespace(NamespaceType::PID)
        .await
        .unwrap();
    assert_eq!(pid_namespace.namespace_type, NamespaceType::PID);
    assert!(pid_namespace.path.contains("pid"));

    let network_namespace = namespace_manager
        .create_namespace(NamespaceType::Network)
        .await
        .unwrap();
    assert_eq!(network_namespace.namespace_type, NamespaceType::Network);
    assert!(network_namespace.path.contains("net"));

    // Test listing namespaces
    let namespaces = namespace_manager.list_namespaces().await.unwrap();
    assert_eq!(namespaces.len(), 2);
}

#[tokio::test]
async fn test_container_namespaces_creation() {
    let mut namespace_manager = NamespaceManager::new();

    // Test creating essential namespaces for containers
    let namespaces = namespace_manager
        .create_container_namespaces()
        .await
        .unwrap();

    // Should create at least 5 essential namespaces
    assert!(namespaces.len() >= 5);

    // Check that essential namespaces are present
    let namespace_types: HashSet<_> = namespaces.iter().map(|ns| &ns.namespace_type).collect();

    assert!(namespace_types.contains(&NamespaceType::PID));
    assert!(namespace_types.contains(&NamespaceType::Network));
    assert!(namespace_types.contains(&NamespaceType::Mount));
    assert!(namespace_types.contains(&NamespaceType::UTS));
    assert!(namespace_types.contains(&NamespaceType::IPC));
}

#[tokio::test]
async fn test_cgroup_manager() {
    let cgroup_path = PathBuf::from("/tmp/polis-test-cgroups");
    let mut cgroup_manager = CgroupManager::new(cgroup_path);

    // Test creating cgroup with resource limits
    let limits = ResourceLimits {
        memory_limit: Some(512 * 1024 * 1024), // 512MB
        memory_swap: Some(1024 * 1024 * 1024), // 1GB
        cpu_quota: Some(0.5),                  // 50% CPU
        cpu_period: Some(100000),              // 100ms
        pids_limit: Some(100),
        disk_quota: Some(5 * 1024 * 1024 * 1024), // 5GB
    };

    let cgroup_info = cgroup_manager
        .create_cgroup("test-container", limits)
        .await
        .unwrap();
    assert_eq!(cgroup_info.name, "test-container");
    assert_eq!(cgroup_info.limits.memory_limit, Some(512 * 1024 * 1024));

    // Test adding process to cgroup
    assert!(cgroup_manager
        .add_process("test-container", 1234)
        .await
        .is_ok());

    // Test removing process from cgroup
    assert!(cgroup_manager
        .remove_process("test-container", 1234)
        .await
        .is_ok());

    // Test getting cgroup stats
    let stats = cgroup_manager
        .get_cgroup_stats("test-container")
        .await
        .unwrap();
    assert_eq!(stats.memory_usage, 0);
    assert_eq!(stats.cpu_usage, 0);
    assert_eq!(stats.process_count, 0);

    // Test listing cgroups
    let cgroups = cgroup_manager.list_cgroups().await.unwrap();
    assert_eq!(cgroups.len(), 1);
    assert_eq!(cgroups[0].name, "test-container");

    // Test deleting cgroup
    assert!(cgroup_manager.delete_cgroup("test-container").await.is_ok());

    // Verify cgroup was deleted
    let cgroups = cgroup_manager.list_cgroups().await.unwrap();
    assert_eq!(cgroups.len(), 0);
}

#[tokio::test]
async fn test_seccomp_manager() {
    let mut seccomp_manager = SeccompManager::new();

    // Test creating default profile
    assert!(seccomp_manager.create_default_profile().await.is_ok());

    // Test listing profiles
    let profiles = seccomp_manager.list_profiles().await.unwrap();
    assert_eq!(profiles.len(), 1);
    assert!(profiles.contains(&"default".to_string()));

    // Test applying profile
    assert!(seccomp_manager.apply_profile("default").await.is_ok());

    // Test applying non-existent profile
    let result = seccomp_manager.apply_profile("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_capability_manager() {
    let mut capability_manager = CapabilityManager::new();

    // Test creating minimal capability set
    assert!(capability_manager.create_minimal_capset().await.is_ok());

    // Test listing capabilities
    let capabilities = capability_manager.list_capabilities().await.unwrap();
    assert!(!capabilities.is_empty());

    // Test adding capabilities
    let additional_caps = vec![Capability::NetAdmin, Capability::SysAdmin];
    assert!(capability_manager
        .add_capabilities(additional_caps)
        .await
        .is_ok());

    // Test removing capabilities
    let caps_to_remove = vec![Capability::SysAdmin];
    assert!(capability_manager
        .drop_capabilities(caps_to_remove)
        .await
        .is_ok());

    // Test getting current capabilities
    let current_caps = capability_manager.get_current_capabilities().await.unwrap();
    assert!(!current_caps.effective.is_empty());
    assert!(!current_caps.permitted.is_empty());
}

#[tokio::test]
async fn test_capability_sets() {
    let mut capability_manager = CapabilityManager::new();

    // Test minimal capability set
    capability_manager.create_minimal_capset().await.unwrap();
    let minimal_caps = capability_manager.get_current_capabilities().await.unwrap();
    assert!(!minimal_caps.effective.is_empty());
    assert!(minimal_caps.effective.len() < 50); // Should be minimal

    // Test privileged capability set
    capability_manager.create_privileged_capset().await.unwrap();
    let privileged_caps = capability_manager.get_current_capabilities().await.unwrap();
    assert!(privileged_caps.effective.len() > minimal_caps.effective.len());
}

#[tokio::test]
async fn test_security_integration() {
    // Test integration between different security components
    let mut namespace_manager = NamespaceManager::new();
    let cgroup_path = PathBuf::from("/tmp/polis-test-integration");
    let mut cgroup_manager = CgroupManager::new(cgroup_path);
    let mut seccomp_manager = SeccompManager::new();
    let mut capability_manager = CapabilityManager::new();

    // Create namespaces
    let namespaces = namespace_manager
        .create_container_namespaces()
        .await
        .unwrap();
    assert!(namespaces.len() >= 5);

    // Create cgroup
    let limits = ResourceLimits::default();
    let cgroup_info = cgroup_manager
        .create_cgroup("integration-test", limits)
        .await
        .unwrap();
    assert_eq!(cgroup_info.name, "integration-test");

    // Create seccomp profile
    seccomp_manager.create_default_profile().await.unwrap();
    let profiles = seccomp_manager.list_profiles().await.unwrap();
    assert_eq!(profiles.len(), 1);

    // Create capability set
    capability_manager.create_minimal_capset().await.unwrap();
    let capabilities = capability_manager.list_capabilities().await.unwrap();
    assert!(!capabilities.is_empty());

    // Test hostname setup
    assert!(namespace_manager
        .setup_hostname("integration-test")
        .await
        .is_ok());
}

#[tokio::test]
async fn test_error_handling() {
    let namespace_manager = NamespaceManager::new();
    let cgroup_path = PathBuf::from("/tmp/polis-test-errors");
    let cgroup_manager = CgroupManager::new(cgroup_path);
    let seccomp_manager = SeccompManager::new();
    let capability_manager = CapabilityManager::new();

    // Test entering non-existent namespace
    let result = namespace_manager
        .enter_namespace("/nonexistent/namespace")
        .await;
    assert!(result.is_err());

    // Test getting stats for non-existent cgroup
    let result = cgroup_manager.get_cgroup_stats("nonexistent").await;
    assert!(result.is_err());

    // Test applying non-existent seccomp profile
    let result = seccomp_manager.apply_profile("nonexistent").await;
    assert!(result.is_err());

    // Test getting capabilities before setting any
    let capabilities = capability_manager.list_capabilities().await.unwrap();
    assert_eq!(capabilities.len(), 0);
}
