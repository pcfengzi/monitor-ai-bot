use sqlx::{any::Any, migrate::MigrateDatabase, AnyPool, Pool};

#[derive(Debug, Clone)]
pub struct DbConfig {
    pub db_type: String,
    pub url: String,
}

impl DbConfig {
    pub fn from_args(db_type: Option<&str>, db_url: Option<&str>) -> Self {
        dotenv::dotenv().ok();
        let db_type = db_type
            .map(String::from)
            .or_else(|| std::env::var("DB_TYPE").ok())
            .unwrap_or_else(|| "sqlite".to_string());

        let url = db_url
            .map(String::from)
            .or_else(|| std::env::var("MONITOR_AI_DB_URL").ok())
            .unwrap_or_else(|| "sqlite://database/monitor_ai.db".to_string());

        DbConfig { db_type, url }
    }
}

pub async fn create_pool(config: &DbConfig) -> sqlx::Result<AnyPool> {
    let db_url = &config.url;

    // 检查数据库是否存在，不存在则创建（仅支持 SQLite）
    match config.db_type.as_str() {
        "sqlite" => {
            if !sqlx::Sqlite::database_exists(db_url).await.unwrap_or(false) {
                println!("SQLite 数据库不存在，正在创建: {}", db_url);
                sqlx::Sqlite::create_database(db_url)
                    .await
                    .expect("无法创建 SQLite 数据库");
            }
        }
        "postgres" => {
            if !sqlx::Postgres::database_exists(db_url).await.unwrap_or(false) {
                println!("PostgreSQL 数据库不存在，正在创建...");
                sqlx::Postgres::create_database(db_url)
                    .await
                    .expect("无法创建 PostgreSQL 数据库");
            }
        }
        "mysql" => {
            if !sqlx::MySql::database_exists(db_url).await.unwrap_or(false) {
                println!("MySQL 数据库不存在，正在创建...");
                sqlx::MySql::create_database(db_url)
                    .await
                    .expect("无法创建 MySQL 数据库");
            }
        }
        other => {
            println!("未知数据库类型 {}，跳过创建逻辑", other);
        }
    }

    Pool::<Any>::connect(db_url).await
}