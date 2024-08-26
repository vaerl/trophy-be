use crate::ApiResult;
use chrono::{DateTime, Utc};
use log::Level;
use serde::Serialize;
use sqlx::{PgPool, Type};
use std::fmt::Display;

#[derive(Serialize, Type, Clone)]
#[sqlx(type_name = "subject_type")]
#[sqlx(rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SubjectType {
    Game,
    Team,
    Outcome,
    History,
    User,
    Eval,
    General,
}

impl Display for SubjectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SubjectType::Game => write!(f, "game"),
            SubjectType::Team => write!(f, "team"),
            SubjectType::Outcome => write!(f, "outcome"),
            SubjectType::History => write!(f, "history"),
            SubjectType::User => write!(f, "user"),
            SubjectType::Eval => write!(f, "eval"),
            SubjectType::General => write!(f, "general"),
        }
    }
}

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

impl Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
        }
    }
}

#[derive(Serialize)]
pub struct History {
    pub id: i32,
    pub user_id: Option<i32>,

    /// Populated by joining `transaction_history` with `users`.
    pub user_name: Option<String>,

    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub operation: String,
    pub subject_id: Option<i32>,
    pub subject_type: SubjectType,
}

#[derive(Serialize)]
pub struct HistoryVec(Vec<History>);

impl History {
    pub async fn find_all(pool: &PgPool) -> ApiResult<HistoryVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            History,
            r#" SELECT transaction_history.id, user_id, users.name as user_name, timestamp, level as "level: LogLevel", operation, subject_id, subject_type as "subject_type: SubjectType" FROM transaction_history
            INNER JOIN users ON transaction_history.user_id=users.id
            ORDER BY id"#,
        )
        .fetch_all(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(HistoryVec(transaction_history))
    }

    /// Create a new history-entry.
    pub async fn create(
        user_id: i32,
        log_level: LogLevel,
        operation: String,
        subject_id: Option<i32>,
        subject_type: SubjectType,
        pool: &PgPool,
    ) -> ApiResult<History> {
        let mut tx = pool.begin().await?;
        let entry = sqlx::query_as!(History,
            r#" WITH inserted as
            (INSERT INTO transaction_history (user_id, timestamp, level, operation, subject_id, subject_type)
            VALUES ($1, $2, $3, $4, $5, $6) RETURNING id, user_id, timestamp, level as "level: LogLevel", operation, subject_id, subject_type as "subject_type: SubjectType")
            SELECT inserted.id, user_id, users.name as user_name, timestamp, "level: LogLevel", operation, subject_id, "subject_type: SubjectType" FROM inserted
            INNER JOIN users ON inserted.user_id=users.id
            "#,
            user_id,
            Utc::now(),
            log_level as LogLevel,
            operation,
            subject_id,
            subject_type as SubjectType
        )
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(entry)
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "History(user: {}, timestamp: {}, operation: {}, subject_type: {}, level: {})",
            self.user_name.clone().unwrap_or(format!("<no user>")),
            self.timestamp,
            self.operation,
            self.subject_type,
            self.level
        )
    }
}
