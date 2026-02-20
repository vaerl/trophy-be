use crate::{
    ApiResult, ToJson,
    middleware::Authenticated,
    model::{History, UserRole},
};
use actix_web::{
    Responder, get,
    web::{self, Data},
};
use sqlx::PgPool;

// TODO paginate
#[get("/history")]
async fn find_all_transactions(
    pool: Data<PgPool>,
    auth: Authenticated,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    History::find_all(&pool).await?.to_json()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_transactions);
}
