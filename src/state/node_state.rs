use std::collections::HashMap;

use bitflags::Flags;

use crate::core_api::node;
use crate::core_api::node::events::NodeEventsBuilder;
use crate::core_api::node::info::NodeInfo;
use crate::core_api::node::NodeRef;
use crate::core_api::proxy::Proxied;
use crate::listeners::OwnListeners;
use crate::spa::param::ParamType;
use crate::spa::pod::{BasicType, ToOwnedPod};
use crate::state::{Message, State};

#[derive(Debug, Clone)]
pub enum NodeMessage {
    Added(u32),
    Removed(u32),

    Info(u32),
    Param(u32, ParamType),
    InputPorts(u32),
    OutputPorts(u32),
    State(u32, node::info::NodeState),
    Props(u32),
}

impl<'a> State<'a> {
    pub fn subscribe_node_changes(&self, id: u32) {
        if let Some(node) = self.nodes.lock().unwrap().get_mut(&id) {
            let listener = NodeEventsBuilder::default()
                .info(Box::new({
                    let self_ = self.clone();
                    let node = node.clone();
                    move |info| {
                        self_
                            .nodes_info
                            .lock()
                            .unwrap()
                            .insert(id, NodeInfo::from_ref(info));
                        self_.send_message(Message::Node(NodeMessage::Info(id)));
                        let change_mask = info.change_mask();
                        if let Some(params_subscriptions) =
                            self_.params_subscriptions.get(&NodeRef::type_info())
                        {
                            if change_mask.contains(node::info::ChangeMask::PARAMS) {
                                let param_types: Vec<ParamType> = info
                                    .params()
                                    .iter()
                                    .filter_map(|p| {
                                        params_subscriptions.contains(&p.id()).then(|| p.id())
                                    })
                                    .collect();
                                node.subscribe_params(param_types.as_slice());
                            }
                        }
                        if change_mask.contains(node::info::ChangeMask::INPUT_PORTS) {
                            self_.send_message(Message::Node(NodeMessage::InputPorts(id)));
                        }
                        if change_mask.contains(node::info::ChangeMask::OUTPUT_PORTS) {
                            self_.send_message(Message::Node(NodeMessage::OutputPorts(id)));
                        }
                        if change_mask.contains(node::info::ChangeMask::STATE) {
                            self_.send_message(Message::Node(NodeMessage::State(id, info.state())));
                        }
                        if change_mask.contains(node::info::ChangeMask::PROPS) {
                            self_.send_message(Message::Node(NodeMessage::Props(id)));
                        }
                    }
                }))
                .param(Box::new({
                    let self_ = self.clone();
                    move |_, type_, _, _, pod| {
                        if let Ok(BasicType::OBJECT(obj)) = pod.downcast() {
                            if let Ok(owned) = obj.to_owned_pod() {
                                self_
                                    .nodes_params
                                    .lock()
                                    .unwrap()
                                    .entry(id)
                                    .and_modify(|params| {
                                        params.insert(type_, owned);
                                    })
                                    .or_insert(HashMap::default());
                                self_.send_message(Message::Node(NodeMessage::Param(id, type_)));
                            }
                        }
                    }
                }))
                .build();
            node.add_listener(listener);
        }
    }

    pub fn remove_node(&self, id: u32) {
        if let Some(node) = &self.nodes.lock().unwrap().remove(&id) {
            self.nodes_info.lock().unwrap().remove(&id);
            self.nodes_params.lock().unwrap().remove(&id);
            self.send_message(Message::Node(NodeMessage::Removed(id)));
        }
    }
}
