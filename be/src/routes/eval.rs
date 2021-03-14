use actix_files::NamedFile;
use actix_web::{get, web};
use sqlx::PgPool;

use crate::{
    eval::{create_xlsx_file, evaluate_trophy},
    model::EvaluationError,
};

// CHECK if calling multiple times evaluates again -> in this case I should separate or use some flag
#[get("/eval")]
async fn evaluate(db_pool: web::Data<PgPool>) -> actix_web::Result<NamedFile, EvaluationError> {
    info!("Received new request: evaluate trophy.");
    evaluate_trophy(db_pool.get_ref()).await?;
    create_xlsx_file(db_pool.get_ref()).await
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
}
