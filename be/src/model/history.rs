use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use futures::future::{ready, Ready};
use serde::Serialize;
use sqlx::PgPool;

use crate::ApiResult;

use super::CustomError;

#[derive(Serialize)]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
    pub action: String,
}

#[derive(Serialize)]
pub struct TransactionVec(Vec<Transaction>);

impl Responder for Transaction {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Responder for TransactionVec {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Transaction {
    pub async fn find_all(pool: &PgPool) -> ApiResult<TransactionVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            Transaction,
            r#"SELECT id, user_id, timestamp, action FROM transaction_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(TransactionVec(transaction_history))
    }

    async fn create(user_id: i32, action: String, pool: &PgPool) -> ApiResult<()> {
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

#[derive(Serialize)]
pub struct LoginTransaction {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct LoginTransactionVec(Vec<LoginTransaction>);

impl Responder for LoginTransaction {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl Responder for LoginTransactionVec {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

impl LoginTransaction {
    pub async fn find_all(pool: &PgPool) -> ApiResult<Vec<LoginTransaction>> {
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

    pub async fn create(user_id: i32, pool: &PgPool) -> ApiResult<()> {
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
