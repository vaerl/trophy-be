#[macro_use]
extern crate log;

extern crate derive_responder;

use actix_web::{body::Body, error, web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use model::CustomError;
use sqlx::PgPool;
use std::env;

mod eval;
mod model;
mod routes;

pub type ApiResult<T> = Result<T, CustomError>;

// TODO routes: user

// TODO log transactions in the transaction table
// - maybe use services

#[actix_web::main]
async fn main() -> Result<(), CustomError> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file!");
    let db_pool = PgPool::connect(&database_url).await?;
    // let pool_clone = db_pool.clone();

    let host = env::var("HOST").expect("HOST is not set in .env file!");
    let port = env::var("PORT").expect("PORT is not set in .env file!");

    let server = HttpServer::new(move || {
        App::new()
            // pass database pool to application so we can access it inside handlers
            .data(db_pool.clone())
            .configure(routes::init)
            // return JSON-parse-errors
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .body(Body::from(format!(r#"{{"error":"{}"}}"#, err))),
                )
                .into()
            }))
    })
    .bind(format!("{}:{}", host, port))?;

    // NOTE this needs to be commented, because it errs when the user exists
    // I'm leaving this here in case I have to reset the database - which I most certainly will.
    // info!("Creating admin-user.");
    // User::create(
    //     CreateUser {
    //         username: "lukas".to_string(),
    //         password: "test".to_string(),
    //         role: UserRole::Admin,
    //     },
    //     &pool_clone,
    // )
    // .await?;

    info!("Starting server.");
    server.run().await?;

    Ok(())
}
