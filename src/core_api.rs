use std::ffi::{CStr, CString};
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Once;
use std::time::Duration;

use spa_sys::spa_support;

use crate::core_api::loop_::LoopRef;
use crate::core_api::main_loop::{MainLoop, MainLoopRef};
use crate::spa::dict::DictRef;
use crate::spa::handle::HandleRef;
use crate::spa::loop_::AsLoopRef;
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::support::SupportRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{error, i32_as_void_result, spa};

pub mod client;
pub mod context;
pub mod core;
pub mod data_loop;
pub mod device;
pub mod factory;
pub mod loop_;
pub mod main_loop;
pub mod node;
pub mod permissions;
pub mod port;
pub mod properties;
pub mod proxy;
pub mod registry;
pub mod type_info;

static mut INIT: Once = Once::new();

#[derive(Debug)]
pub struct Pipewire {}

impl Pipewire {
    pub fn init(args: &Vec<&CStr>) -> Pipewire {
        unsafe {
            INIT.call_once_force(|_state| {
                let argc = &mut (args.len() as i32) as *mut ::std::os::raw::c_int;
                let argv = args.as_ptr() as *mut *mut *mut ::std::os::raw::c_char;
                pw_sys::pw_init(argc, argv);
            });
        }
        Pipewire {}
    }

    pub fn debug_is_category_enabled(&self, name: &CString) -> bool {
        unsafe { pw_sys::pw_debug_is_category_enabled(name.as_ptr()) }
    }

    pub fn get_application_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_application_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_prgname(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_prgname()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_user_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_user_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_host_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_host_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_client_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_client_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn in_valgrind(&self) -> bool {
        unsafe { pw_sys::pw_in_valgrind() }
    }

    pub fn check_option(&self, option: &CStr, value: &CStr) -> bool {
        unsafe { pw_sys::pw_check_option(option.as_ptr(), value.as_ptr()) }
    }

    pub fn direction_reverse(direction: &Direction) -> Direction {
        Direction::from_raw(unsafe { pw_sys::pw_direction_reverse(*direction.as_raw()) })
    }

    pub fn set_domain(&self, domain: &CStr) -> crate::Result<()> {
        unsafe { i32_as_void_result(pw_sys::pw_set_domain(domain.as_ptr())) }
    }

    pub fn get_domain(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_domain()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_spa_support(&self, max_support_elements: usize) -> Vec<SupportRef> {
        let mut support_vec: Vec<SupportRef> = Vec::with_capacity(max_support_elements);
        let support_count = unsafe {
            pw_sys::pw_get_support(
                support_vec.as_mut_ptr() as *mut spa_support,
                max_support_elements as u32,
            )
        };
        support_vec.truncate(support_count as usize);
        support_vec
    }

    pub fn load_spa_handle(
        &self,
        lib: &CStr,
        factory_name: &CStr,
        info: &DictRef,
        support: Vec<SupportRef>,
    ) -> Option<&HandleRef> {
        let handle = unsafe {
            pw_sys::pw_load_spa_handle(
                lib.as_ptr(),
                factory_name.as_ptr(),
                info.as_raw_ptr(),
                support.len() as u32,
                support.as_ptr() as *const spa_sys::spa_support,
            )
        };
        unsafe { (handle as *mut HandleRef).as_ref() }
    }

    pub fn unload_spa_handle(&self, handle: &HandleRef) -> crate::Result<()> {
        unsafe { i32_as_void_result(pw_sys::pw_unload_spa_handle(handle.as_raw_ptr())) }
    }
}

impl Drop for Pipewire {
    fn drop(&mut self) {
        unsafe {
            if INIT.is_completed() {
                pw_sys::pw_deinit();
                INIT = Once::new();
            }
        }
    }
}

impl Default for Pipewire {
    fn default() -> Self {
        Pipewire::init(&Vec::default())
    }
}
