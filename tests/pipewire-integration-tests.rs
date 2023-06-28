use std::ffi::CString;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::port::events::PortEvents;
use pipewire_wrapper::core_api::port::info::PortInfoRef;
use pipewire_wrapper::core_api::port::PortRef;
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::Pipewire;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::type_::pod::object::ObjectType;
use pipewire_wrapper::spa::type_::pod::{BasicType, PodRef, ReadablePod};
use pipewire_wrapper::spa::type_::Type;
use pipewire_wrapper::wrapper::RawWrapper;

#[test]
fn test_init() {
    let arg = CString::new("test_arg").unwrap();
    let pw = Pipewire::init(&vec![&arg]);

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
    let timer = main_loop.add_timer(&callback).unwrap();
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
    let idle = main_loop.add_idle(false, &callback).unwrap();
    main_loop.enable_idle(&idle, false).unwrap();

    let callback = |signal_number| {
        println!("Signal: {:?}", signal_number);
    };
    let _signal = main_loop.add_signal(123, &callback).unwrap();

    let event_signal = AtomicBool::new(false);
    let callback = |count| {
        println!("Event: count {:?}", count);
        event_signal.store(true, Ordering::Relaxed);
    };
    let event = main_loop.add_event(&callback).unwrap();

    let callback = |_expirations| {
        main_loop.signal_event(&event).unwrap();
    };
    let timer = main_loop.add_timer(&callback).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    let callback = |_expirations| {
        main_loop.quit().unwrap();
    };
    let timer = main_loop.add_timer(&callback).unwrap();
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
    let timer = main_loop.add_timer(&callback).unwrap();
    main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
        .unwrap();

    main_loop.run().unwrap();
}

// #[test]
// fn test_init_main_loop_listener() {
//     let destroy_listener = Box::new(|| println!("Main loop destroyed!"));
//     let _main_loop_listener = {
//         let main_loop = MainLoop::default();
//
//         let mut listener = main_loop.add_listener();
//
//         listener.set_destroy_cb(Some(destroy_listener));
//
//         let callback = |_expirations| {
//             dbg!("Main loop quit");
//             main_loop.quit().unwrap();
//         };
//         let timer = main_loop.add_timer(&callback).unwrap();
//         main_loop
//             .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
//             .unwrap();
//
//         dbg!("Main loop run");
//         main_loop.run().unwrap();
//
//         listener
//     };
// }

#[test]
fn test_port_params() {
    let core = Arc::new(Core::default());
    let main_loop = core.context().main_loop();
    let port_ids_queue: Mutex<Vec<u32>> = Mutex::new(Vec::new());

    {
        let mut port_listeners: Vec<Pin<Box<PortEvents>>> = Vec::new();
        let mut registry_listener = core.get_registry(0, 0).unwrap().add_listener();
        let global_callback = {
            |id, _permissions, type_info, _version, _props| {
                if type_info == PortRef::get_type_info() {
                    port_ids_queue.lock().unwrap().push(id);
                }
            }
        };
        registry_listener.set_global(Some(Box::new(global_callback)));

        let main_loop_close_callback = |_expirations| {
            main_loop.quit().unwrap();
        };
        let timer = main_loop.add_timer(&main_loop_close_callback).unwrap();
        main_loop
            .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false)
            .unwrap();

        let main_loop_idle_callback = || {
            if let Some(port_id) = port_ids_queue.lock().unwrap().pop() {
                println!("Port {}", port_id);
                let registry = core.get_registry(0, 0).unwrap();
                if let Ok(port_proxy) = registry.bind(port_id, PortRef::get_type_info(), 0, 0) {
                    let port: &PortRef = port_proxy.as_object().unwrap();
                    let port_param_callback = |seq, id, index, next, param: &PodRef| {
                        if let Ok(basic_pod) = param.downcast() {
                            println!(
                                "Port params seq {} id {:?} index {} next {} param {:?}",
                                seq, id, index, next, basic_pod
                            )
                        }
                    };
                    let port_info_callback = |port_info: &PortInfoRef| {
                        println!("Port info {:?}", port_info.props());
                        for param in port_info.params() {
                            println!("Param info {:?}", param);
                            port.enum_params(0, param.id(), 0, u32::MAX, None).unwrap();
                        }
                    };
                    let mut port_listener = port.add_listener();
                    port_listener.set_param(Some(Box::new(port_param_callback)));
                    port_listener.set_info(Some(Box::new(port_info_callback)));
                    port_listeners.push(port_listener);
                }
            }
        };
        let _idle = main_loop.add_idle(true, &main_loop_idle_callback);

        main_loop.run().unwrap();
    }
}
