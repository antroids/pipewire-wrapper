/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Registry](https://docs.pipewire.org/group__pw__registry.html) bindings.
//!
use std::ops::{Deref, DerefMut};
use std::pin::Pin;

use pw_sys::pw_proxy;

use pipewire_wrapper_proc_macro::{interface, proxy_wrapper, RawWrapper, Wrapper};

use crate::core_api::core::Core;
use crate::core_api::proxy::Proxied;
use crate::core_api::proxy::{Proxy, ProxyRef};
use crate::core_api::registry::events::RegistryEvents;
use crate::core_api::registry::restricted::RegistryBind;
use crate::core_api::type_info::TypeInfo;
use crate::listeners::{AddListener, Listeners, OwnListeners};
use crate::spa_interface_call;
use crate::wrapper::*;
use crate::{i32_as_void_result, raw_wrapper};

pub mod events;

/// Wrapper for the external [pw_sys::pw_registry] proxy.
///
/// The registry object is a singleton object that keeps track of global objects on the PipeWire instance.
/// When a client creates a registry object, the registry object will emit a global event for each
/// global currently in the registry. Globals come and go as a result of device hotplugs or
/// reconfiguration or other events, and the registry will send out global and global_remove events
/// to keep the client up to date with the changes. To mark the end of the initial burst of events,
/// the client can use the pw_core.sync method immediately after calling pw_core.get_registry.
///
/// A client can bind to a global object by using the bind request. This creates a client-side
/// proxy that lets the object emit events to the client and lets the client invoke methods on the
/// object.
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use pipewire_wrapper::core_api::core::Core;
/// use pipewire_wrapper::core_api::device::DeviceRef;
/// use pipewire_wrapper::core_api::registry::events::RegistryEventsBuilder;
/// use pipewire_wrapper::listeners::OwnListeners;
/// use std::time::Duration;
/// use pipewire_wrapper::core_api::loop_::Loop;
///
/// let core = Core::default();
/// let context = core.context();
///  let main_loop = context.main_loop();
///  let registry = core.get_registry(0).unwrap();
///
/// let _registry_events = registry.add_listener(
///      RegistryEventsBuilder::default()
///          .global(Box::new(
///              |_id, permissions, type_info, version, properties| {
///                 println!(
///                      "Global {:?} {:?} {:?} {:?}",
///                      permissions, type_info, version, properties
///                 );
///             },
///         ))
///         .build(),
///  );
///
/// let _timer = main_loop.quit_after(Duration::from_secs(1)).unwrap();
///
/// main_loop.run().unwrap();
/// ```
#[derive(RawWrapper, Debug)]
#[interface(methods=pw_sys::pw_registry_methods, interface="Registry")]
#[repr(transparent)]
pub struct RegistryRef {
    #[raw]
    raw: pw_sys::pw_registry,
}

impl RegistryRef {
    /// Bind to the global object with id and use the client proxy.
    /// After this call, methods can be send to the remote global object and events can be received.
    ///
    /// # Arguments
    ///
    /// * `id` - global object id
    /// * `type_info` - global object type
    /// * `version` - interface version
    ///
    /// Returns reference to the proxy object.
    pub(crate) fn bind(
        &self,
        id: u32,
        type_info: TypeInfo,
        version: u32,
    ) -> crate::Result<&ProxyRef> {
        let result =
            unsafe { spa_interface_call!(self, bind, id, type_info.as_ptr(), version, 0)? };
        raw_wrapper(result as *mut pw_proxy)
    }

    /// Attempt to destroy a global object.
    pub fn destroy(&self, id: u32) -> crate::Result<()> {
        let result = spa_interface_call!(self, destroy, id)?;
        i32_as_void_result(result)
    }
}

impl AddListener for RegistryRef {
    type Events = RegistryEvents;

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

/// [Wrapper] for the Registry proxy.
/// See [RegistryRef]
#[derive(Clone, Debug)]
#[proxy_wrapper(RegistryRef)]
pub struct Registry {
    ref_: Proxy,

    listeners: Listeners<Pin<Box<RegistryEvents>>>,
}

impl RegistryBind for Registry {
    fn from_ref(core: Core, ref_: &ProxyRef) -> Self {
        Self {
            ref_: Proxy::from_ref(core, ref_),
            listeners: Listeners::default(),
        }
    }
}

impl OwnListeners for Registry {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener>::Events>>> {
        &self.listeners
    }
}

impl Registry {
    /// Bind to the global object with id and use the client proxy.
    /// After this call, methods can be send to the remote global object and events can be received.
    ///
    /// # Arguments
    ///
    /// * `id` - global object id
    /// * `version` - interface version, usually 0
    ///
    /// Returns bound [Proxy] object.
    pub fn bind_proxy<T>(&self, id: u32, version: u32) -> crate::Result<T>
    where
        T: RegistryBind,
        <T as Wrapper>::RawWrapperType: Proxied,
    {
        let type_info = T::RawWrapperType::type_info();
        let ref_ = self.bind(id, type_info, version)?;
        Ok(T::from_ref(self.ref_.core().clone(), ref_))
    }
}

pub(crate) mod restricted {
    use crate::core_api::proxy::{Proxied, ProxyRef};
    use crate::wrapper::Wrapper;

    pub trait RegistryBind
    where
        Self: Wrapper,
        Self::RawWrapperType: Proxied,
    {
        fn from_ref(core: crate::core_api::core::Core, ref_: &ProxyRef) -> Self;
    }
}
