use std::collections::HashMap;

use bitflags::Flags;

use crate::core_api::device;
use crate::core_api::device::events::DeviceEventsBuilder;
use crate::core_api::device::info::DeviceInfo;
use crate::core_api::device::DeviceRef;
use crate::core_api::proxy::Proxied;
use crate::listeners::OwnListeners;
use crate::spa::param::ParamType;
use crate::spa::pod::{BasicType, ToOwnedPod};
use crate::state::{Message, State};

#[derive(Debug, Clone)]
pub enum DeviceMessage {
    Added(u32),
    Removed(u32),

    Info(u32),
    Param(u32, ParamType),
    Props(u32),
}

impl<'a> State<'a> {
    pub fn subscribe_device_changes(&self, id: u32) {
        if let Some(device) = self.devices.lock().unwrap().get_mut(&id) {
            let listener = DeviceEventsBuilder::default()
                .info(Box::new({
                    let self_ = self.clone();
                    let device = device.clone();
                    move |info| {
                        self_
                            .devices_info
                            .lock()
                            .unwrap()
                            .insert(id, DeviceInfo::from_ref(info));
                        self_.send_message(Message::Device(DeviceMessage::Info(id)));
                        let change_mask = info.change_mask();
                        if let Some(params_subscriptions) =
                            self_.params_subscriptions.get(&DeviceRef::type_info())
                        {
                            if change_mask.contains(device::info::ChangeMask::PARAMS) {
                                let param_types: Vec<ParamType> = info
                                    .params()
                                    .iter()
                                    .filter_map(|p| {
                                        params_subscriptions.contains(&p.id()).then(|| p.id())
                                    })
                                    .collect();
                                device.subscribe_params(param_types.as_slice());
                            }
                        }
                        if change_mask.contains(device::info::ChangeMask::PROPS) {
                            self_.send_message(Message::Device(DeviceMessage::Props(id)));
                        }
                    }
                }))
                .param(Box::new({
                    let self_ = self.clone();
                    move |_, type_, _, _, pod| {
                        if let Ok(BasicType::OBJECT(obj)) = pod.downcast() {
                            if let Ok(owned) = obj.to_owned_pod() {
                                self_
                                    .devices_params
                                    .lock()
                                    .unwrap()
                                    .entry(id)
                                    .and_modify(|params| {
                                        params.insert(type_, owned);
                                    })
                                    .or_insert(HashMap::default());
                                self_
                                    .send_message(Message::Device(DeviceMessage::Param(id, type_)));
                            }
                        }
                    }
                }))
                .build();
            device.add_listener(listener);
        }
    }

    pub fn remove_device(&self, id: u32) {
        if let Some(device) = &self.devices.lock().unwrap().remove(&id) {
            self.devices_info.lock().unwrap().remove(&id);
            self.devices_params.lock().unwrap().remove(&id);
            self.send_message(Message::Device(DeviceMessage::Removed(id)));
        }
    }
}
