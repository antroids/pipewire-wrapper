extern crate pipewire_wrapper;

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::node::events::NodeEvents;
use pipewire_wrapper::core_api::node::NodeRef;
use pipewire_wrapper::core_api::permissions::Permissions;
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::registry::events::RegistryEvents;
use pipewire_wrapper::core_api::registry::RegistryRef;
use pipewire_wrapper::core_api::type_info::TypeInfo;
use pipewire_wrapper::spa::dict::DictRef;
use pipewire_wrapper::spa::loop_::EventSource;

fn main() {
    let core = Arc::new(Core::default());
    let node_id_queue: Arc<Mutex<Vec<u32>>> = Arc::default();
    let main_loop = core.context().main_loop();
    let registry = core.get_registry(0, 0).unwrap();

    let node_added = add_node_added_event_listener(main_loop, registry, &node_id_queue);
    let registry_listener = add_registry_listener(registry, main_loop, &node_added, &node_id_queue);

    let node_listeners: Arc<Mutex<HashMap<u32, NodeEvents>>> = Arc::default();

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
    node_id_queue: &'a Arc<Mutex<Vec<u32>>>,
) -> EventSource<'a> {
    let node_id_queue = node_id_queue.clone();
    main_loop
        .add_event(Box::new(|count| {
            println!("Node added, count {:?}", count);
        }))
        .unwrap()
}
