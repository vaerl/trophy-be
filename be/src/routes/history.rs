use actix_web::{get, web, HttpRequest, Responder};
use sqlx::PgPool;

use crate::{
    model::{History, LogUserAction, UserRole, UserToken},
    ApiResult, ToJson,
};

// TODO paginate
#[get("/history")]
async fn find_all_transactions(
    req: HttpRequest,
    db_pool: web::Data<PgPool>,
) -> ApiResult<impl Responder> {
    let _user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool)
        .await?
        .log_action(format!("find all transactions"), &db_pool)
        .await?;

    History::find_all(&db_pool).await?.to_json()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(find_all_transactions);
}
