#[macro_use]
extern crate log;

use actix::Actor;
use actix_cors::Cors;
use actix_web::{
    error::{self, InternalError, JsonPayloadError},
    web::{self, Data},
    App, HttpResponse, HttpServer,
};
use dotenv::dotenv;
use model::CustomError;
use serde::Serialize;
use sqlx::{PgPool, Pool, Postgres};
use std::env;

use crate::{
    model::{CreateUser, User},
    ws::lobby::Lobby,
};

mod eval;
mod model;
mod routes;
mod ws;

#[actix_web::main]
async fn main() -> Result<(), CustomError> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "DEBUG");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file!");
    let db_pool = Data::new(PgPool::connect(&database_url).await?);
    let db_pool_clone = db_pool.clone();

    let host = env::var("HOST").expect("HOST is not set in .env file!");
    let port = env::var("PORT").expect("PORT is not set in .env file!");
    let origin = env::var("CORS_ORIGIN").expect("CORS_ORIGIN is not set in .env file!");

    let ws_server = Data::new(Lobby::default().start());

    let server = HttpServer::new(move || {
        // more here: https://docs.rs/actix-cors/latest/actix_cors/index.html
        let cors = Cors::default()
            .allow_any_header()
            .supports_credentials()
            // NOTE this may need to change once we implement apps
            .allowed_origin(&origin)
            .allow_any_method()
            .max_age(3600);

        App::new()
            // pass database pool to application so we can access it inside handlers
            .wrap(cors)
            .app_data(db_pool.clone())
            .configure(routes::init)
            .configure(ws::init)
            // return JSON-parse-errors
            .app_data(
                web::JsonConfig::default()
                    .error_handler(|err, _req| verbose_json_error(err).into()),
            )
            .app_data(ws_server.clone())
    })
    .bind(format!("{}:{}", host, port))?;

    create_admin_user(db_pool_clone).await?;

    info!("Starting server.");
    server.run().await?;

    Ok(())
}

// doing this here is easier to read
fn verbose_json_error(err: JsonPayloadError) -> InternalError<String> {
    error::InternalError::from_response(
        "".to_string(),
        HttpResponse::BadRequest()
            .body(format!("Error while parsing: {}", err))
            .into(),
    )
}

async fn create_admin_user(pool: Data<Pool<Postgres>>) -> ApiResult<()> {
    let admin_name = env::var("ADMIN_NAME").expect("ADMIN_NAME is not set in .env file!");
    let admin_password =
        env::var("ADMIN_PASSWORD").expect("ADMIN_PASSWORD is not set in .env file!");

    match User::find_by_name(&admin_name, &pool).await {
        Ok(_) => {
            info!("Not creating a new admin-user, because a user of that name exists.");
            Ok(())
        }
        Err(_) => {
            info!("Creating admin-user, because it does not exist yet.");
            User::create(
                CreateUser {
                    name: admin_name,
                    password: admin_password,
                    role: model::UserRole::Admin,
                    game_id: None,
                },
                &pool,
            )
            .await?;
            Ok(())
        }
    }
}

// NOTE using this would be nicer (tracking issue: https://github.com/rust-lang/rust/issues/63063):
// pub type ApiResult = Result<impl Responder, CustomError>;
pub type ApiResult<T> = Result<T, CustomError>;

pub trait ToJson<T> {
    fn to_json(self) -> ApiResult<web::Json<T>>;
}

impl<T> ToJson<T> for T
where
    T: Serialize,
{
    fn to_json(self) -> ApiResult<web::Json<T>> {
        Ok(web::Json(self))
    }
}

pub trait TypeInfo {
    fn type_name(&self) -> String;
}
