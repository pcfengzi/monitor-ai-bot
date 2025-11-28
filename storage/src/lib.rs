use core_types::{LogEvent, Metric, AlertEvent, AlertSeverity};
use sqlx::{FromRow, SqlitePool};
use chrono::{DateTime, Utc};

#[derive(Clone)]
pub struct Db {
    pool: SqlitePool,
}

#[derive(FromRow)]
struct PluginApiRow {
    plugin: String,
    base_url: String,
}


impl Db {
    pub async fn connect(database_url: &str) -> sqlx::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let db = Self { pool };
        db.init_schema().await?;
        Ok(db)
    }

    async fn init_schema(&self) -> sqlx::Result<()> {
        // 日志表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time TEXT NOT NULL,
                level TEXT NOT NULL,
                plugin TEXT,
                message TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 指标表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time TEXT NOT NULL,
                plugin TEXT NOT NULL,
                name TEXT NOT NULL,
                value REAL NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 告警表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alerts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                time TEXT NOT NULL,
                plugin TEXT NOT NULL,
                metric_name TEXT NOT NULL,
                severity TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // plugin_apis 表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS plugin_apis (
                plugin      TEXT PRIMARY KEY,
                base_url    TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn insert_log(&self, e: &LogEvent) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO logs (time, level, plugin, message)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(e.time.to_rfc3339())
        .bind(format!("{:?}", e.level))
        .bind(e.plugin.clone())
        .bind(&e.message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn insert_metric(&self, m: &Metric) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO metrics (time, plugin, name, value)
            VALUES (?1, ?2, ?3, ?4)
            "#,
        )
        .bind(m.time.to_rfc3339())
        .bind(&m.plugin)
        .bind(&m.name)
        .bind(m.value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn latest_logs(&self, limit: i64) -> sqlx::Result<Vec<LogEvent>> {
        let rows = sqlx::query_as::<_, LogRow>(
            r#"
            SELECT time, level, plugin, message
            FROM logs
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn latest_metrics(&self, limit: i64) -> sqlx::Result<Vec<Metric>> {
        let rows = sqlx::query_as::<_, MetricRow>(
            r#"
            SELECT time, plugin, name, value
            FROM metrics
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn insert_alert(&self, a: &AlertEvent) -> sqlx::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO alerts (time, plugin, metric_name, severity, title, message)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            "#,
        )
        .bind(a.time.to_rfc3339())
        .bind(&a.plugin)
        .bind(&a.metric_name)
        .bind(format!("{:?}", a.severity))
        .bind(&a.title)
        .bind(&a.message)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn latest_alerts(&self, limit: i64) -> sqlx::Result<Vec<AlertEvent>> {
        let rows = sqlx::query_as::<_, AlertRow>(
            r#"
            SELECT time, plugin, metric_name, severity, title, message
            FROM alerts
            ORDER BY id DESC
            LIMIT ?1
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// upsert 插件 API 映射
    pub async fn upsert_plugin_api(
        &self,
        plugin: &str,
        base_url: &str,
    ) -> Result<(), sqlx::Error> {
        let now: DateTime<Utc> = Utc::now();
        let now_str = now.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO plugin_apis (plugin, base_url, updated_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(plugin) DO UPDATE SET
                base_url = excluded.base_url,
                updated_at = excluded.updated_at
            "#,
        )
        .bind(plugin)
        .bind(base_url)
        .bind(now_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// 读取所有插件 API 映射（给 api-server 启动时缓存用）
    pub async fn get_all_plugin_apis(
        &self,
    ) -> Result<Vec<(String, String)>, sqlx::Error> {
        let rows = sqlx::query_as::<_, PluginApiRow>(
            r#"
            SELECT plugin, base_url
            FROM plugin_apis
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|r| (r.plugin, r.base_url))
            .collect())
    }



}

#[derive(FromRow)]
struct LogRow {
    time: String,
    level: String,
    plugin: Option<String>,
    message: String,
}

impl From<LogRow> for LogEvent {
    fn from(row: LogRow) -> Self {
        let time = row
            .time
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now());
        let level = match row.level.as_str() {
            "Debug" => core_types::LogLevel::Debug,
            "Info" => core_types::LogLevel::Info,
            "Warn" => core_types::LogLevel::Warn,
            "Error" => core_types::LogLevel::Error,
            _ => core_types::LogLevel::Info,
        };
        Self {
            time,
            level,
            plugin: row.plugin,
            message: row.message,
            fields: Default::default(),
        }
    }
}

#[derive(FromRow)]
struct MetricRow {
    time: String,
    plugin: String,
    name: String,
    value: f64,
}

impl From<MetricRow> for Metric {
    fn from(row: MetricRow) -> Self {
        let time = row
            .time
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now());
        Self {
            time,
            plugin: row.plugin,
            name: row.name,
            value: row.value,
            labels: Default::default(),
        }
    }
}

#[derive(FromRow)]
struct AlertRow {
    time: String,
    plugin: String,
    metric_name: String,
    severity: String,
    title: String,
    message: String,
}

impl From<AlertRow> for AlertEvent {
    fn from(row: AlertRow) -> Self {
        let time = row
            .time
            .parse()
            .unwrap_or_else(|_| chrono::Utc::now());
        let severity = match row.severity.as_str() {
            "Info" => AlertSeverity::Info,
            "Warning" => AlertSeverity::Warning,
            "Critical" => AlertSeverity::Critical,
            _ => AlertSeverity::Info,
        };
        Self {
            time,
            plugin: row.plugin,
            metric_name: row.metric_name,
            severity,
            title: row.title,
            message: row.message,
            tags: std::collections::HashMap::new(),
        }
    }
}