use actix_files::NamedFile;
use actix_web::{
    get,
    web::{self, Data},
    HttpResponse, Responder,
};
use sqlx::PgPool;

use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    middleware::Authenticated,
    model::{StatusResponse, UserRole},
    ApiResult, ToJson,
};

#[get("/eval")]
async fn evaluate(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    evaluate_trophy(&pool).await?;
    Ok(HttpResponse::Ok())
}

#[get("/eval/sheet")]
async fn download_sheet(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<NamedFile> {
    auth.has_roles(vec![UserRole::Admin])?;
    Ok(create_xlsx_file(&pool).await?.0)
}

#[get("/eval/done")]
async fn is_evaluated(pool: Data<PgPool>, auth: Authenticated) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    let status = crate::eval::is_evaluated(&pool).await?;
    StatusResponse { status }.to_json()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
    cfg.service(download_sheet);
    cfg.service(is_evaluated);
}
