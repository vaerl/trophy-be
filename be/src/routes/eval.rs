use std::fs::File;

use actix_files::NamedFile;
use actix_web::{body::Body, get, web, BaseHttpResponse, HttpRequest, HttpResponse, Responder};
use serde::Serialize;
use sqlx::PgPool;

use crate::derive_responder::Responder;
use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    model::{CustomError, Game, History, User, UserRole, UserToken},
    ApiResult,
};

// CHECK if calling multiple times evaluates again -> in this case I should separate or use some flag
// #[get("/eval")]
// async fn evaluate(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<()> {
//     let user = token
//         .try_into_authorized_user(vec![UserRole::Admin], db_pool.get_ref())
//         .await?;
//     History::log(user.id, format!("evaluate trophy"), db_pool.get_ref()).await?;

//     // evaluate_trophy(db_pool.get_ref()).await?;
//     // create_xlsx_file(db_pool.get_ref()).await

//     todo!()
// }

#[derive(Serialize, Responder)]
pub struct Res {
    pub file: u128,
}

#[get("/eval")]
async fn evaluate(token: UserToken, db_pool: web::Data<PgPool>) -> ApiResult<> {
    let user = token
        .try_into_authorized_user(vec![UserRole::Admin, UserRole::Referee], db_pool.get_ref())
        .await?;
    History::log(user.id, format!("evaluate trophy"), db_pool.get_ref()).await?;
    evaluate_trophy(db_pool.get_ref()).await?;
    let x = create_xlsx_file(db_pool.get_ref()).await?;
    todo!()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
}
