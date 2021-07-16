use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest};
use sqlx::PgPool;

use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    model::{Log, UserRole, UserToken},
    ApiResult,
};

// CHECK if calling multiple times evaluates again -> in this case I should separate or use some flag
#[get("/eval")]
async fn evaluate(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<NamedFile> {
    let user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Referee],
        db_pool.get_ref(),
    )
    .await?;
    evaluate_trophy(db_pool.get_ref()).await?;
    let file = create_xlsx_file(db_pool.get_ref())
        .await?
        .log_info(user.id, format!("evaluate trophy"), db_pool.get_ref())
        .await?
        .0;
    Ok(file)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
}
