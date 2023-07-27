/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Context](https://docs.pipewire.org/group__pw__context.html) bindings.
//!
use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice;
use std::time::Duration;

use pw_sys::pw_global;
use spa_sys::spa_support;

use pipewire_wrapper_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::events::{ContextEvents, ContextEventsBuilder};
use crate::core_api::core::Core;
use crate::core_api::factory::FactoryRef;
use crate::core_api::loop_::LoopRef;
use crate::core_api::main_loop::{MainLoop, MainLoopRef};
use crate::core_api::properties::{Properties, PropertiesRef};
use crate::impl_api::data_loop::DataLoopRef;
use crate::impl_api::global::GlobalRef;
use crate::impl_api::work_queue::WorkQueueRef;
use crate::listeners::AddListener;
use crate::spa::dict::DictRef;
use crate::spa::support::SupportRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_result, new_instance_raw_wrapper};

pub mod events;

/// Wrapper for the external [pw_sys::pw_context] value.
/// The PipeWire context object manages all locally available resources.
///
/// It is used by both clients and servers.
///
/// The context is used to:
///
/// Load modules and extend the functionality. This includes extending the protocol with new
/// object types or creating any of the available objects.
///
/// Create implementations of various objects like nodes, devices, factories, modules, etc..
/// This will usually also create pw_global objects that can then be shared with clients.
///
/// Connect to another PipeWire instance (the main daemon, for example) and interact with
/// it (See page_core_api).
///
/// Export a local implementation of an object to another instance.
#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ContextRef {
    #[raw]
    raw: pw_sys::pw_context,
}

/// Owned wrapper for the [ContextRef].
#[derive(Wrapper, Debug)]
pub struct Context {
    #[raw_wrapper]
    ref_: NonNull<ContextRef>,

    main_loop: std::sync::Arc<MainLoop>,
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_context_destroy(self.as_raw_ptr()) }
    }
}

impl Context {
    /// Creates a new [Context] for the given [MainLoop].
    ///
    /// # Arguments
    ///
    /// * `main_loop` - main loop
    /// * `properties` - extra properties for the context
    pub fn new(main_loop: std::sync::Arc<MainLoop>, properties: Properties) -> crate::Result<Self> {
        let ptr = unsafe {
            pw_sys::pw_context_new(main_loop.get_loop().as_raw_ptr(), properties.into_raw(), 0)
        };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            main_loop,
        })
    }

    /// Main loop
    pub fn main_loop(&self) -> &std::sync::Arc<MainLoop> {
        &self.main_loop
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new(
            std::sync::Arc::new(MainLoop::default()),
            Properties::default(),
        )
        .unwrap()
    }
}

impl ContextRef {
    /// Extra User data assigned to the context
    unsafe fn get_user_data<T>(&self) -> Option<&mut T> {
        let ptr = pw_sys::pw_context_get_user_data(self.as_raw_ptr()) as *mut T;
        ptr.as_mut()
    }

    /// Context properties
    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe { PropertiesRef::from_raw_ptr(pw_sys::pw_context_get_properties(self.as_raw_ptr())) }
    }

    /// Update context properties, returns the count of updated properties
    pub fn update_properties(&self, properties: &DictRef) -> i32 {
        unsafe { pw_sys::pw_context_update_properties(self.as_raw_ptr(), properties.as_raw_ptr()) }
    }

    //todo pw_context_parse_conf_section
    //todo pw_context_conf_update_props
    //todo pw_context_conf_section_for_each
    //todo pw_context_conf_section_match_rules
    //todo pw_context_conf_section_match_rules

    /// Get [SupportRef] list, associated with the [Context]
    pub fn get_support(&self) -> &[SupportRef] {
        let mut support_elements = 0u32;
        let support_array =
            unsafe { pw_sys::pw_context_get_support(self.as_raw_ptr(), &mut support_elements) };
        unsafe {
            slice::from_raw_parts(support_array as *mut SupportRef, support_elements as usize)
        }
    }

    /// Get [MainLoopRef]
    pub fn get_main_loop(&self) -> &LoopRef {
        unsafe { LoopRef::from_raw_ptr(pw_sys::pw_context_get_main_loop(self.as_raw_ptr())) }
    }

    // Since 0.3.56
    // pub fn data_loop(&self) -> &DataLoopRef {
    //     unsafe { DataLoopRef::from_raw_ptr(pw_sys::pw_context_get_data_loop(self.as_raw_ptr())) }
    // }

    /// Work queue
    pub fn get_work_queue(&self) -> &WorkQueueRef {
        unsafe { WorkQueueRef::from_raw_ptr(pw_sys::pw_context_get_work_queue(self.as_raw_ptr())) }
    }

    /// Evaluate the callback for each global
    ///
    /// # Arguments
    ///
    /// * `callback` - callback to evaluate, should return 0 to continue
    /// or any other value to return from the cycle.
    ///
    /// Return the value from last evaluated callback
    pub fn for_each_global<F>(&self, callback: F) -> crate::Result<i32>
    where
        F: FnMut(&GlobalRef) -> i32,
    {
        unsafe extern "C" fn callback_call<F>(
            data: *mut ::std::os::raw::c_void,
            global: *mut pw_global,
        ) -> i32
        where
            F: FnMut(&GlobalRef) -> i32,
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback(GlobalRef::from_raw_ptr(global))
            } else {
                -1
            }
        }

        let result = unsafe {
            pw_sys::pw_context_for_each_global(
                self.as_raw_ptr(),
                Some(callback_call::<F>),
                &callback as *const _ as *mut _,
            )
        };

        i32_as_result(result, result)
    }

    /// Find global by id
    ///
    /// # Arguments
    ///
    /// * `id` - id of the Global object
    pub fn find_global(&self, id: u32) -> Option<&GlobalRef> {
        let ptr = unsafe { pw_sys::pw_context_find_global(self.as_raw_ptr(), id) };
        if ptr.is_null() {
            None
        } else {
            unsafe { Some(GlobalRef::from_raw_ptr(ptr)) }
        }
    }

    //todo pw_context_add_spa_lib
    //todo pw_context_find_spa_lib
    //todo pw_context_load_spa_handle
    //todo pw_context_register_export_type
    //todo pw_context_find_export_type
    //todo pw_context_set_object
    //todo pw_context_get_object
}

impl<'a> AddListener<'a> for ContextRef {
    type Events = ContextEvents<'a>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
        unsafe {
            pw_sys::pw_context_add_listener(
                self.as_raw_ptr(),
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _,
            );
        }
        events
    }
}
