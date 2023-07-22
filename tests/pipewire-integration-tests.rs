use std::ffi::CString;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::loop_::channel::LoopChannel;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::node::events::{
    NodeEventType, NodeEventsBuilder, NodeEventsChannelBuilder,
};
use pipewire_wrapper::core_api::node::info::NodeInfoRef;
use pipewire_wrapper::core_api::node::{Node, NodeRef};
use pipewire_wrapper::core_api::port::events::PortEventsBuilder;
use pipewire_wrapper::core_api::port::info::PortInfoRef;
use pipewire_wrapper::core_api::port::{Port, PortRef};
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::registry::events::RegistryEventsBuilder;
use pipewire_wrapper::core_api::PipeWire;
use pipewire_wrapper::listeners::OwnListeners;
use pipewire_wrapper::spa::pod::PodRef;

#[test]
fn test_init() {
    let arg = CString::new("test_arg").unwrap();
    let pw = PipeWire::init(&vec![&arg]);

    println!("Spa support{:?}", pw.get_spa_support(5));
    println!("User name {:?}", pw.get_user_name());
    println!("Program name {:?}", pw.get_prgname());
    println!("Application name {:?}", pw.get_application_name());
    println!("Client name {:?}", pw.get_client_name());
    println!("Host name {:?}", pw.get_host_name());
    println!("Domain name {:?}", pw.get_domain());
    println!(
        "Set domain {:?}",
        pw.set_domain(CString::new("test_domain").unwrap().as_ref())
    );
    assert_eq!(
        pw.get_domain().unwrap(),
        CString::new("test_domain").unwrap().as_ref()
    );
    assert!(!pw.debug_is_category_enabled(&CString::new("wrong_debug_category").unwrap()));
    println!("In valgrind {:?}", pw.in_valgrind());
}

#[test]
fn test_init_main_loop() {
    let main_loop = MainLoop::default();

    let callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop.add_timer(Box::new(callback)).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}

#[test]
fn test_sources() {
    let main_loop = MainLoop::default();

    let callback = || {
        println!("Idle");
    };
    let idle = main_loop.add_idle(false, Box::new(callback)).unwrap();
    main_loop.enable_idle(&idle, false).unwrap();

    let callback = |signal_number| {
        println!("Signal: {:?}", signal_number);
    };
    let _signal = main_loop.add_signal(123, Box::new(callback)).unwrap();

    let event_signal = AtomicBool::new(false);
    let callback = |count| {
        println!("Event: count {:?}", count);
        event_signal.store(true, Ordering::Relaxed);
    };
    let event = main_loop.add_event(Box::new(callback)).unwrap();

    let callback = |_expirations| {
        main_loop.signal_event(&event).unwrap();
    };
    let timer = main_loop.add_timer(Box::new(callback)).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    let callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop.add_timer(Box::new(callback)).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(3), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();

    assert_eq!(event_signal.load(Ordering::Relaxed), true)
}

#[test]
fn test_iterate_main_loop() {
    let main_loop = MainLoop::default();
    let loop_iterations = Mutex::new(0);

    let callback = |_expirations| {
        for _elapsed in main_loop.iter(100) {
            let mut loop_iterations = loop_iterations.lock().unwrap();
            *loop_iterations += 1;
            if *loop_iterations == 10 {
                break;
            }
        }

        main_loop.quit().unwrap();

        assert_eq!(*loop_iterations.lock().unwrap(), 10)
    };
    let timer = main_loop.add_timer(Box::new(callback)).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}

#[test]
fn test_port_params() {
    let core = Arc::new(Core::default());
    let registry = core.get_registry(0, 0).unwrap();
    let main_loop = core.context().main_loop();
    let ports: Mutex<Vec<Port>> = Mutex::default();

    let (port_sender, port_receiver) = LoopChannel::channel::<Port>();
    let _registry_listener = registry.add_listener(
        RegistryEventsBuilder::default()
            .global(Box::new({
                let registry = registry.clone();
                move |id, _permissions, type_info, _version, _props| {
                    if type_info == PortRef::type_info() {
                        let port = registry.bind_proxy(id, 0, 0).unwrap();
                        port_sender.send(port).unwrap();
                    }
                }
            }))
            .build(),
    );

    let main_loop_close_callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop
        .add_timer(Box::new(main_loop_close_callback))
        .unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    port_receiver
        .attach(
            main_loop.get_loop(),
            Box::new(move |new_ports| {
                for port in new_ports.try_iter() {
                    let port_param_callback = |seq, id, index, next, param: &PodRef| {
                        if let Ok(basic_pod) = param.downcast() {
                            println!(
                                "Port params seq {} id {:?} index {} next {} param {:?}",
                                seq, id, index, next, basic_pod
                            )
                        }
                    };
                    let port_info_callback = {
                        let port = port.clone();
                        move |port_info: &PortInfoRef| {
                            println!("Port info {:?}", port_info);
                            for param in port_info.params() {
                                println!("Param info {:?}", param);
                                port.enum_params(0, param.id(), 0, u32::MAX, None).unwrap();
                            }
                        }
                    };
                    let port_listener = PortEventsBuilder::default()
                        .param(Box::new(port_param_callback))
                        .info(Box::new(port_info_callback))
                        .build();
                    port.add_listener(port_listener);
                    ports.lock().unwrap().push(port);
                }
            }),
        )
        .unwrap();
    main_loop.run().unwrap();
}

#[test]
fn test_node_params() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();
    let node_ids_queue: Mutex<Vec<u32>> = Mutex::new(Vec::new());

    {
        let nodes: Mutex<Vec<Node>> = Mutex::default();
        let registry = core.get_registry(0, 0).unwrap();
        let _registry_listener = registry.add_listener(
            RegistryEventsBuilder::default()
                .global(Box::new({
                    |id, _permissions, type_info, _version, _props| {
                        if type_info == NodeRef::type_info() {
                            node_ids_queue.lock().unwrap().push(id);
                        }
                    }
                }))
                .build(),
        );

        let main_loop_close_callback = |_expirations| {
            main_loop.quit().unwrap();
        };
        let timer = main_loop
            .add_timer(Box::new(main_loop_close_callback))
            .unwrap();
        main_loop
            .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
            .unwrap();

        let main_loop_idle_callback = || {
            if let Some(node_id) = node_ids_queue.lock().unwrap().pop() {
                println!("Node {}", node_id);
                if let Ok(node) = registry.bind_proxy::<Node>(node_id, 0, 0) {
                    let node_param_callback = |seq, id, index, next, param: &PodRef| {
                        if let Ok(basic_pod) = param.downcast() {
                            println!(
                                "Node params seq {} id {:?} index {} next {} param {:?}",
                                seq, id, index, next, basic_pod
                            )
                        }
                    };
                    let node_info_callback = {
                        let node = node.clone();
                        move |node_info: &NodeInfoRef| {
                            println!("Node info {:?}", node_info);
                            for param in node_info.params() {
                                println!("Param info {:?}", param);
                                node.enum_params(0, param.id(), 0, u32::MAX, None).unwrap();
                            }
                        }
                    };
                    let node_listener = NodeEventsBuilder::default()
                        .param(Box::new(node_param_callback))
                        .info(Box::new(node_info_callback))
                        .build();
                    node.add_listener(node_listener);
                    nodes.lock().unwrap().push(node);
                }
            }
        };
        let _idle = main_loop.add_idle(true, Box::new(main_loop_idle_callback));

        main_loop.run().unwrap();
    }
}

#[test]
fn test_node_events_via_channel() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();
    let nodes = Arc::new(Mutex::new(Vec::<Node>::new()));
    let registry = core.get_registry(0, 0).unwrap();
    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop.add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop.add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    let (node_sender, node_receiver) = LoopChannel::channel::<Node>();

    let registry_listener = RegistryEventsBuilder::default()
        .global(Box::new({
            let registry = registry.clone();
            move |id, _permission, type_, _flags, _props| {
                if type_ == NodeRef::type_info() {
                    let node = registry.bind_proxy(id, 0, 0).unwrap();
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
