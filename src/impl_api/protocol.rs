use std::ffi::CStr;
use std::ptr::NonNull;
use std::rc::Rc;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::{Context, ContextRef};
use crate::new_instance_raw_wrapper;
use crate::wrapper::RawWrapper;

pub mod implementation;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ProtocolRef {
    #[raw]
    raw: pw_sys::pw_protocol,
}

#[derive(Wrapper, Debug)]
pub struct Protocol {
    #[raw_wrapper]
    ref_: NonNull<ProtocolRef>,

    context: std::sync::Arc<Context>,
}

impl Drop for Protocol {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_protocol_destroy(self.as_raw_ptr()) }
    }
}

impl Protocol {
    pub fn new(context: &std::sync::Arc<Context>, name: &CStr) -> crate::Result<Self> {
        let ptr = unsafe { pw_sys::pw_protocol_new(context.as_raw_ptr(), name.as_ptr(), 0) };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            context: context.clone(),
        })
    }

    pub fn context(&self) -> &std::sync::Arc<Context> {
        &self.context
    }
}

impl ProtocolRef {
    pub fn get_context(&self) -> &ContextRef {
        unsafe { ContextRef::from_raw_ptr(pw_sys::pw_protocol_get_context(self.as_raw_ptr())) }
    }

    unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        let ptr = pw_sys::pw_protocol_get_user_data(self.as_raw_ptr()) as *mut T;
        ptr.as_mut()
    }
}
