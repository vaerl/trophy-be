use actix_web::{body::Body, HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Type};

use crate::{derive_responder::Responder, ApiResult};

#[derive(Serialize, Type)]
#[sqlx(type_name = "log_level")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Important,
}

#[derive(Serialize, Responder)]
pub struct History {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
    pub log_level: LogLevel,
    pub action: String,
    pub value: Option<String>,
}

#[derive(Serialize, Responder)]
pub struct HistoryVec(Vec<History>);

impl History {
    pub async fn find_all(pool: &PgPool) -> ApiResult<HistoryVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            History,
            r#"SELECT id, user_id, timestamp, log_level as "log_level: LogLevel", action, value FROM transaction_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(HistoryVec(transaction_history))
    }

    async fn create(
        user_id: i32,
        log_level: LogLevel,
        action: String,
        value: Option<String>,
        pool: &PgPool,
    ) -> ApiResult<()> {
        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO transaction_history (user_id, timestamp, log_level, action, value) VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            Utc::now(),
            log_level as LogLevel,
            action,
            value
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    /// Log the transaction with the DEBUG-level and save it to the database.
    /// Use this for actions like reading from the database - they typically do not have a value.
    pub async fn debug(user_id: i32, action: String, pool: &PgPool) -> ApiResult<()> {
        debug!("User {} executed: '{}'.", user_id, action);
        History::create(user_id, LogLevel::Debug, action, None, pool).await
    }

    /// Log the transaction with level INFO and save it to the database.
    /// Use this for all actions that create, delete or update and have a value.
    pub async fn info(user_id: i32, action: String, value: String, pool: &PgPool) -> ApiResult<()> {
        info!(
            "User {} executed: '{}' with value {}.",
            user_id, action, value
        );
        History::create(user_id, LogLevel::Info, action, Some(value), pool).await
    }

    /// Log the transaction with level IMPORTANT and save it to the database.
    /// Use this for important actions like evaluate or login.
    pub async fn important(user_id: i32, action: String, pool: &PgPool) -> ApiResult<()> {
        info!("User {} executed: '{}'.", user_id, action);
        History::create(user_id, LogLevel::Important, action, None, pool).await
    }
}
