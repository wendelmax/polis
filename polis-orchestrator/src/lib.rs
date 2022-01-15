pub mod auto_scaling;
pub mod health_monitor;
pub mod load_balancer;
pub mod orchestrator;
pub mod scheduler;
pub mod service_discovery;

pub use auto_scaling::{
    AutoScaler, Deployment, MetricsCollector, ScalingAction, ScalingActionType, ScalingEngine,
    ScalingEvent, ScalingPolicy,
};
pub use health_monitor::{
    CheckType, CommandExecutor, HealthCheck as HealthCheckDef, HealthCheckResult, HealthEvent,
    HealthMonitor, HealthStatus, TargetType,
};
pub use load_balancer::{
    ConsistentHashRing, EndpointStats, LoadBalancer, LoadBalancerRequest, LoadBalancerResponse,
    LoadBalancerStats,
};
pub use orchestrator::*;
pub use scheduler::*;
pub use service_discovery::{
    DnsRecord, DnsResolver, HealthCheck, HealthChecker, LoadBalancerConfig, LoadBalancingAlgorithm,
    Protocol, Service, ServiceDiscovery, ServiceEndpoint, ServiceEvent, ServiceStatus,
};
