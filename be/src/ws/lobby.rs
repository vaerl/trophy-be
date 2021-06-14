use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use super::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};

type Socket = Recipient<WsMessage>;

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, // self_id to self
    room: HashSet<Uuid>,             // all users
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
            room: HashSet::new(),
        }
    }
}

impl Lobby {
    fn send_message(&self, message: &str, id_to: &Uuid) {
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            warn!("Attempted to send a message to an unknown user {}.", id_to);
        }
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.room.iter().for_each(|user_id| {
                self.send_message(&format!("{} disconnected.", &msg.id), user_id)
            });
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        self.room.insert(msg.self_id);

        self.room.iter().for_each(|conn_id| {
            self.send_message(&format!("{} just joined!", msg.self_id), conn_id)
        });

        self.sessions.insert(msg.self_id, msg.addr);

        self.send_message(&format!("Your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.room
            .iter()
            .for_each(|client| self.send_message(&msg.msg, client));
    }
}
