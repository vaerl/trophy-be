mod error;
mod game;
mod history;
mod outcome;
mod parsed_outcome;
mod team;
mod user;
mod user_token;

use actix_web::{HttpRequest, HttpResponse, Responder};
pub use error::*;
use futures::future::{ready, Ready};
pub use game::*;
pub use history::*;
pub use outcome::*;
pub use parsed_outcome::*;
use serde::Serialize;
pub use team::*;
pub use user::*;
pub use user_token::*;

use crate::ApiResult;

#[derive(Serialize)]
pub struct Amount(pub usize);

impl Responder for Amount {
    type Error = CustomError;
    type Future = Ready<ApiResult<HttpResponse>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();
        // create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}
