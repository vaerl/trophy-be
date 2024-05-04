use actix::Addr;
use actix_web::{
    get,
    web::{self, Data, Payload},
    HttpRequest, HttpResponse,
};
use actix_web_actors::ws;
use sqlx::PgPool;

use crate::{
    model::{LogUserAction, UserRole, UserToken},
    ApiResult,
};

use super::{lobby::Lobby, ws::WsConn};

// NOTE to use this socket, forward the app's port in VSCode!
#[get("/socket")]
pub async fn start_connection(
    req: HttpRequest,
    stream: Payload,
    db_pool: Data<PgPool>,
    srv: Data<Addr<Lobby>>,
) -> ApiResult<HttpResponse> {
    let _user = UserToken::try_into_authorized_user(
        &req,
        vec![UserRole::Admin, UserRole::Referee, UserRole::Visualizer],
        &db_pool,
    )
    .await?
    .log_action(format!("connected to socket."), &db_pool);
    let ws = WsConn::new(srv.get_ref().clone());

    let resp = ws::start(ws, &req, stream)?;
    Ok(resp)
}

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(start_connection);
}
