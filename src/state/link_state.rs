use bitflags::Flags;

use crate::core_api::link;
use crate::core_api::link::events::LinkEventsBuilder;
use crate::core_api::link::info::LinkInfo;
use crate::listeners::OwnListeners;
use crate::state::{Message, State};

#[derive(Debug)]
pub enum LinkMessage {
    Added(u32),
    Removed(u32),

    Info(u32),
    Format(u32),
    State(u32, link::info::LinkState),
    Props(u32),
}

impl<'a> State<'a> {
    pub fn subscribe_link_changes(&self, id: u32) {
        if let Some(link) = self.links.lock().unwrap().get_mut(&id) {
            let listener = LinkEventsBuilder::default()
                .info(Box::new({
                    let self_ = self.clone();
                    let link = link.clone();
                    move |info| {
                        self_
                            .links_info
                            .lock()
                            .unwrap()
                            .insert(id, LinkInfo::from_ref(info));
                        self_.send_message(Message::Link(LinkMessage::Info(id)));
                        let change_mask = info.change_mask();
                        if change_mask.contains(link::info::ChangeMask::FORMAT) {
                            self_.send_message(Message::Link(LinkMessage::Format(id)));
                        }
                        if change_mask.contains(link::info::ChangeMask::STATE) {
                            self_.send_message(Message::Link(LinkMessage::State(id, info.state())));
                        }
                        if change_mask.contains(link::info::ChangeMask::PROPS) {
                            self_.send_message(Message::Link(LinkMessage::Props(id)));
                        }
                    }
                }))
                .build();
            link.add_listener(listener);
        }
    }

    pub fn remove_link(&self, id: u32) {
        if let Some(link) = &self.links.lock().unwrap().remove(&id) {
            self.links_info.lock().unwrap().remove(&id);
            self.send_message(Message::Link(LinkMessage::Removed(id)));
        }
    }
}
