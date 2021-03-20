use actix_web::{HttpRequest, HttpResponse, Responder};
use chrono::{DateTime, Utc};
use futures::future::{ready, Ready};
use serde::Serialize;
use sqlx::PgPool;

use crate::ApiResult;

use super::CustomError;

#[derive(Serialize)]
pub struct History {
    pub id: i32,
    pub user_id: i32,
    pub timestamp: DateTime<Utc>,
    pub action: String,
}

#[derive(Serialize)]
pub struct HistoryVec(Vec<History>);

impl Responder for History {
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

impl Responder for HistoryVec {
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

impl History {
    pub async fn find_all(pool: &PgPool) -> ApiResult<HistoryVec> {
        let mut tx = pool.begin().await?;
        let transaction_history = sqlx::query_as!(
            History,
            r#"SELECT id, user_id, timestamp, action FROM transaction_history ORDER BY id"#,
        )
        .fetch_all(&mut tx)
        .await?;
        tx.commit().await?;
        Ok(HistoryVec(transaction_history))
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

    /// Log the transaction and save it to the database.
    pub async fn log(user_id: i32, action: String, pool: &PgPool) -> ApiResult<()> {
        info!("User {} executed: '{}'.", user_id, action);
        History::create(user_id, action, pool).await
    }
}
