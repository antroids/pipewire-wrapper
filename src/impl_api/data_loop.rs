/*
 * SPDX-License-Identifier: MIT
 */

//! PipeWire [Data Loop](https://docs.pipewire.org/group__pw__data__loop.html) bindings.
//!
use std::ptr::NonNull;
use std::rc::Rc;
use std::time::Duration;

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::loop_::LoopRef;
use crate::core_api::properties::Properties;
use crate::core_api::PipeWire;
use crate::spa::dict::DictRef;
use crate::spa::thread::ThreadRef;
use crate::wrapper::RawWrapper;
use crate::{i32_as_result, new_instance_raw_wrapper};

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct DataLoopRef {
    #[raw]
    raw: pw_sys::pw_data_loop,
}

#[derive(Wrapper, Debug)]
pub struct DataLoop {
    #[raw_wrapper]
    ref_: NonNull<DataLoopRef>,

    pipewire: std::sync::Arc<PipeWire>,
}

impl Drop for DataLoop {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_data_loop_destroy(self.as_raw_ptr()) }
    }
}

impl Default for DataLoop {
    fn default() -> Self {
        Self::new(
            &std::sync::Arc::new(PipeWire::default()),
            Properties::default().dict(),
        )
        .unwrap()
    }
}

impl DataLoop {
    pub fn new(pipewire: &std::sync::Arc<PipeWire>, properties: &DictRef) -> crate::Result<Self> {
        let ptr = unsafe { pw_sys::pw_data_loop_new(properties.as_raw_ptr()) };
        Ok(Self {
            ref_: new_instance_raw_wrapper(ptr)?,
            pipewire: pipewire.clone(),
        })
    }

    // since listener has only destroy handler, there is no big reason to implement add_listener
}

impl DataLoopRef {
    pub fn wait(&self, timeout: i32) -> crate::Result<i32> {
        let result = unsafe { pw_sys::pw_data_loop_wait(self.as_raw_ptr(), timeout) };
        i32_as_result(result, result)
    }

    pub fn exit(&self) {
        unsafe { pw_sys::pw_data_loop_exit(self.as_raw_ptr()) };
    }

    pub fn get_loop(&self) -> &LoopRef {
        unsafe { LoopRef::from_raw_ptr(pw_sys::pw_data_loop_get_loop(self.as_raw_ptr())) }
    }

    pub fn start(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_data_loop_start(self.as_raw_ptr()) };
        i32_as_result(result, ())
    }

    pub fn stop(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_data_loop_stop(self.as_raw_ptr()) };
        i32_as_result(result, ())
    }

    pub fn in_thread(&self) -> bool {
        unsafe { pw_sys::pw_data_loop_in_thread(self.as_raw_ptr()) }
    }

    pub fn get_thread(&self) -> &ThreadRef {
        unsafe { ThreadRef::from_raw_ptr(pw_sys::pw_data_loop_get_thread(self.as_raw_ptr())) }
    }

    //todo pw_data_loop_invoke
    //todo pw_data_loop_set_thread_utils
}
