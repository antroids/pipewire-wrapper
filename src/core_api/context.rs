use std::ffi::CStr;
use std::pin::Pin;
use std::ptr::NonNull;
use std::rc::Rc;
use std::slice;
use std::time::Duration;

use pw_sys::pw_global;
use spa_sys::spa_support;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::context::events::{ContextEvents, ContextEventsBuilder};
use crate::core_api::core::Core;
use crate::core_api::data_loop::DataLoopRef;
use crate::core_api::factory::FactoryRef;
use crate::core_api::loop_::LoopRef;
use crate::core_api::main_loop::{MainLoop, MainLoopRef};
use crate::core_api::properties::{Properties, PropertiesRef};
use crate::impl_api::global::GlobalRef;
use crate::impl_api::work_queue::WorkQueueRef;
use crate::spa::dict::DictRef;
use crate::spa::support::SupportRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_result, new_instance_raw_wrapper};

pub mod events;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ContextRef {
    #[raw]
    raw: pw_sys::pw_context,
}

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
    pub fn new(
        main_loop: std::sync::Arc<MainLoop>,
        properties: Properties,
        user_data_size: usize,
    ) -> crate::Result<Self> {
        let ptr = unsafe {
            pw_sys::pw_context_new(
                main_loop.get_loop().as_raw_ptr(),
                properties.into_raw(),
                user_data_size,
            )
        };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            main_loop,
        })
    }

    pub fn main_loop(&self) -> &std::sync::Arc<MainLoop> {
        &self.main_loop
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new(
            std::sync::Arc::new(MainLoop::default()),
            Properties::default(),
            0,
        )
        .unwrap()
    }
}

impl ContextRef {
    #[must_use]
    pub fn add_listener<'a>(
        &self,
        events: Pin<Box<ContextEvents<'a>>>,
    ) -> Pin<Box<ContextEvents<'a>>> {
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

    pub unsafe fn get_user_data<T>(&self) -> &mut T {
        let ptr = pw_sys::pw_context_get_user_data(self.as_raw_ptr()) as *mut T;
        &mut *ptr
    }

    pub fn get_properties(&self) -> &PropertiesRef {
        unsafe { PropertiesRef::from_raw_ptr(pw_sys::pw_context_get_properties(self.as_raw_ptr())) }
    }

    pub fn update_properties(&self, properties: &DictRef) -> i32 {
        unsafe { pw_sys::pw_context_update_properties(self.as_raw_ptr(), properties.as_raw_ptr()) }
    }

    //todo pw_context_parse_conf_section
    //todo pw_context_conf_update_props
    //todo pw_context_conf_section_for_each
    //todo pw_context_conf_section_match_rules
    //todo pw_context_conf_section_match_rules

    pub fn get_support(&self) -> &[SupportRef] {
        let mut support_elements = 0u32;
        let support_array =
            unsafe { pw_sys::pw_context_get_support(self.as_raw_ptr(), &mut support_elements) };
        unsafe {
            slice::from_raw_parts(support_array as *mut SupportRef, support_elements as usize)
        }
    }

    pub fn get_main_loop(&self) -> &LoopRef {
        unsafe { LoopRef::from_raw_ptr(pw_sys::pw_context_get_main_loop(self.as_raw_ptr())) }
    }

    // Since 0.3.56
    // pub fn data_loop(&self) -> &DataLoopRef {
    //     unsafe { DataLoopRef::from_raw_ptr(pw_sys::pw_context_get_data_loop(self.as_raw_ptr())) }
    // }

    pub fn get_work_queue(&self) -> &WorkQueueRef {
        unsafe { WorkQueueRef::from_raw_ptr(pw_sys::pw_context_get_work_queue(self.as_raw_ptr())) }
    }

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

#[test]
fn test_context_init() {
    let context = Context::default();

    let timer_callback = |_| {
        context.main_loop.quit();
    };
    let timer = context
        .main_loop()
        .add_timer(Box::new(timer_callback))
        .unwrap();
    context
        .main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false);

    context.main_loop().run();
}

#[test]
fn test_context_events() {
    let context = std::sync::Arc::new(Context::default());

    let events = ContextEventsBuilder::default()
        .global_added(Box::new(|global| {
            println!("Global added {:?}", global);
        }))
        .build();
    let events = context.add_listener(events);

    let core = Core::connect(&context, Properties::default(), 0).unwrap();
    let registry = core.get_registry(0, 0);

    let timer_callback = |_| {
        context.main_loop.quit();
    };
    let timer = context
        .main_loop()
        .add_timer(Box::new(timer_callback))
        .unwrap();
    context
        .main_loop
        .update_timer(&timer, Duration::from_secs(1), Duration::ZERO, false);

    context.main_loop().run();
}

#[test]
fn test_for_each_global() {
    let context = Context::default();

    context
        .for_each_global(|global: &GlobalRef| {
            println!("Global {:?}", global.as_raw_ptr());
            0i32
        })
        .unwrap();
}
