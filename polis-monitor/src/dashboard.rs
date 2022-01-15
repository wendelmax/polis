use polis_core::{PolisError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widgets: Vec<Widget>,
    pub layout: DashboardLayout,
    pub created_at: u64,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub position: WidgetPosition,
    pub size: WidgetSize,
    pub config: HashMap<String, String>,
    pub data_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WidgetType {
    Metric,
    Chart,
    Table,
    Status,
    Alert,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: u32,
    pub rows: u32,
    pub grid_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub dashboard_id: String,
    pub widgets_data: HashMap<String, WidgetData>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetData {
    pub widget_id: String,
    pub data: serde_json::Value,
    pub status: String,
    pub last_updated: u64,
}

pub struct DashboardManager {
    dashboards: HashMap<String, Dashboard>,
    next_dashboard_id: u64,
    next_widget_id: u64,
}

impl DashboardManager {
    pub fn new() -> Self {
        let mut manager = Self {
            dashboards: HashMap::new(),
            next_dashboard_id: 1,
            next_widget_id: 1,
        };

        // Create default dashboard
        manager.create_default_dashboard().unwrap();

        manager
    }

    pub fn create_dashboard(&mut self, name: &str, description: &str) -> Result<String> {
        let dashboard_id = format!("dashboard-{}", self.next_dashboard_id);
        self.next_dashboard_id += 1;

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let dashboard = Dashboard {
            id: dashboard_id.clone(),
            name: name.to_string(),
            description: description.to_string(),
            widgets: Vec::new(),
            layout: DashboardLayout {
                columns: 12,
                rows: 8,
                grid_size: 1,
            },
            created_at: current_time,
            updated_at: current_time,
        };

        self.dashboards.insert(dashboard_id.clone(), dashboard);
        println!("📊 Dashboard criado: {} ({})", name, dashboard_id);

        Ok(dashboard_id)
    }

    pub fn add_widget(
        &mut self,
        dashboard_id: &str,
        title: &str,
        widget_type: WidgetType,
        data_source: &str,
    ) -> Result<String> {
        let dashboard = self.dashboards.get_mut(dashboard_id).ok_or_else(|| {
            PolisError::Api(format!("Dashboard '{}' não encontrado", dashboard_id))
        })?;

        let widget_id = format!("widget-{}", self.next_widget_id);
        self.next_widget_id += 1;

        let widget = Widget {
            id: widget_id.clone(),
            title: title.to_string(),
            widget_type,
            position: WidgetPosition { x: 0, y: 0 },
            size: WidgetSize {
                width: 4,
                height: 3,
            },
            config: HashMap::new(),
            data_source: data_source.to_string(),
        };

        dashboard.widgets.push(widget);
        dashboard.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        println!(
            "📊 Widget adicionado ao dashboard {}: {} ({})",
            dashboard_id, title, widget_id
        );

        Ok(widget_id)
    }

    pub async fn get_dashboard_data(&self, dashboard_id: &str) -> Result<DashboardData> {
        let dashboard = self.dashboards.get(dashboard_id).ok_or_else(|| {
            PolisError::Api(format!("Dashboard '{}' não encontrado", dashboard_id))
        })?;

        let mut widgets_data = HashMap::new();
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for widget in &dashboard.widgets {
            let data = self.generate_widget_data(widget).await?;
            widgets_data.insert(widget.id.clone(), data);
        }

        Ok(DashboardData {
            dashboard_id: dashboard_id.to_string(),
            widgets_data,
            timestamp: current_time,
        })
    }

    pub async fn get_dashboard_summary(&self) -> Result<DashboardSummary> {
        let total_dashboards = self.dashboards.len();
        let total_widgets: usize = self.dashboards.values().map(|d| d.widgets.len()).sum();

        let mut widget_types = HashMap::new();
        for dashboard in self.dashboards.values() {
            for widget in &dashboard.widgets {
                *widget_types.entry(widget.widget_type.clone()).or_insert(0) += 1;
            }
        }

        Ok(DashboardSummary {
            total_dashboards,
            total_widgets,
            widget_types,
            last_updated: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    pub fn list_dashboards(&self) -> Result<Vec<Dashboard>> {
        Ok(self.dashboards.values().cloned().collect())
    }

    pub fn get_dashboard(&self, dashboard_id: &str) -> Result<Option<Dashboard>> {
        Ok(self.dashboards.get(dashboard_id).cloned())
    }

    async fn generate_widget_data(&self, widget: &Widget) -> Result<WidgetData> {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let data = match widget.widget_type {
            WidgetType::Metric => self.generate_metric_data(widget).await?,
            WidgetType::Chart => self.generate_chart_data(widget).await?,
            WidgetType::Table => self.generate_table_data(widget).await?,
            WidgetType::Status => self.generate_status_data(widget).await?,
            WidgetType::Alert => self.generate_alert_data(widget).await?,
            WidgetType::Log => self.generate_log_data(widget).await?,
        };

        Ok(WidgetData {
            widget_id: widget.id.clone(),
            data,
            status: "ok".to_string(),
            last_updated: current_time,
        })
    }

    async fn generate_metric_data(&self, widget: &Widget) -> Result<serde_json::Value> {
        // Simulate metric data based on data source
        let data = match widget.data_source.as_str() {
            "cpu_usage" => {
                serde_json::json!({
                    "value": 25.5,
                    "unit": "%",
                    "trend": "stable",
                    "threshold": 80.0
                })
            }
            "memory_usage" => {
                serde_json::json!({
                    "value": 50.0,
                    "unit": "%",
                    "trend": "increasing",
                    "threshold": 90.0
                })
            }
            "disk_usage" => {
                serde_json::json!({
                    "value": 40.0,
                    "unit": "%",
                    "trend": "stable",
                    "threshold": 95.0
                })
            }
            "container_count" => {
                serde_json::json!({
                    "value": 12,
                    "unit": "containers",
                    "trend": "stable",
                    "threshold": 100
                })
            }
            _ => {
                serde_json::json!({
                    "value": 0,
                    "unit": "unknown",
                    "trend": "unknown",
                    "threshold": 0
                })
            }
        };

        Ok(data)
    }

    async fn generate_chart_data(&self, widget: &Widget) -> Result<serde_json::Value> {
        // Simulate chart data
        let data = serde_json::json!({
            "type": "line",
            "data": {
                "labels": ["00:00", "04:00", "08:00", "12:00", "16:00", "20:00"],
                "datasets": [{
                    "label": widget.data_source,
                    "data": [20.0, 25.0, 30.0, 28.0, 35.0, 32.0],
                    "borderColor": "rgb(75, 192, 192)",
                    "backgroundColor": "rgba(75, 192, 192, 0.2)"
                }]
            },
            "options": {
                "responsive": true,
                "scales": {
                    "y": {
                        "beginAtZero": true,
                        "max": 100
                    }
                }
            }
        });

        Ok(data)
    }

    async fn generate_table_data(&self, _widget: &Widget) -> Result<serde_json::Value> {
        // Simulate table data
        let data = serde_json::json!({
            "columns": ["Container ID", "Name", "Status", "CPU %", "Memory %"],
            "rows": [
                ["container-1", "web-app", "running", "15.2", "25.0"],
                ["container-2", "api-server", "running", "8.5", "18.0"],
                ["container-3", "database", "running", "5.0", "45.0"],
                ["container-4", "cache", "stopped", "0.0", "0.0"]
            ]
        });

        Ok(data)
    }

    async fn generate_status_data(&self, _widget: &Widget) -> Result<serde_json::Value> {
        // Simulate status data
        let data = serde_json::json!({
            "status": "healthy",
            "message": "All systems operational",
            "last_check": SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "components": [
                {"name": "API Server", "status": "healthy"},
                {"name": "Database", "status": "healthy"},
                {"name": "Cache", "status": "degraded"},
                {"name": "Queue", "status": "healthy"}
            ]
        });

        Ok(data)
    }

    async fn generate_alert_data(&self, _widget: &Widget) -> Result<serde_json::Value> {
        // Simulate alert data
        let data = serde_json::json!({
            "active_alerts": 2,
            "critical_alerts": 0,
            "high_alerts": 1,
            "medium_alerts": 1,
            "low_alerts": 0,
            "recent_alerts": [
                {
                    "id": "alert-1",
                    "title": "High CPU Usage",
                    "severity": "high",
                    "created_at": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - 300
                },
                {
                    "id": "alert-2",
                    "title": "Memory Usage Warning",
                    "severity": "medium",
                    "created_at": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - 600
                }
            ]
        });

        Ok(data)
    }

    async fn generate_log_data(&self, _widget: &Widget) -> Result<serde_json::Value> {
        // Simulate log data
        let data = serde_json::json!({
            "logs": [
                {
                    "timestamp": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    "level": "INFO",
                    "message": "Container started successfully",
                    "source": "container-1"
                },
                {
                    "timestamp": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - 60,
                    "level": "WARN",
                    "message": "High memory usage detected",
                    "source": "container-2"
                },
                {
                    "timestamp": SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - 120,
                    "level": "ERROR",
                    "message": "Failed to connect to database",
                    "source": "api-server"
                }
            ]
        });

        Ok(data)
    }

    fn create_default_dashboard(&mut self) -> Result<()> {
        let dashboard_id =
            self.create_dashboard("Sistema Principal", "Dashboard principal do Polis")?;

        // Add system metrics widgets
        self.add_widget(&dashboard_id, "CPU Usage", WidgetType::Metric, "cpu_usage")?;
        self.add_widget(
            &dashboard_id,
            "Memory Usage",
            WidgetType::Metric,
            "memory_usage",
        )?;
        self.add_widget(
            &dashboard_id,
            "Disk Usage",
            WidgetType::Metric,
            "disk_usage",
        )?;
        self.add_widget(
            &dashboard_id,
            "Container Count",
            WidgetType::Metric,
            "container_count",
        )?;

        // Add charts
        self.add_widget(&dashboard_id, "CPU Trend", WidgetType::Chart, "cpu_usage")?;
        self.add_widget(
            &dashboard_id,
            "Memory Trend",
            WidgetType::Chart,
            "memory_usage",
        )?;

        // Add status and alerts
        self.add_widget(&dashboard_id, "System Status", WidgetType::Status, "system")?;
        self.add_widget(&dashboard_id, "Active Alerts", WidgetType::Alert, "alerts")?;

        // Add container table
        self.add_widget(&dashboard_id, "Containers", WidgetType::Table, "containers")?;

        // Add logs
        self.add_widget(&dashboard_id, "Recent Logs", WidgetType::Log, "logs")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSummary {
    pub total_dashboards: usize,
    pub total_widgets: usize,
    pub widget_types: HashMap<WidgetType, usize>,
    pub last_updated: u64,
}

impl Default for DashboardManager {
    fn default() -> Self {
        Self::new()
    }
}
