extern crate pipewire_wrapper;

use std::collections::HashMap;
use std::ffi::CString;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::node::events::NodeEvents;
use pipewire_wrapper::core_api::node::NodeRef;
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::registry::events::RegistryEvents;
use pipewire_wrapper::core_api::registry::RegistryRef;
use pipewire_wrapper::spa::loop_::EventSource;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::type_::pod::object::prop::{AudioChannel, ObjectPropType};
use pipewire_wrapper::spa::type_::pod::object::ObjectType;
use pipewire_wrapper::spa::type_::pod::{BasicType, PodValue};

#[derive(Debug)]
pub struct VolumeInfo {
    name: CString,
    channels: Vec<AudioChannel>,
    channel_volumes: Vec<f32>,
}

// pub struct VolumeInfoExample<'a> {
//     core: Arc<Core>,
//     main_loop: Arc<MainLoop>,
//     registry: &'a RegistryRef,
// }
//
// impl<'a> VolumeInfoExample<'a> {
//     pub fn new() -> pipewire_wrapper::Result<Self> {
//         let core = Arc::new(Core::default());
//         let main_loop = core.context().main_loop().clone();
//         let registry = core.get_registry(0, 0).unwrap();
//         Ok(Self {
//             core: core.clone(),
//             main_loop,
//             registry,
//         })
//     }
// }

const QUIT_SYNC: u32 = 123;

fn main() {
    let core = Arc::new(Core::default());
    let node_id_queue_to_enum_params: Arc<Mutex<Vec<u32>>> = Arc::default();
    let node_id_queue_to_process_params: Arc<Mutex<Vec<u32>>> = Arc::default();
    let main_loop = core.context().main_loop();
    let registry = core.get_registry(0, 0).unwrap();
    let node_listeners: Arc<Mutex<HashMap<u32, Pin<Box<NodeEvents>>>>> = Arc::default();
    let node_added = add_node_added_event_listener(
        main_loop,
        registry,
        &node_id_queue_to_enum_params,
        node_listeners.clone(),
    );
    let _registry_listener = add_registry_listener(
        registry,
        main_loop,
        &node_added,
        &node_id_queue_to_enum_params,
    );

    let mut core_listener = core.add_listener();
    core_listener.set_done(Some(Box::new(|id, _seq| {
        if id == QUIT_SYNC {
            let node_count = node_id_queue_to_enum_params.lock().unwrap().len();
            if node_count == 0 {
                println!("Quit");
                main_loop.quit().unwrap();
            } else {
                println!("Processing {} nodes...", node_count);
                core.sync(QUIT_SYNC, 0).unwrap();
            }
        }
    })));
    core.sync(QUIT_SYNC, 0).unwrap();

    println!("Running main loop");
    main_loop.run().unwrap();
}

fn add_registry_listener<'a>(
    registry: &'a RegistryRef,
    main_loop: &'a Arc<MainLoop>,
    node_added: &'a EventSource,
    node_id_queue: &'a Arc<Mutex<Vec<u32>>>,
) -> Pin<Box<RegistryEvents<'a>>> {
    let mut listener = registry.add_listener();
    listener.set_global(Some(Box::new(
        |id, _permissions, type_info, _version, _props| {
            if type_info == NodeRef::type_info() {
                node_id_queue.lock().unwrap().push(id);
                main_loop.signal_event(node_added).unwrap();
            }
        },
    )));
    listener
}

fn add_node_added_event_listener<'a>(
    main_loop: &'a Arc<MainLoop>,
    registry: &'a RegistryRef,
    node_id_queue_to_enum_params: &'a Arc<Mutex<Vec<u32>>>,
    node_listeners: Arc<Mutex<HashMap<u32, Pin<Box<NodeEvents<'a>>>>>>,
) -> EventSource<'a> {
    main_loop
        .add_event(Box::new(move |_count| {
            while let Some(id) = node_id_queue_to_enum_params.lock().unwrap().pop() {
                if !node_listeners.lock().unwrap().contains_key(&id) {
                    let node_proxy = registry.bind(id, NodeRef::type_info(), 0, 0).unwrap();
                    let node: &NodeRef = node_proxy.as_object().unwrap();
                    let mut node_listener = node.add_listener();

                    node_listener.set_param(Some(Box::new(
                        move |_seq, _type_, _index, _next, param| {
                            let mut info: VolumeInfo = VolumeInfo {
                                name: CString::new("UNKNOWN").unwrap(),
                                channels: vec![],
                                channel_volumes: vec![],
                            };
                            if let BasicType::OBJECT(obj) = param.downcast().unwrap() {
                                if let ObjectType::OBJECT_PROPS(props) = obj.value().unwrap() {
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
                                                    info.channel_volumes =
                                                        array.value().unwrap().collect()
                                                }
                                                _ => {}
                                            };
                                        }
                                    }
                                }
                            }
                            if !info.channel_volumes.is_empty() {
                                println!("Volume info: {:?}", info);
                            }
                        },
                    )));
                    node_listeners.lock().unwrap().insert(id, node_listener);

                    node.enum_params(0, ParamType::PROPS, 0, u32::MAX, None)
                        .unwrap();
                }
            }
        }))
        .unwrap()
}
