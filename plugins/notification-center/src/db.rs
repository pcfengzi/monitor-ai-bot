// plugins/notification-center/src/db.rs
use std::sync::Arc;

use chrono::{DateTime, Utc};
use once_cell::sync::OnceCell;
use sqlx::{Pool, Sqlite, Row};

use crate::types::{Channel, MessageStatus};

static GLOBAL_DB: OnceCell<Arc<NcDb>> = OnceCell::new();

#[derive(Clone)]
pub struct NcDb {
    pool: Pool<Sqlite>,
}

pub async fn init_db() -> Result<(), sqlx::Error> {
    let db = get_or_init().await?;
    db.init_schema().await
}

pub async fn get_or_init() -> Result<Arc<NcDb>, sqlx::Error> {
    if let Some(db) = GLOBAL_DB.get() {
        return Ok(db.clone());
    }

    let url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());

    let pool = Pool::<Sqlite>::connect(&url).await?;
    let db = Arc::new(NcDb { pool });

    GLOBAL_DB.set(db.clone()).ok();

    Ok(db)
}

pub fn db() -> Arc<NcDb> {
    GLOBAL_DB
        .get()
        .expect("NcDb not initialized")
        .clone()
}

// ======== schema 初始化 ========

impl NcDb {
    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // 模板
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_templates (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                scene       TEXT NOT NULL,
                channel     TEXT NOT NULL,
                lang        TEXT NOT NULL,
                version     INTEGER NOT NULL,
                content     TEXT NOT NULL,
                is_active   INTEGER NOT NULL,
                created_at  TEXT NOT NULL,
                updated_at  TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 发送记录
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_history (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                msg_id       TEXT NOT NULL UNIQUE,
                user_id      TEXT NOT NULL,
                scene        TEXT NOT NULL,
                channel      TEXT NOT NULL,
                content      TEXT NOT NULL,
                status       TEXT NOT NULL,
                trace_id     TEXT NOT NULL,
                error        TEXT,
                retries      INTEGER NOT NULL DEFAULT 0,
                created_at   TEXT NOT NULL,
                waiting_at   TEXT,
                processing_at TEXT,
                sent_at      TEXT,
                delivered_at TEXT,
                failed_at    TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // 用户偏好（简单版本）
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_user_pref (
                id           INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id      TEXT NOT NULL,
                channel      TEXT NOT NULL,
                enabled      INTEGER NOT NULL,
                updated_at   TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// ======== 发送记录操作（状态机） ========

#[derive(Debug, Clone)]
pub struct HistoryRecord {
    pub msg_id: String,
    pub user_id: String,
    pub scene: String,
    pub channel: String,
    pub content: String,
    pub status: MessageStatus,
    pub trace_id: String,
    pub error: Option<String>,
    pub retries: i32,
    pub created_at: DateTime<Utc>,
}

impl NcDb {
    pub async fn insert_waiting(
        &self,
        msg_id: &str,
        user_id: &str,
        scene: &str,
        trace_id: &str,
        created_at: DateTime<Utc>,
    ) -> Result<(), sqlx::Error> {
        let ts = created_at.to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO notification_history
                (msg_id, user_id, scene, channel, content, status, trace_id,
                 retries, created_at, waiting_at)
            VALUES (?1, ?2, ?3, '', '', 'waiting', ?4, 0, ?5, ?5)
            "#,
        )
        .bind(msg_id)
        .bind(user_id)
        .bind(scene)
        .bind(trace_id)
        .bind(ts)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_status_processing(
        &self,
        msg_id: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            UPDATE notification_history
            SET status = 'processing',
                processing_at = ?1
            WHERE msg_id = ?2
            "#,
        )
        .bind(now)
        .bind(msg_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_status_final(
        &self,
        msg_id: &str,
        status: MessageStatus,
        channel: &Channel,
        content: &str,
        error: Option<String>,
        retries: i32,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        let status_str = status.as_str();

        // 不同状态更新不同时间字段
        let (sent_at, delivered_at, failed_at) = match status {
            MessageStatus::Sent => (Some(now.clone()), None, None),
            MessageStatus::Delivered => (None, Some(now.clone()), None),
            MessageStatus::Failed => (None, None, Some(now.clone())),
            MessageStatus::Blocked => (None, None, None),
            _ => (None, None, None),
        };

        sqlx::query(
            r#"
            UPDATE notification_history
            SET status = ?1,
                channel = ?2,
                content = ?3,
                error = ?4,
                retries = ?5,
                sent_at = COALESCE(sent_at, ?6),
                delivered_at = COALESCE(delivered_at, ?7),
                failed_at = COALESCE(failed_at, ?8)
            WHERE msg_id = ?9
            "#,
        )
        .bind(status_str)
        .bind(channel.as_str())
        .bind(content)
        .bind(error)
        .bind(retries)
        .bind(sent_at)
        .bind(delivered_at)
        .bind(failed_at)
        .bind(msg_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_history(
        &self,
        msg_id: &str,
    ) -> Result<Option<HistoryRecord>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT msg_id, user_id, scene, channel, content,
                   status, trace_id, error, retries, created_at
            FROM notification_history
            WHERE msg_id = ?1
            "#,
        )
        .bind(msg_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(r) = row else {
            return Ok(None);
        };

        let created_at: String = r.try_get("created_at")?;
        let status: String = r.try_get("status")?;

        Ok(Some(HistoryRecord {
            msg_id: r.try_get("msg_id")?,
            user_id: r.try_get("user_id")?,
            scene: r.try_get("scene")?,
            channel: r.try_get("channel")?,
            content: r.try_get("content")?,
            status: match status.as_str() {
                "waiting" => MessageStatus::Waiting,
                "processing" => MessageStatus::Processing,
                "sent" => MessageStatus::Sent,
                "delivered" => MessageStatus::Delivered,
                "failed" => MessageStatus::Failed,
                "blocked" => MessageStatus::Blocked,
                _ => MessageStatus::Failed,
            },
            trace_id: r.try_get("trace_id")?,
            error: r.try_get("error")?,
            retries: r.try_get("retries")?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .unwrap()
                .with_timezone(&Utc),
        }))
    }

    // 简单模板获取（你可以后面再做管理 API）
    pub async fn get_template(
        &self,
        scene: &str,
        channel: &Channel,
    ) -> Result<Option<String>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT content
            FROM notification_templates
            WHERE scene = ?1 AND channel = ?2 AND is_active = 1
            ORDER BY version DESC
            LIMIT 1
            "#,
        )
        .bind(scene)
        .bind(channel.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get::<String, _>("content")))
    }

    // 用户偏好：是否允许某渠道
    pub async fn is_channel_enabled(
        &self,
        user_id: &str,
        channel: &Channel,
    ) -> Result<bool, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT enabled
            FROM notification_user_pref
            WHERE user_id = ?1 AND channel = ?2
            ORDER BY updated_at DESC
            LIMIT 1
            "#,
        )
        .bind(user_id)
        .bind(channel.as_str())
        .fetch_optional(&self.pool)
        .await?;

        let enabled = row
            .map(|r| r.get::<i64, _>("enabled") == 1)
            .unwrap_or(true); // 默认允许

        Ok(enabled)
    }
}
