/*
 * SPDX-License-Identifier: MIT
 */

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::object::enum_format::ObjectEnumFormatInfo;
use pipewire_wrapper::spa::pod::object::format::ObjectFormatInfo;
use pipewire_wrapper::spa::pod::object::param_buffers::ParamBuffersInfo;
use pipewire_wrapper::spa::pod::object::param_io::ParamIoInfo;
use pipewire_wrapper::spa::pod::object::param_latency::ParamLatencyInfo;
use pipewire_wrapper::spa::pod::object::param_meta::ParamMetaInfo;
use pipewire_wrapper::spa::pod::object::param_port_config::ParamPortConfigInfo;
use pipewire_wrapper::spa::pod::object::param_process_latency::ParamProcessLatencyInfo;
use pipewire_wrapper::spa::pod::object::param_profile::ParamProfileInfo;
use pipewire_wrapper::spa::pod::object::param_route::ParamRouteInfo;
use pipewire_wrapper::spa::pod::object::profiler::ProfilerInfo;
use pipewire_wrapper::spa::pod::object::prop::ObjectPropInfo;
use pipewire_wrapper::spa::pod::object::prop_info::ObjectPropInfoInfo;
use pipewire_wrapper::spa::pod::BasicType;
use pipewire_wrapper::spa::type_::Type;
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

#[test]
fn test_port_params_as_object_info() {
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
                    let port_param_callback = |_seq, id, _index, _next, param: &PodRef| {
                        if let Ok(BasicType::OBJECT(object)) = param.downcast() {
                            match object.body_type() {
                                Type::OBJECT_PROP_INFO => {
                                    let info = ObjectPropInfoInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PROPS => {
                                    let info = ObjectPropInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_FORMAT => {
                                    if id == ParamType::ENUM_FORMAT {
                                        let info = ObjectEnumFormatInfo::try_from(object).unwrap();
                                        println!("Prop info: {:?}", info);
                                    } else {
                                        let info = ObjectFormatInfo::try_from(object).unwrap();
                                        println!("Prop info: {:?}", info);
                                    }
                                }
                                Type::OBJECT_PARAM_BUFFERS => {
                                    let info = ParamBuffersInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_META => {
                                    let info = ParamMetaInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_IO => {
                                    let info = ParamIoInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_PROFILE => {
                                    let info = ParamProfileInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_PORT_CONFIG => {
                                    let info = ParamPortConfigInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_ROUTE => {
                                    let info = ParamRouteInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PROFILER => {
                                    let info = ProfilerInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_LATENCY => {
                                    let info = ParamLatencyInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                Type::OBJECT_PARAM_PROCESS_LATENCY => {
                                    let info = ParamProcessLatencyInfo::try_from(object).unwrap();
                                    println!("Prop info: {:?}", info);
                                }
                                _ => panic!("Unknown type"),
                            }
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
