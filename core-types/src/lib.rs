use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 日志级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

/// 一条日志事件（供 host / api-server / 存储使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEvent {
    pub time: DateTime<Utc>,
    pub level: LogLevel,
    pub plugin: Option<String>,   // 哪个插件产生的（host 自己写日志时可为 None）
    pub message: String,
    pub fields: HashMap<String, String>,
}

/// 一条监控指标（时间点）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub time: DateTime<Utc>,
    pub plugin: String,           // 来源插件
    pub name: String,             // 如 "cpu_usage"
    pub value: f64,               // 数值型指标
    pub labels: HashMap<String, String>,
}

/// 告警级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// 告警事件（可由 AI 或规则引擎产生）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    pub time: DateTime<Utc>,
    pub plugin: String,           // 来源插件或 "ai-engine"
    pub metric_name: String,  
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub tags: HashMap<String, String>,
}
