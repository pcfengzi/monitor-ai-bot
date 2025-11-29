use once_cell::sync::OnceCell;
use sqlx::{Pool, Sqlite};
use sqlx::Row;
use chrono::{DateTime, Utc};
use std::sync::Arc;

static GLOBAL_DB: OnceCell<Arc<NcDb>> = OnceCell::new();

#[derive(Clone)]
pub struct NcDb {
    pool: Pool<Sqlite>,
}

// ====== 对外接口：初始化 / 获取 DB ======

pub async fn init_db() -> Arc<NcDb> {
    if let Some(db) = GLOBAL_DB.get() {
        return db.clone();
    }

    let url = std::env::var("MONITOR_AI_DB_URL")
        .unwrap_or_else(|_| "sqlite://database/monitor_ai.db".to_string());

    let pool = Pool::<Sqlite>::connect(&url)
        .await
        .expect("[notification-center] connect db failed");

    let db = Arc::new(NcDb { pool });
    db.init_schema().await.expect("[notification-center] init_schema failed");

    GLOBAL_DB.set(db.clone()).ok();

    db
}

pub fn get_db() -> Arc<NcDb> {
    GLOBAL_DB.get().expect("NcDb not initialized").clone()
}

// ====== Schema & 结构体 ======

#[derive(Debug, Clone)]
pub struct NotificationTemplate {
    pub id: i64,
    pub scene: String,
    pub channel: String,
    pub lang: String,
    pub version: i64,
    pub content: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct NotificationHistory {
    pub msg_id: String,
    pub user_id: String,
    pub scene: String,
    pub channel: String,
    pub content: String,
    pub status: String,
    pub trace_id: String,
    pub error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl NcDb {
    async fn init_schema(&self) -> Result<(), sqlx::Error> {
        // 只管自己两张表，不动系统原有表
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
                created_at   TEXT NOT NULL,
                sent_at      TEXT,
                delivered_at TEXT
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ===== 模板相关 =====

    pub async fn insert_template(
        &self,
        scene: &str,
        channel: &str,
        lang: &str,
        version: i64,
        content: &str,
        is_active: bool,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            r#"
            INSERT INTO notification_templates
                (scene, channel, lang, version, content, is_active, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
        )
        .bind(scene)
        .bind(channel)
        .bind(lang)
        .bind(version)
        .bind(content)
        .bind(if is_active { 1 } else { 0 })
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn list_templates(&self) -> Result<Vec<NotificationTemplate>, sqlx::Error> {
        let rows = sqlx::query(
            r#"
            SELECT id, scene, channel, lang, version, content, is_active, created_at, updated_at
            FROM notification_templates
            ORDER BY scene, channel, version DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut result = Vec::new();
        for row in rows {
            let created_at: String = row.try_get("created_at")?;
            let updated_at: String = row.try_get("updated_at")?;
            result.push(NotificationTemplate {
                id: row.try_get("id")?,
                scene: row.try_get("scene")?,
                channel: row.try_get("channel")?,
                lang: row.try_get("lang")?,
                version: row.try_get("version")?,
                content: row.try_get("content")?,
                is_active: row.try_get::<i64, _>("is_active")? == 1,
                created_at: DateTime::parse_from_rfc3339(&created_at)
                    .unwrap()
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&updated_at)
                    .unwrap()
                    .with_timezone(&Utc),
            });
        }
        Ok(result)
    }

    // （后面你可以加 get_active_template / 按 scene+channel+lang 查询的函数）

    // ===== 发送记录相关 =====

    pub async fn insert_history(&self, h: &NotificationHistory) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO notification_history
                (msg_id, user_id, scene, channel, content, status, trace_id, error,
                 created_at, sent_at, delivered_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            "#,
        )
        .bind(&h.msg_id)
        .bind(&h.user_id)
        .bind(&h.scene)
        .bind(&h.channel)
        .bind(&h.content)
        .bind(&h.status)
        .bind(&h.trace_id)
        .bind(&h.error)
        .bind(h.created_at.to_rfc3339())
        .bind(h.sent_at.as_ref().map(|d| d.to_rfc3339()))
        .bind(h.delivered_at.as_ref().map(|d| d.to_rfc3339()))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_history(
        &self,
        msg_id: &str,
    ) -> Result<Option<NotificationHistory>, sqlx::Error> {
        let row_opt = sqlx::query(
            r#"
            SELECT msg_id, user_id, scene, channel, content,
                   status, trace_id, error,
                   created_at, sent_at, delivered_at
            FROM notification_history
            WHERE msg_id = ?1
            "#,
        )
        .bind(msg_id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row_opt else {
            return Ok(None);
        };

        let created_at: String = row.try_get("created_at")?;
        let sent_at: Option<String> = row.try_get("sent_at")?;
        let delivered_at: Option<String> = row.try_get("delivered_at")?;

        Ok(Some(NotificationHistory {
            msg_id: row.try_get("msg_id")?,
            user_id: row.try_get("user_id")?,
            scene: row.try_get("scene")?,
            channel: row.try_get("channel")?,
            content: row.try_get("content")?,
            status: row.try_get("status")?,
            trace_id: row.try_get("trace_id")?,
            error: row.try_get("error")?,
            created_at: DateTime::parse_from_rfc3339(&created_at)
                .unwrap()
                .with_timezone(&Utc),
            sent_at: sent_at
                .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
            delivered_at: delivered_at
                .map(|s| DateTime::parse_from_rfc3339(&s).unwrap().with_timezone(&Utc)),
        }))
    }
}
