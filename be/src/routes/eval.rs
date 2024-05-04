use actix_files::NamedFile;
use actix_web::{get, web, HttpRequest, HttpResponse, Responder};
use sqlx::PgPool;

use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    model::{History, Log, LogLevel, UserRole, UserToken},
    ApiResult,
};

#[get("/eval")]
async fn evaluate(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<impl Responder> {
    let user =
        UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], db_pool.get_ref()).await?;
    let action = format!("User {} executed: evaluate trophy", user.id);
    // NOTE rather than implementing Log for () (if even possible), I decided to just update history "manually"
    History::create(user.id, LogLevel::Info, action, &db_pool).await?;
    evaluate_trophy(&db_pool).await?;
    Ok(HttpResponse::Ok())
}

#[get("/eval/sheet")]
async fn download_sheet(req: HttpRequest, db_pool: web::Data<PgPool>) -> ApiResult<NamedFile> {
    let user = UserToken::try_into_authorized_user(&req, vec![UserRole::Admin], &db_pool).await?;
    let file = create_xlsx_file(&db_pool)
        .await?
        .log_info(user.id, format!("download sheet"), &db_pool)
        .await?
        .0;
    Ok(file)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
    cfg.service(download_sheet);
}
