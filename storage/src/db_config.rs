use sqlx::{AnyPool, Pool, any::Any};

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
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .unwrap_or_else(|| "sqlite://database/monitor_ai.db".to_string());

        DbConfig { db_type, url }
    }
}

pub async fn create_pool(config: &DbConfig) -> sqlx::Result<AnyPool> {
    Pool::<Any>::connect(&config.url).await
}
