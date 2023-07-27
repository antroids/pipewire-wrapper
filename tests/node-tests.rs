use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use pipewire_wrapper::{
    core_api::{
        core::Core,
        loop_::channel::LoopChannel,
        node::{
            events::{NodeEventType, NodeEventsChannelBuilder},
            Node, NodeRef,
        },
        proxy::Proxied,
        registry::events::RegistryEventsBuilder,
    },
    listeners::OwnListeners,
};

/*
 * SPDX-License-Identifier: MIT
 */
#[test]
fn test_node_events_via_channel() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();
    let nodes = Arc::new(Mutex::new(Vec::<Node>::new()));
    let registry = core.get_registry(0).unwrap();
    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop
        .get_loop()
        .add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop
        .get_loop()
        .add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    let main_loop_close_callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop
        .get_loop()
        .add_timer(Box::new(main_loop_close_callback))
        .unwrap();
    main_loop
        .get_loop()
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    let (node_sender, node_receiver) = LoopChannel::channel::<Node>();

    let registry_listener = RegistryEventsBuilder::default()
        .global(Box::new({
            let registry = registry.clone();
            move |id, _permission, type_, _flags, _props| {
                if type_ == NodeRef::type_info() {
                    let node = registry.bind_proxy(id, 0).unwrap();
                    node_sender.send(node).unwrap();
                }
            }
        }))
        .build();
    registry.add_listener(registry_listener);

    let _attached_node_receiver = node_receiver.attach(
        main_loop.get_loop(),
        Box::new(move |new_nodes| {
            for node in new_nodes.try_iter() {
                let (node_listener, node_event_receiver) = NodeEventsChannelBuilder::default()
                    .info()
                    .param()
                    .build_loop_channel();
                node.add_listener(node_listener);
                node_event_receiver
                    .attach(
                        main_loop.get_loop(),
                        Box::new({
                            let node = node.clone();
                            move |events| {
                                for event in events.try_iter() {
                                    match event {
                                        NodeEventType::Info(info) => {
                                            println!("Node info: {:?}", &info);
                                            for param in info.params() {
                                                node.enum_params(0, param.id(), 0, u32::MAX, None)
                                                    .unwrap();
                                            }
                                        }
                                        NodeEventType::Param(seq, type_, index, next, pod) => {
                                            println!("Node param (seq={:?} type={:?}, index={:?}, next={:?}): {:?}", seq, type_, index, next, &pod.as_pod().downcast());
                                        }
                                    }
                                }
                            }
                        }),
                    )
                    .unwrap();
                nodes.lock().unwrap().push(node);
            }
        }),
    );

    main_loop.run().unwrap();
}
