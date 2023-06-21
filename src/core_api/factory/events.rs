use crate::core_api::factory::info::FactoryInfoRef;
use crate::core_api::factory::FactoryRef;
use crate::spa::interface::Hook;
use crate::wrapper::RawWrapper;
use pipewire_proc_macro::{RawWrapper, Wrapper};
use pw_sys::pw_factory_info;
use std::pin::Pin;
use std::ptr::NonNull;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FactoryEventsRef {
    #[raw]
    raw: pw_sys::pw_factory_events,
}

#[derive(Wrapper)]
pub struct FactoryEvents<'f> {
    #[raw_wrapper]
    ref_: NonNull<FactoryEventsRef>,

    factory: &'f FactoryRef,

    raw: Pin<Box<FactoryEventsRef>>,
    hook: Pin<Box<Hook>>,

    info: Option<Box<dyn FnMut(&'f FactoryInfoRef) + 'f>>,
}

impl Drop for FactoryEvents<'_> {
    fn drop(&mut self) {
        // handled by hook
    }
}

impl<'f> FactoryEvents<'f> {
    pub(crate) fn new(factory: &'f FactoryRef) -> Pin<Box<Self>> {
        let hook = Hook::new();
        let raw = FactoryEventsRef::from_raw(pw_sys::pw_factory_events {
            version: 0,
            info: None,
        });
        let mut pinned_raw = Box::into_pin(Box::new(raw));

        Box::into_pin(Box::new(Self {
            ref_: NonNull::new(pinned_raw.as_ptr()).unwrap(),
            factory,
            raw: pinned_raw,
            hook,
            info: None,
        }))
    }

    fn info_call(
    ) -> unsafe extern "C" fn(data: *mut ::std::os::raw::c_void, info: *const pw_factory_info) {
        unsafe extern "C" fn call(data: *mut ::std::os::raw::c_void, info: *const pw_factory_info) {
            if let Some(factory_events) = (data as *mut FactoryEvents).as_mut() {
                if let Some(callback) = &mut factory_events.info {
                    callback(FactoryInfoRef::from_raw_ptr(info));
                }
            }
        }
        call
    }

    pub fn set_info(&mut self, info: Option<Box<dyn FnMut(&'f FactoryInfoRef) + 'f>>) {
        self.info = info;
        self.raw.raw.info = self.info.as_ref().map(|_| Self::info_call());
    }

    pub fn factory(&self) -> &'f FactoryRef {
        self.factory
    }
    pub fn hook(&self) -> &Pin<Box<Hook>> {
        &self.hook
    }
}
