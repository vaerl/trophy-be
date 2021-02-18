mod eval;
mod game;
mod misc;
mod outcome;
mod team;

use actix_web::web;

pub fn init(cfg: &mut web::ServiceConfig) {
    misc::init(cfg);
    game::init(cfg);
    team::init(cfg);
    outcome::init(cfg);
    eval::init(cfg);
}
