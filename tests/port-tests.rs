/*
 * SPDX-License-Identifier: MIT
 */

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use pipewire_wrapper::{
    core_api::{
        core::Core,
        loop_::channel::LoopChannel,
        port::{events::PortEventsBuilder, info::PortInfoRef, Port, PortRef},
        proxy::Proxied,
        registry::events::RegistryEventsBuilder,
    },
    listeners::OwnListeners,
    spa::pod::PodRef,
};

#[test]
fn test_port_params() {
    let core = Arc::new(Core::default());
    let registry = core.get_registry(0).unwrap();
    let main_loop = core.context().main_loop();
    let ports: Mutex<Vec<Port>> = Mutex::default();

    let (port_sender, port_receiver) = LoopChannel::channel::<Port>();
    let _registry_listener = registry.add_listener(
        RegistryEventsBuilder::default()
            .global(Box::new({
                let registry = registry.clone();
                move |id, _permissions, type_info, _version, _props| {
                    if type_info == PortRef::type_info() {
                        let port = registry.bind_proxy(id, 0).unwrap();
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
        .get_loop()
        .add_timer(Box::new(main_loop_close_callback))
        .unwrap();
    main_loop
        .get_loop()
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
