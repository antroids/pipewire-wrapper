extern crate pipewire_wrapper;

use std::collections::HashMap;
use std::ffi::CString;
use std::sync::{Arc, Mutex};

use pipewire_wrapper::core_api::core::Core;
use pipewire_wrapper::core_api::device::events::DeviceEventsBuilder;
use pipewire_wrapper::core_api::device::{Device, DeviceRef};
use pipewire_wrapper::core_api::main_loop::MainLoop;
use pipewire_wrapper::core_api::proxy::Proxied;
use pipewire_wrapper::core_api::registry::events::RegistryEventsBuilder;
use pipewire_wrapper::core_api::registry::Registry;
use pipewire_wrapper::listeners::{ListenerId, OwnListeners};
use pipewire_wrapper::spa::loop_::EventSource;
use pipewire_wrapper::spa::param::ParamType;
use pipewire_wrapper::spa::pod::object::param_route::ParamRouteType;
use pipewire_wrapper::spa::pod::object::prop::{AudioChannel, ObjectPropType};
use pipewire_wrapper::spa::pod::object::ObjectType;
use pipewire_wrapper::spa::pod::{BasicType, PodRef, PodValue};

#[derive(Debug, Default)]
pub struct DeviceRouteInfo {
    name: CString,
    description: CString,
    channels: Vec<AudioChannel>,
    channel_volumes: Vec<f32>,
}

fn main() {
    let core = Arc::new(Core::default());
    let devices: Arc<Mutex<HashMap<u32, Device>>> = Arc::new(Mutex::default());
    let device_added_queue = Arc::new(Mutex::new(Vec::<u32>::new()));
    let main_loop = core.context().main_loop();
    let registry = core.get_registry(0, 0).unwrap();
    let device_added_event = add_device_added_event(
        main_loop,
        devices.clone(),
        registry.clone(),
        device_added_queue.clone(),
    );
    let _registry_listener = add_registry_listener(
        registry,
        main_loop.clone(),
        device_added_event,
        device_added_queue,
    );
    let quit_main_loop = Box::new(|_| {
        main_loop.quit().unwrap();
    });
    let _sigint_handler = main_loop.add_signal(signal_hook::consts::SIGINT, quit_main_loop.clone());
    let _sigterm_handler = main_loop.add_signal(signal_hook::consts::SIGTERM, quit_main_loop);

    println!("Running main loop");
    main_loop.run().unwrap();
}

fn add_registry_listener<'a>(
    registry: Registry<'a>,
    main_loop: Arc<MainLoop>,
    device_added_event: EventSource<'a>,
    device_added_queue: Arc<Mutex<Vec<u32>>>,
) -> ListenerId {
    let listener = RegistryEventsBuilder::default()
        .global(Box::new(
            move |id, _permissions, type_info, _version, _props| {
                if type_info == DeviceRef::type_info() {
                    device_added_queue.lock().unwrap().push(id);
                    main_loop.signal_event(&device_added_event).unwrap();
                }
            },
        ))
        .build();
    registry.add_listener(listener)
}

fn add_device_added_event<'a>(
    main_loop: &'a Arc<MainLoop>,
    devices: Arc<Mutex<HashMap<u32, Device<'a>>>>,
    registry: Registry<'a>,
    device_added_queue: Arc<Mutex<Vec<u32>>>,
) -> EventSource<'a> {
    main_loop
        .add_event(Box::new({
            move |_count| {
                let devices = &mut devices.lock().unwrap();
                let device_added_queue = device_added_queue.lock().unwrap();
                let new_device_ids: Vec<&u32> = device_added_queue
                    .iter()
                    .filter(|device_id| !devices.contains_key(&device_id))
                    .collect();
                for &id in new_device_ids {
                    let device: Device = registry.bind_proxy(id, 0, 0).unwrap();
                    let listener = DeviceEventsBuilder::default()
                        .param(Box::new(device_param_callback))
                        .build();
                    // Params subscription is not working for some reason
                    device
                        .subscribe_params(&[ParamType::ROUTE, ParamType::PROP_INFO])
                        .unwrap();
                    device.add_listener(listener);
                    devices.insert(id, device);
                }
            }
        }))
        .unwrap()
}

fn device_param_callback(_seq: i32, _type_: ParamType, index: u32, _next: u32, param: &PodRef) {
    let mut info = DeviceRouteInfo::default();

    if let BasicType::OBJECT(obj) = param.downcast().unwrap() {
        let object_value = obj.value().unwrap();
        if let ObjectType::OBJECT_PARAM_ROUTE(props) = object_value {
            for prop in props {
                if let Ok(prop) = prop.value() {
                    match prop {
                        ParamRouteType::PROPS(obj) => {
                            if let Ok(ObjectType::OBJECT_PROPS(props)) = obj.value() {
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
                        ParamRouteType::NAME(name) => {
                            info.name = CString::from(name.value().unwrap());
                        }
                        ParamRouteType::DESCRIPTION(name) => {
                            info.description = CString::from(name.value().unwrap());
                        }
                        _ => (),
                    }
                }
            }
        }
    }
    if !info.channel_volumes.is_empty() {
        println!("Volume info: index {:?} {:?}", index, info);
    }
}
