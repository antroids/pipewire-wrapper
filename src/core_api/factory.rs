use crate::core_api::factory::events::FactoryEvents;
use crate::wrapper::RawWrapper;
use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{proxied, RawWrapper};
use std::pin::Pin;

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[proxied(methods=pw_sys::pw_factory_methods, interface="Factory")]
#[repr(transparent)]
pub struct FactoryRef {
    #[raw]
    raw: pw_sys::pw_factory,
}

impl FactoryRef {
    pub fn add_listener(&self) -> Pin<Box<FactoryEvents>> {
        let mut events = FactoryEvents::new(self);

        unsafe {
            spa_interface_call!(
                self,
                add_listener,
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _
            )
        };

        events
    }
}
