use std::ffi::CStr;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::ptr::{addr_of, addr_of_mut, null_mut, NonNull};

use spa_sys::{spa_callbacks, spa_hook, spa_list};

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::spa::list::{List, ListElement, ListRef};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct HookRef {
    #[raw]
    raw: spa_sys::spa_hook,
}

#[derive(Wrapper, Debug)]
pub struct Hook {
    #[raw_wrapper]
    ref_: NonNull<HookRef>,

    hook: spa_hook,
    pinned: PhantomPinned,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct HookListRef {
    #[raw]
    raw: spa_sys::spa_hook_list,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct CallbacksRef {
    #[raw]
    raw: spa_sys::spa_callbacks,
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct InterfaceRef {
    #[raw]
    raw: spa_sys::spa_interface,
}

impl InterfaceRef {
    pub fn type_(&self) -> Option<&CStr> {
        unsafe { self.raw.type_.as_ref().map(|ptr| CStr::from_ptr(ptr)) }
    }

    pub fn version(&self) -> u32 {
        self.raw.version
    }

    pub fn cb(&self) -> &CallbacksRef {
        unsafe { CallbacksRef::from_raw_ptr(&self.raw.cb as *const spa_callbacks) }
    }

    pub fn version_min(&self, version_min: u32) -> bool {
        version_min == 0 || self.version() > version_min - 1
    }
}

impl CallbacksRef {
    pub fn funcs<M>(&self) -> *const M {
        self.raw.funcs as *const M
    }

    pub fn data(&self) -> *mut ::std::os::raw::c_void {
        self.raw.data
    }

    pub fn init<M>(&mut self, funcs: *const M, data: *mut ::std::os::raw::c_void) {
        self.raw.funcs = funcs as *const _;
        self.raw.data = data;
    }
}

impl ListElement for HookRef {
    fn as_list_ptr(&self) -> *mut spa_list {
        addr_of!(self.raw.link) as *mut _
    }

    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self {
        ptr as *mut Self
    }
}

impl List for HookListRef {
    type Elem = HookRef;

    fn as_list_ptr(&self) -> *mut spa_list {
        addr_of!(self.raw.list) as *mut _
    }

    fn from_list_ptr(ptr: *mut spa_list) -> *mut Self {
        ptr as *mut Self
    }
}

impl HookRef {
    pub fn remove(&mut self) {
        unsafe {
            ListElement::remove(self);
        }
        if let Some(removed_callback) = self.raw.removed {
            unsafe {
                removed_callback(self.as_raw_ptr());
            }
        }
    }
}

impl HookListRef {
    pub fn clean(&mut self) {
        unsafe {
            while let Some(first) = self.first() {
                HookRef::remove(first)
            }
        }
    }
}

impl Drop for Hook {
    fn drop(&mut self) {
        self.remove()
    }
}

impl Hook {
    pub fn new() -> Pin<Box<Self>> {
        let spa_hook = spa_hook {
            link: spa_list {
                next: null_mut(),
                prev: null_mut(),
            },
            cb: spa_callbacks {
                funcs: null_mut(),
                data: null_mut(),
            },
            removed: None,
            priv_: null_mut(),
        };

        let mut hook = Box::new(Self {
            ref_: NonNull::dangling(),
            hook: spa_hook,
            pinned: PhantomPinned::default(),
        });
        unsafe {
            hook.ref_ = NonNull::new(addr_of_mut!(hook.hook) as *mut HookRef).unwrap();
            hook.init_detached();
        }

        Box::into_pin(hook)
    }
}
