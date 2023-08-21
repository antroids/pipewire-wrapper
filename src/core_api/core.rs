/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Core](https://docs.pipewire.org/group__pw__core.html) bindings.
//!
use std::ffi::CStr;
use std::ops::Deref;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_wrapper_proc_macro::{interface, RawWrapper, Wrapper};

use crate::core_api::context::Context;
use crate::core_api::core::events::CoreEvents;
use crate::core_api::properties::Properties;
use crate::core_api::registry::{Registry, RegistryRef};
use crate::listeners::{AddListener, OwnListeners};
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_void_result, new_instance_raw_wrapper, raw_wrapper};

pub mod events;
pub mod info;

/// Wrapper for the external [pw_sys::pw_core] value.
/// This is a special singleton object.
/// It is used for internal PipeWire protocol features.
/// Connecting to a PipeWire instance returns one core object, the caller should then register
/// event listeners using add_listener method.
#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_core_methods, interface="Core")]
#[repr(transparent)]
pub struct CoreRef {
    #[raw]
    raw: pw_sys::pw_core,
}

/// Owned Wrapper for the [CoreRef]
#[derive(Clone, Default, Debug)]
pub struct Core {
    inner: Rc<InnerCore>,
}

#[derive(Wrapper, Debug)]
pub struct InnerCore {
    #[raw_wrapper]
    ref_: NonNull<CoreRef>,

    context: Context,
}

impl InnerCore {
    fn connect(context: Context, properties: Properties) -> crate::Result<Self> {
        let ptr =
            unsafe { pw_sys::pw_context_connect(context.as_raw_ptr(), properties.into_raw(), 0) };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            context,
        })
    }

    /// Context
    pub fn context(&self) -> &Context {
        &self.context
    }
}

impl Default for InnerCore {
    fn default() -> Self {
        Self::connect(Context::default(), Properties::default()).unwrap()
    }
}

impl Drop for InnerCore {
    fn drop(&mut self) {
        unsafe {
            pw_sys::pw_core_disconnect(self.as_raw_ptr());
        }
    }
}

impl CoreRef {
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

impl<'a> AddListener<'a> for CoreRef {
    type Events = CoreEvents<'a>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
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

impl Deref for Core {
    type Target = InnerCore;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl Core {
    /// Connect to a PipeWire instance.
    ///
    /// # Arguments
    ///
    /// * `context` - [Context]
    /// * `properties` - properties for the [Core]
    pub fn connect(context: Context, properties: Properties) -> crate::Result<Self> {
        let inner = InnerCore::connect(context, properties)?;
        Ok(Self {
            inner: Rc::new(inner),
        })
    }

    /// Create a new [Registry] proxy.
    /// The registry object will emit a global event for each global currently in the registry.
    ///
    /// # Arguments
    ///
    /// * `version` - registry version
    pub fn get_registry(&self, version: u32) -> crate::Result<Registry> {
        use crate::core_api::proxy::Proxied;
        use crate::core_api::registry::restricted::RegistryBind;
        let ref_: &RegistryRef = self.as_ref().get_registry(version, 0)?;
        Ok(Registry::from_ref(self.clone(), ref_.as_proxy()))
    }
}
