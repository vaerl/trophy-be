use actix::prelude::{Actor, Context, Handler, Recipient};
use std::collections::HashMap;
use uuid::Uuid;

use super::messages::{ClientActorMessage, Connect, Disconnect, WsMessage};

type Socket = Recipient<WsMessage>;

/*
/ NOTE
/ The Lobby was not really creating rooms in the sense of separate structs.
/ It just created entries in a HashMap - this was later used to map incoming requests to their respective "rooms".
/ Moving away from this architecture only required adding all users to a single room!
*/

pub struct Lobby {
    sessions: HashMap<Uuid, Socket>, // save the clients id and address
}

impl Default for Lobby {
    fn default() -> Lobby {
        Lobby {
            sessions: HashMap::new(),
        }
    }
}

impl Lobby {
    /// I send messages on a user-based method. Thus I need this helper to reach everybody.
    /// Furthermore, this also supports sending specific messages.
    fn send_message(&self, message: &str, id_to: &Uuid) {
        debug!("Sending message '{}' to {}", message, id_to);
        if let Some(socket_recipient) = self.sessions.get(id_to) {
            let _ = socket_recipient.do_send(WsMessage(message.to_owned()));
        } else {
            warn!("Attempted to send a message to an unknown user {}.", id_to);
        }
    }

    /// Send a message to every registered user. Uses [send_message()](send_message()).
    fn send_message_to_all(&self, message: &str) {
        self.sessions
            .iter()
            .for_each(|(user_id, _)| self.send_message(message, user_id));
    }
}

impl Actor for Lobby {
    type Context = Context<Self>;
}

impl Handler<Disconnect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        if self.sessions.remove(&msg.id).is_some() {
            self.send_message_to_all(&format!("{} disconnected.", &msg.id))
        }
    }
}

impl Handler<Connect> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) -> Self::Result {
        // notify about user joining
        self.send_message_to_all(&format!("{} just joined!", msg.self_id));

        // then add the user afterwards
        self.sessions.insert(msg.self_id, msg.addr);

        self.send_message(&format!("Your id is {}", msg.self_id), &msg.self_id);
    }
}

impl Handler<ClientActorMessage> for Lobby {
    type Result = ();

    fn handle(&mut self, msg: ClientActorMessage, _ctx: &mut Context<Self>) -> Self::Result {
        self.send_message_to_all(&msg.msg);
    }
}
