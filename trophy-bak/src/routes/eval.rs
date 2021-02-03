use actix_files::NamedFile;
use actix_web::{delete, get, post, put, web, Error, HttpResponse, Responder};
use anyhow::Result;
use sqlx::PgPool;
use std::path::PathBuf;

use crate::eval::{create_xlsx_file, evaluate_trophy};

// TODO this does not work with the anyhow-result => converting to std::Result would be difficult
// TODO -> separate methods in /eval and /eval/xlsx
// #[get("/eval")]
// async fn evaluate(db_pool: web::Data<PgPool>) -> Responder {
//     info!("Received new request: evaluate game.");

//     evaluate_trophy(db_pool.get_ref()).await?;
//     create_xlsx_file().await;
//     let path: PathBuf = PathBuf::from("test");
//     Ok(NamedFile::open(path)?)
// }

pub fn init(cfg: &mut web::ServiceConfig) {
    // cfg.service(evaluate);
}
