use actix_web::{get, web, HttpRequest};
use sqlx::PgPool;

use crate::{
    model::{History, HistoryVec, LogUserAction, UserRole, UserToken},
    ApiResult,
};

#[get("/history")]
async fn find_all_transactions(
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<HistoryVec> {
    let _user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref())
        .await?
        .log_action(format!("find all transactions"), db_pool.get_ref())
        .await?;

    History::find_all(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_transactions);
}
