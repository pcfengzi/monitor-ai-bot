use sqlx::AnyPool;
use tokio::fs;

pub async fn init_schema(pool: &AnyPool, db_type: &str) -> sqlx::Result<()> {
    let script_path = match db_type {
        "postgres" => "storage/migrations/schema_postgres.sql",
        "mysql" => "storage/migrations/schema_mysql.sql",
        _ => "storage/migrations/schema_sqlite.sql",
    };

    let sql = fs::read_to_string(script_path)
        .await
        .expect("Failed to read schema file");

    for stmt in sql.split(';') {
        if !stmt.trim().is_empty() {
            sqlx::query(stmt).execute(pool).await?;
        }
    }

    Ok(())
}
