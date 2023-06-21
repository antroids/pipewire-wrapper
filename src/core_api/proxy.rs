use crate::core_api::proxy::events::ProxyEvents;
use crate::core_api::type_info::TypeInfo;
use crate::error::Error;
use crate::i32_as_void_result;
use crate::impl_api::protocol::ProtocolRef;
use crate::spa::SPA_ID_INVALID;
use crate::wrapper::{RawWrapper, Wrapper};
use pipewire_macro_impl::spa_interface_call;
use pipewire_proc_macro::RawWrapper;
use spa_sys::SPA_TYPE_INFO_Data;
use std::ffi::CStr;
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub mod events;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ProxyRef {
    #[raw]
    raw: pw_sys::pw_proxy,
}

pub struct Proxy {
    inner: Arc<InnerProxy>,
}

struct InnerProxy {
    ref_: NonNull<ProxyRef>,
}

pub trait Proxied: RawWrapper {
    fn get_type_info() -> TypeInfo<'static>;

    fn as_proxy(&self) -> &ProxyRef {
        unsafe { ProxyRef::from_raw_ptr(self.as_raw_ptr() as *mut _) }
    }
}

impl Wrapper for Proxy {
    type RawWrapperType = ProxyRef;
}

impl AsMut<ProxyRef> for Proxy {
    fn as_mut(&mut self) -> &mut ProxyRef {
        unsafe { &mut *self.inner.ref_.as_ptr() }
    }
}

impl AsRef<ProxyRef> for Proxy {
    fn as_ref(&self) -> &ProxyRef {
        unsafe { self.inner.ref_.as_ref() }
    }
}

impl Deref for Proxy {
    type Target = <Self as crate::wrapper::Wrapper>::RawWrapperType;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.ref_.as_ref() }
    }
}

impl DerefMut for Proxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl Drop for InnerProxy {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_proxy_destroy(self.ref_.as_ref().as_raw_ptr()) }
    }
}

impl Clone for Proxy {
    fn clone(&self) -> Self {
        let cloned = Self {
            inner: self.inner.clone(),
        };
        unsafe { pw_sys::pw_proxy_ref(self.as_raw_ptr()) };
        cloned
    }
}

impl Drop for Proxy {
    fn drop(&mut self) {
        if Arc::strong_count(&self.inner) > 1 {
            // can cause assert errors when used in concurrent env, probably additional sync required
            unsafe { pw_sys::pw_proxy_unref(self.as_raw_ptr()) }
        }
    }
}

impl ProxyRef {
    pub fn add_listener(&self) -> Pin<Box<ProxyEvents>> {
        let mut events = ProxyEvents::new(self);

        unsafe {
            pw_sys::pw_proxy_add_listener(
                self.as_raw_ptr(),
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _,
            )
        }

        events
    }

    //todo add_object_listener
    //todo get_user_data

    pub fn get_id(&self) -> u32 {
        unsafe { pw_sys::pw_proxy_get_id(self.as_raw_ptr()) }
    }

    pub fn get_type_and_version(&self) -> (TypeInfo, u32) {
        let mut version = 0u32;
        let type_str = unsafe { pw_sys::pw_proxy_get_type(self.as_raw_ptr(), &mut version) };
        unsafe { (TypeInfo::from_c_str(CStr::from_ptr(type_str)), version) }
    }

    pub fn get_type(&self) -> TypeInfo {
        self.get_type_and_version().0
    }

    pub fn get_protocol(&self) -> &ProtocolRef {
        unsafe { ProtocolRef::from_raw_ptr(pw_sys::pw_proxy_get_protocol(self.as_raw_ptr())) }
    }

    pub fn sync(&self, seq: i32) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_proxy_sync(self.as_raw_ptr(), seq) };
        i32_as_void_result(result)
    }

    pub fn get_bound_id(&self) -> Option<u32> {
        let id = unsafe { pw_sys::pw_proxy_get_bound_id(self.as_raw_ptr()) };
        if id == SPA_ID_INVALID {
            None
        } else {
            Some(id)
        }
    }

    pub fn error(&self, res: i32, error: &CStr) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_proxy_error(self.as_raw_ptr(), res, error.as_ptr()) };
        i32_as_void_result(result)
    }

    // todo errorf
    //todo get_object_listeners
    //todo get_get_marshal
    //todo get_install_marshal

    pub fn as_object<T>(&self) -> crate::Result<&T>
    where
        T: Proxied,
    {
        let proxy_type = self.get_type();
        let target_type = T::get_type_info();
        if proxy_type == target_type {
            unsafe { Ok(T::from_raw_ptr(self.as_raw_ptr() as *mut _)) }
        } else {
            Err(Error::TypeMismatch)
        }
    }
}
