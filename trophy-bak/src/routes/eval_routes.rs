use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::PgPool;

#[post("/eval/{game_id}")]
async fn evaluate_game(id: web::Path<i32>, db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: evaluate game.");
    // TODO check status for game -> only continue if done!
    // TODO get all results for game
    // TODO separate game_results by gender
    // TODO get list of all teams
    // TODO add points after place in list: 1. -> 50P, 2 -> 49., ...
    // -> do this separately for every gender!
    // NOTE use constant for now, might need to be changed later
    
    // TODO find result-strategy: 
    // a: create every n:m-relation, leave result at NULL -> I'm currently doing this!
    // b: create relation if result is saved
    let result = Team::find_all(db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all teams from database: {}",
            err
        )),
    }
}

#[get("/eval/file")]
async fn get_file(db_pool: web::Data<PgPool>) -> impl Responder {
    info!("Received new request: serve file.");
    // TODO get list of all teams
    // TODO separate list by gender -> implement in team.rs
    // TODO sort list by points
    // TODO build xlsx-file with three tabs -> name tabs!
    let result = Team::find_all(db_pool.get_ref()).await;
    match result {
        Ok(teams) => HttpResponse::Ok().json(teams),
        Err(err) => HttpResponse::BadRequest().body(format!(
            "Error trying to read all teams from database: {}",
            err
        )),
    }
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(evaluate_game);
    cfg.service(get_file);
}
