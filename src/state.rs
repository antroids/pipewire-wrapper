use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use bitflags::Flags;

use node_state::NodeMessage;

use crate::core_api::client::info::ClientInfo;
use crate::core_api::client::{Client, ClientRef};
use crate::core_api::context::Context;
use crate::core_api::core::Core;
use crate::core_api::device::info::DeviceInfo;
use crate::core_api::device::{Device, DeviceRef};
use crate::core_api::link::info::LinkInfo;
use crate::core_api::link::{Link, LinkRef};
use crate::core_api::node::info::NodeInfo;
use crate::core_api::node::{Node, NodeRef};
use crate::core_api::port::info::PortInfo;
use crate::core_api::port::{Port, PortRef};
use crate::core_api::proxy::Proxied;
use crate::core_api::registry::events::RegistryEventsBuilder;
use crate::core_api::registry::Registry;
use crate::core_api::type_info::TypeInfo;
use crate::listeners::{AddListener, OwnListeners};
use crate::spa::param::ParamType;
use crate::spa::pod::object::PodObjectRef;
use crate::spa::pod::pod_buf::AllocPod;
use crate::spa::pod::ToOwnedPod;
use crate::state::client_state::ClientMessage;
use crate::state::device_state::DeviceMessage;
use crate::state::link_state::LinkMessage;
use crate::state::port_state::PortMessage;

mod client_state;
mod device_state;
mod link_state;
mod node_state;
mod port_state;

#[derive(Debug, Clone)]
pub enum Message {
    GlobalAdded(u32),

    Node(NodeMessage),
    Port(PortMessage),
    Link(LinkMessage),
    Device(DeviceMessage),
    Client(ClientMessage),
}

type ObjectsMap<T> = Arc<Mutex<HashMap<u32, T>>>;
type ObjectsInfoMap<T> = ObjectsMap<T>;
type ObjectsParamsMap = Arc<Mutex<HashMap<u32, HashMap<ParamType, AllocPod<PodObjectRef>>>>>;

#[derive(Debug, Clone)]
pub struct State<'a> {
    core: Rc<Core>,
    context: Context,

    registry: Registry<'a>,

    subscriptions: Vec<TypeInfo<'static>>,
    params_subscriptions: HashMap<TypeInfo<'static>, Vec<ParamType>>,

    messages_sender: Option<crossbeam_channel::Sender<Message>>,

    nodes: ObjectsMap<Node<'a>>,
    nodes_info: ObjectsInfoMap<NodeInfo>,
    nodes_params: ObjectsParamsMap,

    ports: ObjectsMap<Port<'a>>,
    ports_info: ObjectsInfoMap<PortInfo>,
    ports_params: ObjectsParamsMap,

    links: ObjectsMap<Link<'a>>,
    links_info: ObjectsInfoMap<LinkInfo>,

    devices: ObjectsMap<Device<'a>>,
    devices_info: ObjectsInfoMap<DeviceInfo>,
    devices_params: ObjectsParamsMap,

    clients: ObjectsMap<Client<'a>>,
    clients_info: ObjectsInfoMap<ClientInfo>,
}

impl<'a> State<'a> {
    pub fn new(
        core: Rc<Core>,
        context: Context,

        registry: Registry<'a>,
        subscriptions: Vec<TypeInfo<'static>>,
        params_subscriptions: HashMap<TypeInfo<'static>, Vec<ParamType>>,
    ) -> Self {
        Self {
            core,
            context,

            registry,

            subscriptions,
            params_subscriptions,

            messages_sender: None,

            nodes: Default::default(),
            nodes_info: Default::default(),
            nodes_params: Default::default(),

            ports: Default::default(),
            ports_info: Default::default(),
            ports_params: Default::default(),

            links: Default::default(),
            links_info: Default::default(),

            devices: Default::default(),
            devices_info: Default::default(),
            devices_params: Default::default(),

            clients: Default::default(),
            clients_info: Default::default(),
        }
    }

    pub fn create_channel(&mut self) -> crossbeam_channel::Receiver<Message> {
        let (sender, receiver) = crossbeam_channel::unbounded::<Message>();
        self.messages_sender = Some(sender);
        receiver
    }

    pub fn subscribe_changes(&mut self) {
        let listener = RegistryEventsBuilder::default()
            .global(Box::new({
                let self_ = self.clone();
                move |id, _permissions, type_, version, _props| {
                    self_.send_message(Message::GlobalAdded(id));
                    if self_.subscriptions.contains(&type_) {
                        if type_ == NodeRef::type_info() {
                            if let Ok(obj) = self_.registry.bind_proxy::<Node>(id, version) {
                                self_.nodes.lock().unwrap().insert(id, obj);
                                self_.send_message(Message::Node(NodeMessage::Added(id)));
                                self_.subscribe_node_changes(id);
                            }
                        } else if type_ == PortRef::type_info() {
                            if let Ok(obj) = self_.registry.bind_proxy::<Port>(id, version) {
                                self_.ports.lock().unwrap().insert(id, obj);
                                self_.send_message(Message::Port(PortMessage::Added(id)));
                                self_.subscribe_port_changes(id);
                            }
                        } else if type_ == LinkRef::type_info() {
                            if let Ok(obj) = self_.registry.bind_proxy::<Link>(id, version) {
                                self_.links.lock().unwrap().insert(id, obj);
                                self_.send_message(Message::Link(LinkMessage::Added(id)));
                                self_.subscribe_link_changes(id);
                            }
                        } else if type_ == DeviceRef::type_info() {
                            if let Ok(obj) = self_.registry.bind_proxy::<Device>(id, version) {
                                self_.devices.lock().unwrap().insert(id, obj);
                                self_.send_message(Message::Device(DeviceMessage::Added(id)));
                                self_.subscribe_device_changes(id);
                            }
                        } else if type_ == ClientRef::type_info() {
                            if let Ok(obj) = self_.registry.bind_proxy::<Client>(id, version) {
                                self_.clients.lock().unwrap().insert(id, obj);
                                self_.send_message(Message::Client(ClientMessage::Added(id)));
                                self_.subscribe_client_changes(id);
                            }
                        }
                    }
                }
            }))
            .global_remove(Box::new({
                let self_ = self.clone();
                move |id| {}
            }))
            .build();
        self.registry.add_listener(listener);
    }

    fn remove_global_object(&self, id: u32) {
        self.remove_node(id);
        self.remove_port(id);
        self.remove_link(id);
        self.remove_device(id);
        self.remove_client(id);
    }

    fn send_message(&self, message: Message) {
        if let Some(sender) = &self.messages_sender {
            sender.send(message);
        }
    }
    pub fn nodes(&self) -> &ObjectsMap<Node<'a>> {
        &self.nodes
    }
    pub fn nodes_info(&self) -> &ObjectsInfoMap<NodeInfo> {
        &self.nodes_info
    }
    pub fn nodes_params(&self) -> &ObjectsParamsMap {
        &self.nodes_params
    }
    pub fn ports(&self) -> &ObjectsMap<Port<'a>> {
        &self.ports
    }
    pub fn ports_info(&self) -> &ObjectsInfoMap<PortInfo> {
        &self.ports_info
    }
    pub fn ports_params(&self) -> &ObjectsParamsMap {
        &self.ports_params
    }
    pub fn links(&self) -> &ObjectsMap<Link<'a>> {
        &self.links
    }
    pub fn links_info(&self) -> &ObjectsInfoMap<LinkInfo> {
        &self.links_info
    }
}
