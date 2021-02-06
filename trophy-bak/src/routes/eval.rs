use actix_files::NamedFile;
use actix_web::{get, web};
use sqlx::PgPool;

use crate::eval::{create_xlsx_file, evaluate_trophy};

// CHECK if calling multiple times evaluates again -> in this case I should separate or use some flag
#[get("/eval")]
async fn evaluate(db_pool: web::Data<PgPool>) -> actix_web::Result<NamedFile> {
    info!("Received new request: evaluate trophy.");

    evaluate_trophy(db_pool.get_ref())
        .await
        .to_internal_error()?;
    create_xlsx_file(db_pool.get_ref())
        .await
        .to_internal_error()
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate);
}

// CHECK if this works
trait ConvertToActix<T> {
    fn to_internal_error(self) -> actix_web::Result<T>;
}

impl<T> ConvertToActix<T> for anyhow::Result<T> {
    fn to_internal_error(self) -> actix_web::Result<T> {
        self.map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))
    }
}
