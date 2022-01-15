use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use polis_core::types::ContainerId;
use polis_security::{AppArmorManager, SELinuxManager, SecurityManager};
use tokio::runtime::Runtime;

fn security_manager_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mut security_manager = SecurityManager::new();

        let mut group = c.benchmark_group("security_manager");

        // Benchmark container profile creation
        group.bench_function("create_container_profile", |b| {
            b.to_async(&rt).iter(|| async {
                let container_id = ContainerId::new();
                security_manager
                    .create_container_profile(&container_id)
                    .await
            });
        });

        // Benchmark high security profile creation
        group.bench_function("create_high_security_profile", |b| {
            b.to_async(&rt).iter(|| async {
                let container_id = ContainerId::new();
                security_manager
                    .create_high_security_profile(&container_id)
                    .await
            });
        });

        // Benchmark privileged profile creation
        group.bench_function("create_privileged_profile", |b| {
            b.to_async(&rt).iter(|| async {
                let container_id = ContainerId::new();
                security_manager
                    .create_privileged_profile(&container_id)
                    .await
            });
        });

        // Benchmark profile retrieval
        group.bench_function("get_container_profile", |b| {
            b.to_async(&rt).iter(|| async {
                let container_id = ContainerId::new();
                let _ = security_manager
                    .create_container_profile(&container_id)
                    .await;
                security_manager.get_container_profile(&container_id).await
            });
        });

        group.finish();
    });
}

fn apparmor_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let apparmor_manager = AppArmorManager::new();

        let mut group = c.benchmark_group("apparmor");

        // Benchmark profile creation
        group.bench_function("create_profile", |b| {
            b.to_async(&rt)
                .iter(|| async { apparmor_manager.create_profile("test-profile").await });
        });

        // Benchmark container profile creation
        group.bench_function("create_container_profile", |b| {
            b.to_async(&rt).iter(|| async {
                apparmor_manager
                    .create_container_profile("test-container")
                    .await
            });
        });

        // Benchmark availability check
        group.bench_function("check_availability", |b| {
            b.to_async(&rt)
                .iter(|| async { apparmor_manager.is_available().await });
        });

        group.finish();
    });
}

fn selinux_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let selinux_manager = SELinuxManager::new();

        let mut group = c.benchmark_group("selinux");

        // Benchmark policy creation
        group.bench_function("create_policy", |b| {
            b.to_async(&rt)
                .iter(|| async { selinux_manager.create_policy("test-policy").await });
        });

        // Benchmark container policy creation
        group.bench_function("create_container_policy", |b| {
            b.to_async(&rt).iter(|| async {
                selinux_manager
                    .create_container_policy("test-container")
                    .await
            });
        });

        // Benchmark context parsing
        group.bench_function("parse_context", |b| {
            b.iter(|| {
                let context_str = "user_u:system_r:container_t:s0";
                selinux_manager.parse_context(context_str)
            });
        });

        // Benchmark availability check
        group.bench_function("check_availability", |b| {
            b.to_async(&rt)
                .iter(|| async { selinux_manager.is_available().await });
        });

        group.finish();
    });
}

fn concurrent_security_operations_benchmark(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        let mut security_manager = SecurityManager::new();

        let mut group = c.benchmark_group("concurrent_security_operations");

        group.bench_function("concurrent_profile_creation", |b| {
            b.to_async(&rt).iter(|| async {
                let mut handles = Vec::new();

                for i in 0..10 {
                    let mut security_manager = security_manager.clone();
                    let handle = tokio::spawn(async move {
                        let container_id = ContainerId::new();
                        security_manager
                            .create_container_profile(&container_id)
                            .await
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap().unwrap();
                }
            });
        });

        group.finish();
    });
}

fn security_profile_serialization_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_profile_serialization");

    // Benchmark AppArmor profile serialization
    group.bench_function("serialize_apparmor_profile", |b| {
        b.iter(|| {
            let profile = polis_security::AppArmorProfile {
                name: "test-profile".to_string(),
                rules: vec![
                    "capability net_admin,".to_string(),
                    "capability sys_admin,".to_string(),
                    "network,".to_string(),
                ],
                container_id: Some("test-container".to_string()),
            };

            serde_json::to_string(&profile)
        });
    });

    // Benchmark SELinux policy serialization
    group.bench_function("serialize_selinux_policy", |b| {
        b.iter(|| {
            let policy = polis_security::SELinuxPolicy {
                name: "test-policy".to_string(),
                rules: vec![
                    "allow container_t self:capability { setuid setgid };".to_string(),
                    "allow container_t self:process { transition };".to_string(),
                ],
                container_id: Some("test-container".to_string()),
            };

            serde_json::to_string(&policy)
        });
    });

    // Benchmark container security profile serialization
    group.bench_function("serialize_container_security_profile", |b| {
        b.iter(|| {
            let profile = polis_security::ContainerSecurityProfile {
                container_id: ContainerId::new(),
                namespaces: vec!["pid".to_string(), "net".to_string(), "mount".to_string()],
                cgroup_limits: Some(polis_core::types::ResourceLimits {
                    memory_limit: Some(512 * 1024 * 1024),
                    memory_swap: Some(512 * 1024 * 1024),
                    cpu_quota: Some(50000.0),
                    cpu_period: Some(100000),
                    pids_limit: Some(100),
                    disk_quota: Some(1024 * 1024 * 1024),
                }),
                seccomp_profile: Some("default".to_string()),
                capabilities: vec!["NET_ADMIN".to_string(), "SYS_ADMIN".to_string()],
                apparmor_profile: Some("container-profile".to_string()),
                selinux_context: Some(polis_security::SELinuxContext {
                    user: "user_u".to_string(),
                    role: "system_r".to_string(),
                    r#type: "container_t".to_string(),
                    level: "s0".to_string(),
                }),
                sandbox_config: Some(polis_security::SandboxConfig {
                    read_only_rootfs: true,
                    no_new_privileges: true,
                    drop_capabilities: vec!["ALL".to_string()],
                }),
            };

            serde_json::to_string(&profile)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    security_manager_benchmark,
    apparmor_benchmark,
    selinux_benchmark,
    concurrent_security_operations_benchmark,
    security_profile_serialization_benchmark
);
criterion_main!(benches);
