use crate::{
    middleware::Authenticated,
    model::{ImportTeam, ImportUpload, Team, UserRole},
    ApiResult,
};
use actix_multipart::form::MultipartForm;
use actix_web::{
    post,
    web::{self, Data},
    HttpResponse, Responder,
};
use calamine::{open_workbook, RangeDeserializerBuilder, Reader, Xlsx};
use sqlx::PgPool;

#[post("/import")]
async fn import_sheet(
    pool: Data<PgPool>,
    auth: Authenticated,
    MultipartForm(form): MultipartForm<ImportUpload>,
) -> ApiResult<impl Responder> {
    auth.has_roles(vec![UserRole::Admin])?;
    read_sheet(form, &pool).await?;
    Ok(HttpResponse::Ok())
}

/// Import an Excel-file.
async fn read_sheet(form: ImportUpload, pool: &PgPool) -> ApiResult<()> {
    let mut workbook: Xlsx<_> = open_workbook(form.file.file.path())?;
    let range = workbook.worksheet_range(&form.metadata.sheet_name)?;

    let iter_records = RangeDeserializerBuilder::with_headers(&[
        &form.metadata.trophy_id_header,
        &form.metadata.name_header,
        &form.metadata.gender_header,
    ])
    .from_range(&range)?;

    for result in iter_records {
        let team: ImportTeam = result?;
        Team::create(team.with_year(form.metadata.year), &pool).await?;
    }

    Ok(())
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(import_sheet);
}
