#[macro_use]
extern crate log;

use actix_web::{error, web, App, HttpResponse, HttpServer};
use anyhow::Result;
use dotenv::dotenv;
use sqlx::PgPool;
use std::env;

mod model;
mod routes;

// TODO improve error-handling in routes
// TODO routes
// TODO check for ids?
// TODO use metadata-table for storing number of entries, etc.?

#[actix_web::main]
async fn main() -> Result<()> {
    dotenv().ok();
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    let db_pool = PgPool::connect(&database_url).await?;

    let host = env::var("HOST").expect("HOST is not set in .env file");
    let port = env::var("PORT").expect("PORT is not set in .env file");

    let server = HttpServer::new(move || {
        App::new()
            .data(db_pool.clone()) // pass database pool to application so we can access it inside handlers
            .configure(routes::init) // init routes
            // return JSON-parse-errors
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                error::InternalError::from_response(
                    "",
                    HttpResponse::BadRequest()
                        .content_type("application/json")
                        .body(format!(r#"{{"error":"{}"}}"#, err)),
                )
                .into()
            }))
    })
    .bind(format!("{}:{}", host, port))?;

    info!("Starting server");
    server.run().await?;

    Ok(())
}
