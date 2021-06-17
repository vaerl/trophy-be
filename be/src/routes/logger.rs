use actix_web::{get, web};
use sqlx::PgPool;

use crate::{
    model::{History, HistoryVec, UserRole, UserToken},
    ApiResult,
};

#[get("/logs")]
async fn find_all_transactions(
    token: UserToken,
    db_pool: web::Data<PgPool>,
) -> ApiResult<HistoryVec> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
        .await?;
    History::read(user.id, format!("find all transactions"), db_pool.get_ref()).await?;
    History::find_all(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_transactions);
}
