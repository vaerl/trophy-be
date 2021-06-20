use actix_web::{body::Body, HttpRequest, HttpResponse, Responder};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::{PgPool, Type};
use std::fmt::Display;

use crate::{derive_responder::Responder, ApiResult};

use super::User;

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
}

#[derive(Serialize, Responder)]
pub struct HistoryVec(Vec<History>);

impl History {
    pub async fn find_all(pool: &PgPool) -> ApiResult<HistoryVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            History,
            r#"SELECT id, user_id, timestamp, log_level as "log_level: LogLevel", action FROM transaction_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(HistoryVec(transaction_history))
    }

    // TODO run cargo sqlx prepare
    pub async fn create(
        user_id: i32,
        log_level: LogLevel,
        action: String,
        pool: &PgPool,
    ) -> ApiResult<()> {
        match log_level {
            LogLevel::Debug => debug!("{}", action),
            LogLevel::Info => info!("{}", action),
            LogLevel::Important => info!("{}", action),
        }

        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO transaction_history (user_id, timestamp, log_level, action) VALUES ($1, $2, $3, $4)"#,
            user_id,
            Utc::now(),
            log_level as LogLevel,
            action,
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

pub trait TypeInfo {
    fn type_name(&self) -> String;
}

// NOTE using "?Send" might go horribly wrong!
#[async_trait(?Send)]
pub trait Log<T> {
    async fn log_read(self, user_id: i32, pool: &PgPool) -> ApiResult<T>;
    async fn log_create(self, user_id: i32, pool: &PgPool) -> ApiResult<T>;
    async fn log_update(self, user_id: i32, pool: &PgPool) -> ApiResult<T>;
    async fn log_delete(self, user_id: i32, pool: &PgPool) -> ApiResult<T>;
    async fn log_info(self, user_id: i32, action: String, pool: &PgPool) -> ApiResult<T>;
}

#[async_trait(?Send)]
impl<T> Log<T> for T
where
    T: Display + TypeInfo,
{
    async fn log_read(self, user_id: i32, pool: &PgPool) -> ApiResult<T> {
        let action = format!("User {} read {} -> {}", user_id, self.type_name(), self);
        History::create(user_id, LogLevel::Debug, action, pool).await?;
        Ok(self)
    }

    async fn log_create(self, user_id: i32, pool: &PgPool) -> ApiResult<T> {
        let action = format!("User {} created {} -> {}.", user_id, self.type_name(), self);
        History::create(user_id, LogLevel::Info, action, pool).await?;
        Ok(self)
    }

    async fn log_update(self, user_id: i32, pool: &PgPool) -> ApiResult<T> {
        let action = format!(
            "User {} updated {} ->  value changed to {}.",
            user_id,
            self.type_name(),
            self
        );
        History::create(user_id, LogLevel::Info, action, pool).await?;
        Ok(self)
    }

    async fn log_delete(self, user_id: i32, pool: &PgPool) -> ApiResult<T> {
        let action = format!("User {} deleted {} -> {}.", user_id, self.type_name(), self);
        History::create(user_id, LogLevel::Info, action, pool).await?;
        Ok(self)
    }

    async fn log_info(self, user_id: i32, action: String, pool: &PgPool) -> ApiResult<T> {
        let action = format!("User {} executed: {}", user_id, action);
        History::create(user_id, LogLevel::Info, action, pool).await?;
        Ok(self)
    }
}

#[async_trait(?Send)]
pub trait LogUserAction {
    async fn log_action(self, action: String, pool: &PgPool) -> ApiResult<User>;
}

#[async_trait(?Send)]
impl LogUserAction for User {
    async fn log_action(self, action: String, pool: &PgPool) -> ApiResult<User> {
        let action = format!("User {} executed: {}", self.id, action);
        History::create(self.id, LogLevel::Info, action, pool).await?;
        Ok(self)
    }
}
