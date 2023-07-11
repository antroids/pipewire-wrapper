use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;

use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::{interface, spa_interface, RawWrapper, Wrapper};

use crate::core_api::context::Context;
use crate::core_api::core::events::CoreEvents;
use crate::core_api::properties::Properties;
use crate::core_api::registry::events::RegistryEventsBuilder;
use crate::core_api::registry::{Registry, RegistryRef};
use crate::core_api::type_info::TypeInfo;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_void_result, new_instance_raw_wrapper, raw_wrapper};

pub mod events;
pub mod info;

#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_core_methods, interface="Core")]
#[repr(transparent)]
pub struct CoreRef {
    #[raw]
    raw: pw_sys::pw_core,
}

#[derive(Wrapper, Debug)]
pub struct Core {
    #[raw_wrapper]
    ref_: NonNull<CoreRef>,

    context: std::sync::Arc<Context>,
}

impl Core {
    pub fn connect(
        context: &std::sync::Arc<Context>,
        properties: Properties,
        user_data_size: usize,
    ) -> crate::Result<Self> {
        let ptr = unsafe {
            pw_sys::pw_context_connect(context.as_raw_ptr(), properties.into_raw(), user_data_size)
        };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            context: context.clone(),
        })
    }

    pub fn context(&self) -> &std::sync::Arc<Context> {
        &self.context
    }

    pub fn get_registry(&self, version: u32, user_data_size: usize) -> crate::Result<Registry> {
        use crate::core_api::proxy::Proxied;
        use crate::core_api::registry::restricted::RegistryBind;
        let ref_: &RegistryRef = self.as_ref().get_registry(version, user_data_size)?;
        Ok(Registry::from_ref(self, ref_.as_proxy()))
    }
}

impl Default for Core {
    fn default() -> Self {
        Core::connect(
            &std::sync::Arc::new(Context::default()),
            Properties::default(),
            0,
        )
        .unwrap()
    }
}

impl Drop for Core {
    fn drop(&mut self) {
        unsafe {
            pw_sys::pw_core_disconnect(self.as_raw_ptr());
        }
    }
}

impl CoreRef {
    #[must_use]
    pub fn add_listener<'a>(&'a self, events: Pin<Box<CoreEvents<'a>>>) -> Pin<Box<CoreEvents>> {
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

    pub fn hello(&self, version: u32) -> crate::Result<()> {
        let result = spa_interface_call!(self, hello, version)?;
        i32_as_void_result(result)
    }

    pub fn sync(&self, id: u32, seq: i32) -> crate::Result<()> {
        let result = spa_interface_call!(self, sync, id, seq)?;
        i32_as_void_result(result)
    }

    pub fn pong(&self, id: u32, seq: i32) -> crate::Result<()> {
        let result = spa_interface_call!(self, pong, id, seq)?;
        i32_as_void_result(result)
    }

    pub fn error(
        &self,
        id: u32,
        seq: i32,
        error_code: i32,
        description: &CStr,
    ) -> crate::Result<()> {
        let result = spa_interface_call!(self, error, id, seq, error_code, description.as_ptr())?;
        i32_as_void_result(result)
    }

    fn get_registry(&self, version: u32, user_data_size: usize) -> crate::Result<&RegistryRef> {
        let ptr = spa_interface_call!(self, get_registry, version, user_data_size)?;
        raw_wrapper(ptr)
    }

    // todo create_object
    // todo destroy
}

#[test]
fn test_create_core() {
    let core = Core::default();
    let context = core.context();
    let main_loop = context.main_loop();
    let registry = core.get_registry(0, 0).unwrap();

    let registry_events = registry.add_listener(
        RegistryEventsBuilder::default()
            .global(Box::new(
                |id, permissions, type_info, version, properties| {
                    println!(
                        "Global {:?} {:?} {:?} {:?}",
                        permissions, type_info, version, properties
                    );
                },
            ))
            .build(),
    );

    let timer_callback = |_| {
        core.context().main_loop().quit();
    };
    let timer = main_loop.add_timer(Box::new(timer_callback)).unwrap();
    main_loop.update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false);

    main_loop.run();
}
