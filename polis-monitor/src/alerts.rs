use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AlertStatus {
    Active,
    Resolved,
    Acknowledged,
    Suppressed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub source: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub resolved_at: Option<u64>,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: String,
    pub severity: AlertSeverity,
    pub enabled: bool,
    pub cooldown_seconds: u64,
    pub last_triggered: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: String,
    pub name: String,
    pub channel_type: NotificationType,
    pub config: HashMap<String, String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NotificationType {
    Email,
    Slack,
    Webhook,
    Console,
}

pub struct AlertManager {
    alerts: HashMap<String, Alert>,
    rules: HashMap<String, AlertRule>,
    channels: HashMap<String, NotificationChannel>,
    next_alert_id: u64,
}

impl AlertManager {
    pub fn new() -> Self {
        let mut manager = Self {
            alerts: HashMap::new(),
            rules: HashMap::new(),
            channels: HashMap::new(),
            next_alert_id: 1,
        };

        // Create default notification channel
        manager
            .create_notification_channel(
                "console",
                "Console Output",
                NotificationType::Console,
                HashMap::new(),
            )
            .unwrap();

        manager
    }

    pub fn create_alert_rule(
        &mut self,
        name: &str,
        description: &str,
        condition: &str,
        severity: AlertSeverity,
    ) -> Result<String> {
        let rule_id = format!("rule-{}", self.next_alert_id);
        self.next_alert_id += 1;

        let rule = AlertRule {
            id: rule_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            condition: condition.to_string(),
            severity,
            enabled: true,
            cooldown_seconds: 300, // 5 minutes default
            last_triggered: None,
        };

        self.rules.insert(rule_id.clone(), rule);
        println!("🚨 Regra de alerta criada: {} ({})", name, rule_id);

        Ok(rule_id)
    }

    pub fn create_notification_channel(
        &mut self,
        id: &str,
        name: &str,
        channel_type: NotificationType,
        config: HashMap<String, String>,
    ) -> Result<()> {
        let channel = NotificationChannel {
            id: id.to_string(),
            name: name.to_string(),
            channel_type,
            config,
            enabled: true,
        };

        self.channels.insert(id.to_string(), channel);
        println!("📢 Canal de notificação criado: {} ({})", name, id);

        Ok(())
    }

    pub async fn evaluate_rules(&mut self, metrics: &HashMap<String, f64>) -> Result<Vec<String>> {
        let mut triggered_alerts = Vec::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Collect rule IDs and conditions first to avoid borrow conflicts
        let mut rules_to_evaluate = Vec::new();
        for (rule_id, rule) in &self.rules {
            if !rule.enabled {
                continue;
            }

            // Check cooldown
            if let Some(last_triggered) = rule.last_triggered {
                if current_time - last_triggered < rule.cooldown_seconds {
                    continue;
                }
            }

            rules_to_evaluate.push((
                rule_id.clone(),
                rule.condition.clone(),
                rule.name.clone(),
                rule.description.clone(),
                rule.severity.clone(),
            ));
        }

        // Evaluate conditions and create alerts
        for (rule_id, condition, name, description, severity) in rules_to_evaluate {
            if self.evaluate_condition(&condition, metrics).await? {
                let alert_id = self
                    .create_alert(&name, &description, severity, "alert-rule", &rule_id)
                    .await?;

                // Update last triggered time
                if let Some(rule) = self.rules.get_mut(&rule_id) {
                    rule.last_triggered = Some(current_time);
                }

                triggered_alerts.push(alert_id);
            }
        }

        Ok(triggered_alerts)
    }

    pub async fn create_alert(
        &mut self,
        title: &str,
        description: &str,
        severity: AlertSeverity,
        source: &str,
        rule_id: &str,
    ) -> Result<String> {
        let alert_id = format!("alert-{}", self.next_alert_id);
        self.next_alert_id += 1;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut labels = HashMap::new();
        labels.insert("rule_id".to_string(), rule_id.to_string());
        labels.insert("source".to_string(), source.to_string());

        let alert = Alert {
            id: alert_id.clone(),
            title: title.to_string(),
            description: description.to_string(),
            severity,
            status: AlertStatus::Active,
            source: source.to_string(),
            created_at: current_time,
            updated_at: current_time,
            resolved_at: None,
            labels,
            annotations: HashMap::new(),
        };

        self.alerts.insert(alert_id.clone(), alert.clone());

        // Send notifications
        self.send_notifications(&alert).await?;

        println!("🚨 Alerta criado: {} ({})", title, alert_id);

        Ok(alert_id)
    }

    pub async fn resolve_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.alerts.get_mut(alert_id) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            alert.status = AlertStatus::Resolved;
            alert.resolved_at = Some(current_time);
            alert.updated_at = current_time;

            println!("✅ Alerta resolvido: {}", alert_id);
        } else {
            return Err(PolisError::Api(format!(
                "Alerta '{}' não encontrado",
                alert_id
            )));
        }

        Ok(())
    }

    pub async fn acknowledge_alert(&mut self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.alerts.get_mut(alert_id) {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

            alert.status = AlertStatus::Acknowledged;
            alert.updated_at = current_time;

            println!("👁️ Alerta reconhecido: {}", alert_id);
        } else {
            return Err(PolisError::Api(format!(
                "Alerta '{}' não encontrado",
                alert_id
            )));
        }

        Ok(())
    }

    pub async fn get_active_alerts(&self) -> Result<Vec<Alert>> {
        let active_alerts: Vec<Alert> = self
            .alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Active)
            .cloned()
            .collect();

        Ok(active_alerts)
    }

    pub async fn get_alerts_by_severity(&self, severity: AlertSeverity) -> Result<Vec<Alert>> {
        let filtered_alerts: Vec<Alert> = self
            .alerts
            .values()
            .filter(|alert| alert.severity == severity)
            .cloned()
            .collect();

        Ok(filtered_alerts)
    }

    pub async fn get_alert_summary(&self) -> Result<AlertSummary> {
        let total_alerts = self.alerts.len();
        let active_alerts = self
            .alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Active)
            .count();
        let resolved_alerts = self
            .alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Resolved)
            .count();
        let acknowledged_alerts = self
            .alerts
            .values()
            .filter(|alert| alert.status == AlertStatus::Acknowledged)
            .count();

        let critical_alerts = self
            .alerts
            .values()
            .filter(|alert| {
                alert.severity == AlertSeverity::Critical && alert.status == AlertStatus::Active
            })
            .count();
        let high_alerts = self
            .alerts
            .values()
            .filter(|alert| {
                alert.severity == AlertSeverity::High && alert.status == AlertStatus::Active
            })
            .count();

        Ok(AlertSummary {
            total_alerts,
            active_alerts,
            resolved_alerts,
            acknowledged_alerts,
            critical_alerts,
            high_alerts,
            total_rules: self.rules.len(),
            enabled_rules: self.rules.values().filter(|rule| rule.enabled).count(),
            total_channels: self.channels.len(),
            enabled_channels: self
                .channels
                .values()
                .filter(|channel| channel.enabled)
                .count(),
        })
    }

    async fn evaluate_condition(
        &self,
        condition: &str,
        metrics: &HashMap<String, f64>,
    ) -> Result<bool> {
        // Simplified condition evaluation
        // In a real implementation, this would parse and evaluate complex expressions

        if condition.contains("cpu_usage > 80") {
            if let Some(&cpu_usage) = metrics.get("cpu_usage") {
                return Ok(cpu_usage > 80.0);
            }
        }

        if condition.contains("memory_usage > 90") {
            if let Some(&memory_usage) = metrics.get("memory_usage") {
                return Ok(memory_usage > 90.0);
            }
        }

        if condition.contains("disk_usage > 95") {
            if let Some(&disk_usage) = metrics.get("disk_usage") {
                return Ok(disk_usage > 95.0);
            }
        }

        if condition.contains("container_count > 100") {
            if let Some(&container_count) = metrics.get("container_count") {
                return Ok(container_count > 100.0);
            }
        }

        // Default to false for unknown conditions
        Ok(false)
    }

    async fn send_notifications(&self, alert: &Alert) -> Result<()> {
        for channel in self.channels.values() {
            if !channel.enabled {
                continue;
            }

            match channel.channel_type {
                NotificationType::Console => {
                    println!(
                        "🚨 NOTIFICAÇÃO: {} - {} ({:?})",
                        alert.title, alert.description, alert.severity
                    );
                }
                NotificationType::Email => {
                    println!(
                        "📧 Email enviado para: {} - {}",
                        channel
                            .config
                            .get("email")
                            .unwrap_or(&"unknown".to_string()),
                        alert.title
                    );
                }
                NotificationType::Slack => {
                    println!(
                        "💬 Slack enviado para: {} - {}",
                        channel
                            .config
                            .get("webhook")
                            .unwrap_or(&"unknown".to_string()),
                        alert.title
                    );
                }
                NotificationType::Webhook => {
                    println!(
                        "🔗 Webhook enviado para: {} - {}",
                        channel.config.get("url").unwrap_or(&"unknown".to_string()),
                        alert.title
                    );
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSummary {
    pub total_alerts: usize,
    pub active_alerts: usize,
    pub resolved_alerts: usize,
    pub acknowledged_alerts: usize,
    pub critical_alerts: usize,
    pub high_alerts: usize,
    pub total_rules: usize,
    pub enabled_rules: usize,
    pub total_channels: usize,
    pub enabled_channels: usize,
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}
