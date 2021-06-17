use actix_web::web;

mod eval;
mod game;
mod logger;
mod misc;
mod outcome;
mod team;
mod user;

pub fn init(cfg: &mut web::ServiceConfig) {
    misc::init(cfg);
    game::init(cfg);
    team::init(cfg);
    outcome::init(cfg);
    eval::init(cfg);
    user::init(cfg);
    logger::init(cfg);
}
