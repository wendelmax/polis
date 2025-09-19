use polis_monitor::{
    AlertManager, AlertSeverity, DashboardManager, ExportConfig, ExportData, ExportFormat,
    HealthChecker, LogLevel, LogManager, MetricsCollector, MetricsExporter, NotificationType,
    WidgetType,
};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Exemplo do Sistema de Monitoramento do Polis");
    println!("==============================================\n");

    // 1. Coleta de Métricas
    println!("1.  Coletando Métricas do Sistema...");
    let mut metrics_collector = MetricsCollector::new(60);

    // Coletar métricas do sistema
    let _system_metrics = metrics_collector.collect_system_metrics().await?;
    println!("    Métricas do sistema coletadas");

    // Coletar métricas de containers
    metrics_collector
        .collect_container_metrics("container-1")
        .await?;
    metrics_collector
        .collect_container_metrics("container-2")
        .await?;
    metrics_collector
        .collect_container_metrics("container-3")
        .await?;

    // Obter resumo das métricas
    let metrics_summary = metrics_collector.get_metrics_summary().await?;
    println!(
        "    Resumo: {} containers totais, {} rodando",
        metrics_summary.total_containers, metrics_summary.running_containers
    );

    // 2. Health Checks
    println!("\n2.  Executando Health Checks...");
    let health_checker = HealthChecker::new();

    // Health check do sistema
    let system_health = health_checker.check_system_health().await?;
    println!(
        "    Status geral do sistema: {:?}",
        system_health.overall_status
    );

    // Health check de containers
    health_checker.check_container_health("container-1").await?;
    health_checker.check_container_health("container-2").await?;

    // Resumo de health
    let health_summary = health_checker.get_system_health_summary().await?;
    println!(
        "    Health Summary: {} checks saudáveis, {} degradados, {} não saudáveis",
        health_summary.healthy_checks,
        health_summary.degraded_checks,
        health_summary.unhealthy_checks
    );

    // 3. Sistema de Alertas
    println!("\n3. � Configurando Sistema de Alertas...");
    let mut alert_manager = AlertManager::new();

    // Criar regras de alerta
    alert_manager.create_alert_rule(
        "High CPU Usage",
        "Alerta quando uso de CPU excede 80%",
        "cpu_usage > 80",
        AlertSeverity::High,
    )?;

    alert_manager.create_alert_rule(
        "High Memory Usage",
        "Alerta quando uso de memória excede 90%",
        "memory_usage > 90",
        AlertSeverity::Critical,
    )?;

    alert_manager.create_alert_rule(
        "Too Many Containers",
        "Alerta quando número de containers excede 100",
        "container_count > 100",
        AlertSeverity::Medium,
    )?;

    // Criar canais de notificação
    let mut email_config = HashMap::new();
    email_config.insert("email".to_string(), "admin@polis.local".to_string());
    alert_manager.create_notification_channel(
        "email",
        "Email Notifications",
        NotificationType::Email,
        email_config,
    )?;

    let mut slack_config = HashMap::new();
    slack_config.insert(
        "webhook".to_string(),
        "https://hooks.slack.com/...".to_string(),
    );
    alert_manager.create_notification_channel(
        "slack",
        "Slack Notifications",
        NotificationType::Slack,
        slack_config,
    )?;

    // Simular métricas e avaliar regras
    let mut test_metrics = HashMap::new();
    test_metrics.insert("cpu_usage".to_string(), 85.0);
    test_metrics.insert("memory_usage".to_string(), 45.0);
    test_metrics.insert("container_count".to_string(), 5.0);

    let triggered_alerts = alert_manager.evaluate_rules(&test_metrics).await?;
    println!("   � {} alertas disparados", triggered_alerts.len());

    // Resumo de alertas
    let alert_summary = alert_manager.get_alert_summary().await?;
    println!(
        "    Alertas: {} ativos, {} críticos, {} altos",
        alert_summary.active_alerts, alert_summary.critical_alerts, alert_summary.high_alerts
    );

    // 4. Dashboard
    println!("\n4.  Configurando Dashboard...");
    let mut dashboard_manager = DashboardManager::new();

    // Criar dashboard personalizado
    let custom_dashboard_id = dashboard_manager.create_dashboard(
        "Dashboard Personalizado",
        "Dashboard customizado para monitoramento específico",
    )?;

    // Adicionar widgets
    dashboard_manager.add_widget(
        &custom_dashboard_id,
        "CPU Usage Widget",
        WidgetType::Metric,
        "cpu_usage",
    )?;

    dashboard_manager.add_widget(
        &custom_dashboard_id,
        "Memory Chart",
        WidgetType::Chart,
        "memory_usage",
    )?;

    dashboard_manager.add_widget(
        &custom_dashboard_id,
        "Container Status",
        WidgetType::Status,
        "containers",
    )?;

    // Obter dados do dashboard
    let dashboard_data = dashboard_manager
        .get_dashboard_data(&custom_dashboard_id)
        .await?;
    println!(
        "    Dashboard '{}' com {} widgets",
        custom_dashboard_id,
        dashboard_data.widgets_data.len()
    );

    // Resumo de dashboards
    let dashboard_summary = dashboard_manager.get_dashboard_summary().await?;
    println!(
        "    Total: {} dashboards, {} widgets",
        dashboard_summary.total_dashboards, dashboard_summary.total_widgets
    );

    // 5. Sistema de Logs
    println!("\n5.  Configurando Sistema de Logs...");
    let mut log_manager = LogManager::new(1000);

    // Adicionar logs
    log_manager
        .add_log(
            LogLevel::Info,
            "Sistema de monitoramento iniciado",
            "monitor",
            None,
        )
        .await?;

    log_manager
        .add_log(
            LogLevel::Warn,
            "Uso de CPU elevado detectado",
            "cpu-monitor",
            Some("container-1"),
        )
        .await?;

    log_manager
        .add_log(
            LogLevel::Error,
            "Falha na conexão com banco de dados",
            "database",
            Some("container-2"),
        )
        .await?;

    // Adicionar log estruturado
    let mut fields = HashMap::new();
    fields.insert("cpu_usage".to_string(), "85.5".to_string());
    fields.insert("memory_usage".to_string(), "45.2".to_string());

    let tags = vec!["monitoring".to_string(), "alert".to_string()];

    log_manager
        .add_log_with_fields(
            LogLevel::Warn,
            "Recursos do container excedendo limites",
            "resource-monitor",
            Some("container-1"),
            fields,
            tags,
        )
        .await?;

    // Consultar logs
    let recent_logs = log_manager.get_recent_logs(5).await?;
    println!("    {} logs recentes consultados", recent_logs.len());

    // Estatísticas de logs
    let log_stats = log_manager.get_log_stats().await?;
    println!(
        "    Logs: {} totais, {} por nível",
        log_stats.total_entries,
        log_stats.entries_by_level.len()
    );

    // 6. Exportação de Métricas
    println!("\n6. � Configurando Exportação de Métricas...");

    // Configurar exportador JSON
    let json_config = ExportConfig {
        format: ExportFormat::Json,
        endpoint: None,
        headers: HashMap::new(),
        batch_size: 10,
        timeout_seconds: 30,
    };
    let mut json_exporter = MetricsExporter::new(json_config);

    // Preparar dados para exportação
    let mut export_data = Vec::new();
    for i in 0..5 {
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 20.0 + i as f64 * 10.0);
        metrics.insert("memory_usage".to_string(), 30.0 + i as f64 * 5.0);

        let mut labels = HashMap::new();
        labels.insert("environment".to_string(), "production".to_string());
        labels.insert("region".to_string(), "us-east-1".to_string());

        export_data.push(ExportData {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (i * 60) as u64,
            metrics,
            labels,
            source: format!("polis-node-{}", i + 1),
        });
    }

    // Exportar em diferentes formatos
    let json_result = json_exporter.export_metrics(&export_data).await?;
    println!(
        "   � Exportação JSON: {} registros em {}ms",
        json_result.records_exported, json_result.duration_ms
    );

    // Configurar exportador Prometheus
    let prometheus_config = ExportConfig {
        format: ExportFormat::Prometheus,
        endpoint: Some("http://prometheus:9090/api/v1/write".to_string()),
        headers: HashMap::new(),
        batch_size: 50,
        timeout_seconds: 10,
    };
    let mut prometheus_exporter = MetricsExporter::new(prometheus_config);

    let prometheus_result = prometheus_exporter.export_metrics(&export_data).await?;
    println!(
        "    Exportação Prometheus: {} registros em {}ms",
        prometheus_result.records_exported, prometheus_result.duration_ms
    );

    // Estatísticas de exportação
    let export_stats = json_exporter.get_export_stats().await?;
    println!(
        "    Exportações: {} totais, {} bem-sucedidas",
        export_stats.total_exports, export_stats.successful_exports
    );

    // 7. Resumo Final
    println!("\n7. � Resumo do Sistema de Monitoramento...");
    println!("    Coleta de métricas funcionando");
    println!("    Health checks implementados");
    println!("    Sistema de alertas configurado");
    println!("    Dashboard com widgets funcionais");
    println!("    Sistema de logs estruturado");
    println!("    Exportação de métricas em múltiplos formatos");

    println!("\n Sistema de Monitoramento do Polis implementado com sucesso!");
    println!("\n Recursos de Monitoramento Implementados:");
    println!("   - Coleta de métricas de sistema e containers");
    println!("   - Health checks personalizáveis");
    println!("   - Sistema de alertas com regras e notificações");
    println!("   - Dashboard com widgets interativos");
    println!("   - Sistema de logs centralizado e estruturado");
    println!("   - Exportação de métricas para múltiplos sistemas");

    Ok(())
}
