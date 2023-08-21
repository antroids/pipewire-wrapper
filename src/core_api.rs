/*
 * SPDX-License-Identifier: MIT
 */

//! Bindings for PipeWire [Core API](https://docs.pipewire.org/group__api__pw__core.html)
//!
use std::ffi::{CStr, CString};
use std::ops::{AddAssign, SubAssign};
use std::sync::Mutex;

use spa_sys::spa_support;

use crate::spa::dict::DictRef;
use crate::spa::handle::HandleRef;
use crate::spa::loop_::AsLoopRef;
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::support::SupportRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_void_result, SPA_ID_INVALID};

pub mod client;
pub mod context;
pub mod core;
pub mod device;
pub mod factory;
pub mod link;
pub mod loop_;
pub mod main_loop;
pub mod node;
pub mod permissions;
pub mod port;
pub mod properties;
pub mod proxy;
pub mod registry;
pub mod type_info;

pub const PW_ID_ANY: u32 = SPA_ID_INVALID;

static mut INSTANCES: Mutex<usize> = Mutex::new(0);

/// PipeWire structure
#[derive(Debug)]
pub struct PipeWire {}

impl PipeWire {
    /// Init the pipewire, can be called several times.
    ///
    /// # Arguments
    ///
    /// * `args` - arguments
    ///
    /// Returns PipeWire struct
    ///
    /// # Examples
    ///
    /// ```
    /// use pipewire_wrapper::core_api::PipeWire;
    /// let pipe_wire = PipeWire::init(&Vec::default());
    /// ```
    pub fn init(args: &Vec<&CStr>) -> PipeWire {
        unsafe {
            let argc = &mut (args.len() as i32) as *mut ::std::os::raw::c_int;
            let argv = args.as_ptr() as *mut *mut *mut ::std::os::raw::c_char;
            {
                let mut instances = INSTANCES.lock().unwrap(); // todo try to recover with decrement after (mutex_unpoison #96469)
                pw_sys::pw_init(argc, argv);
                instances.add_assign(1);
            }
        }
        PipeWire {}
    }

    /// Check if a debug category is enabled.
    /// Debugging categories can be enabled by using the PIPEWIRE_DEBUG environment variable.
    ///
    /// # Arguments
    ///
    /// * `name` - the name of the category to check
    pub fn debug_is_category_enabled(&self, name: &CString) -> bool {
        unsafe { pw_sys::pw_debug_is_category_enabled(name.as_ptr()) }
    }

    /// Application name.
    pub fn get_application_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_application_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// Program name.
    pub fn get_prgname(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_prgname()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// User name
    pub fn get_user_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_user_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// Host name
    pub fn get_host_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_host_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// Client name
    pub fn get_client_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_client_name()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// Is PipeWire running on Valgrind
    pub fn in_valgrind(&self) -> bool {
        unsafe { pw_sys::pw_in_valgrind() }
    }

    /// Check option switch, i.e. in-valgrind, no-color, no-config, do-dlclose
    pub fn check_option(&self, option: &CStr, value: &CStr) -> bool {
        unsafe { pw_sys::pw_check_option(option.as_ptr(), value.as_ptr()) }
    }

    /// Get reversed [Direction]
    pub fn direction_reverse(direction: &Direction) -> Direction {
        Direction::from_raw(unsafe { pw_sys::pw_direction_reverse(*direction.as_raw()) })
    }

    /// Set domain
    pub fn set_domain(&self, domain: &CStr) -> crate::Result<()> {
        unsafe { i32_as_void_result(pw_sys::pw_set_domain(domain.as_ptr())) }
    }

    /// Domain
    pub fn get_domain(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_get_domain()
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    /// Get support list
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

    /// Load SPA handle
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

    /// Unload handle
    pub fn unload_spa_handle(&self, handle: &HandleRef) -> crate::Result<()> {
        unsafe { i32_as_void_result(pw_sys::pw_unload_spa_handle(handle.as_raw_ptr())) }
    }
}

impl Drop for PipeWire {
    fn drop(&mut self) {
        unsafe {
            let mut instances = INSTANCES.lock().unwrap();
            if *instances == 0 {
                pw_sys::pw_deinit();
            } else {
                instances.sub_assign(1);
            }
        }
    }
}

impl Clone for PipeWire {
    fn clone(&self) -> Self {
        let mut instances = unsafe { INSTANCES.lock().unwrap() };
        instances.add_assign(1);
        Self {}
    }
}

impl Default for PipeWire {
    fn default() -> Self {
        PipeWire::init(&Vec::default())
    }
}
