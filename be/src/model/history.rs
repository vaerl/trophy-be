use actix_web::HttpRequest;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;

use super::{DataBaseError, UserRole};

pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
    pub action: String,
}

impl Transaction {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<Transaction>, DataBaseError> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            Transaction,
            r#"SELECT id, user_id, timestamp, action FROM transaction_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(transaction_history)
    }

    async fn create(user_id: i32, action: String, pool: &PgPool) -> Result<(), DataBaseError> {
        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO transaction_history (user_id, timestamp, action) VALUES ($1, $2, $3)"#,
            user_id,
            Utc::now(),
            action
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}

pub struct LoginTransaction {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
}

impl LoginTransaction {
    pub async fn find_all(pool: &PgPool) -> Result<Vec<LoginTransaction>, DataBaseError> {
        let mut tx = pool.begin().await?;
        let login_history = sqlx::query_as!(
            LoginTransaction,
            r#"SELECT id, user_id, timestamp FROM login_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(login_history)
    }

    pub async fn create(user_id: i32, pool: &PgPool) -> Result<(), DataBaseError> {
        let mut tx = pool.begin().await?;
        sqlx::query!(
            r#"INSERT INTO login_history (user_id, timestamp) VALUES ($1, $2)"#,
            user_id,
            Utc::now()
        )
        .execute(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
