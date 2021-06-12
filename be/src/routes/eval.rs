use actix_files::NamedFile;
use actix_web::{get, web};
use sqlx::PgPool;

use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    model::{History, UserRole, UserToken},
    ApiResult,
};

// CHECK if calling multiple times evaluates again -> in this case I should separate or use some flag
#[get("/eval")]
async fn evaluate(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<NamedFile> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;
    History::important(user.id, format!("evaluate trophy"), db_pool.get_ref()).await?;
    evaluate_trophy(db_pool.get_ref()).await?;
    create_xlsx_file(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
}
