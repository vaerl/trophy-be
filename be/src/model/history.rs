use chrono::{DateTime, Utc};
use log::Level;
use serde::Serialize;
use sqlx::{PgPool, Type};

use crate::ApiResult;

#[derive(Serialize, Type, Clone)]
#[sqlx(type_name = "log_level")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
}

// I've chosen to do this rather than having some newtype-like construct
impl Into<Level> for LogLevel {
    fn into(self) -> Level {
        match self {
            LogLevel::Debug => Level::Debug,
            LogLevel::Info => Level::Info,
            LogLevel::Warn => Level::Warn,
        }
    }
}

#[derive(Serialize)]
pub struct History {
    pub id: i32,
    pub user_id: i32,
    pub user_name: String,
    pub timestamp: DateTime<Utc>,
    pub log_level: LogLevel,
    pub action: String,
}

#[derive(Serialize)]
pub struct HistoryVec(Vec<History>);

impl History {
    pub async fn find_all(pool: &PgPool) -> ApiResult<HistoryVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            History,
            r#"SELECT transaction_history.id, user_id, users.name as user_name, timestamp, log_level as "log_level: LogLevel", action FROM transaction_history
            INNER JOIN users ON users.id=transaction_history.user_id
            ORDER BY id"#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(HistoryVec(transaction_history))
    }

    pub async fn create(
        user_id: i32,
        log_level: LogLevel,
        action: String,
        pool: &PgPool,
    ) -> ApiResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO transaction_history (user_id, timestamp, log_level, action) VALUES ($1, $2, $3, $4)"#,
            user_id,
            Utc::now(),
            log_level as LogLevel,
            action,
        )
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
