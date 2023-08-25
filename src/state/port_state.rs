use std::collections::HashMap;

use bitflags::Flags;

use crate::core_api::port;
use crate::core_api::port::events::PortEventsBuilder;
use crate::core_api::port::info::PortInfo;
use crate::core_api::port::PortRef;
use crate::core_api::proxy::Proxied;
use crate::listeners::OwnListeners;
use crate::spa::param::ParamType;
use crate::spa::pod::{BasicType, ToOwnedPod};
use crate::state::{Message, State};

#[derive(Debug, Clone)]
pub enum PortMessage {
    Added(u32),
    Removed(u32),

    Info(u32),
    Param(u32, ParamType),
    Props(u32),
}

impl State {
    pub fn subscribe_port_changes(&self, id: u32) {
        if let Some(port) = self.ports.lock().unwrap().get_mut(&id) {
            let listener = PortEventsBuilder::default()
                .info(Box::new({
                    let self_ = self.clone();
                    let port = port.clone();
                    move |info| {
                        self_
                            .ports_info
                            .lock()
                            .unwrap()
                            .insert(id, PortInfo::from_ref(info));
                        self_.send_message(Message::Port(PortMessage::Info(id)));
                        let change_mask = info.change_mask();
                        if let Some(params_subscriptions) =
                            self_.params_subscriptions.get(&PortRef::type_info())
                        {
                            if change_mask.contains(port::info::ChangeMask::PARAMS) {
                                let param_types: Vec<ParamType> = info
                                    .params()
                                    .iter()
                                    .filter_map(|p| {
                                        params_subscriptions.contains(&p.id()).then(|| p.id())
                                    })
                                    .collect();
                                port.subscribe_params(param_types.as_slice());
                            }
                        }
                        if change_mask.contains(port::info::ChangeMask::PROPS) {
                            self_.send_message(Message::Port(PortMessage::Props(id)));
                        }
                    }
                }))
                .param(Box::new({
                    let self_ = self.clone();
                    move |_, type_, _, _, pod| {
                        if let Ok(BasicType::OBJECT(obj)) = pod.downcast() {
                            if let Ok(owned) = obj.to_owned_pod() {
                                self_
                                    .ports_params
                                    .lock()
                                    .unwrap()
                                    .entry(id)
                                    .and_modify(|params| {
                                        params.insert(type_, owned);
                                    })
                                    .or_insert(HashMap::default());
                                self_.send_message(Message::Port(PortMessage::Param(id, type_)));
                            }
                        }
                    }
                }))
                .build();
            port.add_listener(listener);
        }
    }

    pub fn remove_port(&self, id: u32) {
        if let Some(port) = &self.ports.lock().unwrap().remove(&id) {
            self.ports_info.lock().unwrap().remove(&id);
            self.ports_params.lock().unwrap().remove(&id);
            self.send_message(Message::Port(PortMessage::Removed(id)));
        }
    }
}
