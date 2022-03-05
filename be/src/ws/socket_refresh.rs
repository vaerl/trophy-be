use std::fmt::Display;

use actix::Addr;
use actix_web::web::Json;

use crate::{ApiResult, TypeInfo};

use super::{lobby::Lobby, messages::ClientActorMessage};
use serde::Serialize;

#[derive(Serialize)]
pub struct SocketRefresh {
    kind: String,
}

impl SocketRefresh {
    fn send_socket_refresh(lobby: &Addr<Lobby>, kind: String) -> ApiResult<()> {
        lobby.try_send(ClientActorMessage {
            msg: Json::<SocketRefresh>(SocketRefresh { kind }).to_string(),
        })?;
        Ok(())
    }
}

impl Display for SocketRefresh {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SocketRefresh(kind: {})", self.kind)
    }
}

pub trait SendRefresh<T: Serialize> {
    fn send_refresh(self, lobby: &Addr<Lobby>) -> ApiResult<T>;
}

impl<T> SendRefresh<T> for T
where
    T: TypeInfo + Serialize,
{
    fn send_refresh(self, lobby: &Addr<Lobby>) -> ApiResult<T> {
        SocketRefresh::send_socket_refresh(lobby, self.type_name())?;
        Ok(self)
    }
}
