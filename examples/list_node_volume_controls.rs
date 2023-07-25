extern crate pipewire_wrapper;

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::node::events::NodeEventsBuilder;
use pipewire_wrapper::core_api::node::{Node, NodeRef};
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::registry::events::RegistryEventsBuilder;
use pipewire_wrapper::core_api::registry::Registry;
use pipewire_wrapper::listeners::{ListenerId, OwnListeners};
use pipewire_wrapper::spa::loop_::EventSource;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::object::prop::{AudioChannel, ObjectPropType};
use pipewire_wrapper::spa::pod::object::ObjectType;
use pipewire_wrapper::spa::pod::{BasicType, PodRef, PodValue};

#[derive(Debug)]
pub struct VolumeInfo {
    name: CString,
    channels: Vec<AudioChannel>,
    channel_volumes: Vec<f32>,
}

fn main() {
    let core = Arc::new(Core::default());
    let nodes: Arc<Mutex<HashMap<u32, Node>>> = Arc::new(Mutex::default());
    let node_added_queue = Arc::new(Mutex::new(Vec::<u32>::new()));
    let main_loop = core.context().main_loop();
    let registry = core.get_registry(0).unwrap();
    let node_added_event = add_node_added_event(
        main_loop,
        nodes.clone(),
        registry.clone(),
        node_added_queue.clone(),
    );
    let _registry_listener = add_registry_listener(
        registry,
        main_loop.clone(),
        node_added_event,
        node_added_queue,
    );

    println!("Running main loop");
    main_loop.run().unwrap();
}

fn add_registry_listener<'a>(
    registry: Registry<'a>,
    main_loop: Arc<MainLoop>,
    node_added_event: EventSource<'a>,
    node_added_queue: Arc<Mutex<Vec<u32>>>,
) -> ListenerId {
    let listener = RegistryEventsBuilder::default()
        .global(Box::new(
            move |id, _permissions, type_info, _version, _props| {
                if type_info == NodeRef::type_info() {
                    node_added_queue.lock().unwrap().push(id);
                    main_loop
                        .get_loop()
                        .signal_event(&node_added_event)
                        .unwrap();
                }
            },
        ))
        .build();
    registry.add_listener(listener)
}

fn add_node_added_event<'a>(
    main_loop: &'a Arc<MainLoop>,
    nodes: Arc<Mutex<HashMap<u32, Node<'a>>>>,
    registry: Registry<'a>,
    node_added_queue: Arc<Mutex<Vec<u32>>>,
) -> EventSource<'a> {
    main_loop
        .get_loop()
        .add_event(Box::new({
            move |_count| {
                let nodes = &mut nodes.lock().unwrap();
                let node_added_queue = node_added_queue.lock().unwrap();
                let new_node_ids: Vec<&u32> = node_added_queue
                    .iter()
                    .filter(|node_id| !nodes.contains_key(node_id))
                    .collect();
                for &id in new_node_ids {
                    let node: Node = registry.bind_proxy(id, 0).unwrap();
                    let listener = NodeEventsBuilder::default()
                        .param(Box::new(node_param_callback))
                        .build();
                    node.subscribe_params(&[ParamType::PROPS]).unwrap();
                    node.add_listener(listener);
                    nodes.insert(id, node);
                }
            }
        }))
        .unwrap()
}

fn node_param_callback(_seq: i32, _type_: ParamType, index: u32, _next: u32, param: &PodRef) {
    let mut info: VolumeInfo = VolumeInfo {
        name: CString::new("UNKNOWN").unwrap(),
        channels: vec![],
        channel_volumes: vec![],
    };
    if let BasicType::OBJECT(obj) = param.downcast().unwrap() {
        let object_value = obj.value().unwrap();
        if let ObjectType::OBJECT_PROPS(props) = object_value {
            for prop in props {
                if let Ok(prop_val) = prop.value() {
                    match prop_val {
                        ObjectPropType::CARD(name)
                        | ObjectPropType::CARD_NAME(name)
                        | ObjectPropType::DEVICE(name)
                        | ObjectPropType::DEVICE_NAME(name) => {
                            info.name = CString::from(name.value().unwrap())
                        }
                        ObjectPropType::CHANNEL_MAP(array) => {
                            info.channels = array.value().unwrap().collect()
                        }
                        ObjectPropType::CHANNEL_VOLUMES(array) => {
                            info.channel_volumes = array.value().unwrap().collect()
                        }
                        _ => {}
                    };
                }
            }
        }
    }
    if !info.channel_volumes.is_empty() {
        println!("Volume info: index {:?} {:?}", index, info);
    }
}
