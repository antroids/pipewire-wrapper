use bitflags::Flags;

use crate::core_api::client;
use crate::core_api::client::events::ClientEventsBuilder;
use crate::core_api::client::info::ClientInfo;
use crate::listeners::OwnListeners;
use crate::state::{Message, State};

#[derive(Debug, Clone)]
pub enum ClientMessage {
    Added(u32),
    Removed(u32),

    Info(u32),
    Props(u32),
}

impl State {
    pub fn subscribe_client_changes(&self, id: u32) {
        if let Some(client) = self.clients.lock().unwrap().get_mut(&id) {
            let listener = ClientEventsBuilder::default()
                .info(Box::new({
                    let self_ = self.clone();
                    let client = client.clone();
                    move |info| {
                        self_
                            .clients_info
                            .lock()
                            .unwrap()
                            .insert(id, ClientInfo::from_ref(info));
                        self_.send_message(Message::Client(ClientMessage::Info(id)));
                        let change_mask = info.change_mask();
                        if change_mask.contains(client::info::ChangeMask::PROPS) {
                            self_.send_message(Message::Client(ClientMessage::Props(id)));
                        }
                    }
                }))
                .build();
            client.add_listener(listener);
        }
    }

    pub fn remove_client(&self, id: u32) {
        if let Some(client) = &self.clients.lock().unwrap().remove(&id) {
            self.clients_info.lock().unwrap().remove(&id);
            self.send_message(Message::Client(ClientMessage::Removed(id)));
        }
    }
}
