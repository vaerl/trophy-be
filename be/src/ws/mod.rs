use actix_web::web;

pub mod lobby;
pub mod messages;
pub mod route;
pub mod socket_refresh;
pub mod ws;

pub fn init(cfg: &mut web::ServiceConfig) {
    route::init(cfg);
}
