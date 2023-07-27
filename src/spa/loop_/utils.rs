/*
 * SPDX-License-Identifier: MIT
 */
use std::os::fd::RawFd;
use std::ptr::{null_mut, NonNull};
use std::rc::Rc;
use std::time::Duration;

use pipewire_wrapper_proc_macro::{spa_interface, RawWrapper};

use crate::error::Error;
use crate::i32_as_void_result;
use crate::spa::loop_::{
    AsLoopRef, EventSource, IOSource, IdleSource, SignalSource, SourceRef, TimerSource,
};
use crate::spa::type_::TimespecRef;
use crate::spa_interface_call;
use crate::wrapper::{RawWrapper, SpaInterface, Wrapper};

#[derive(RawWrapper, Debug)]
#[spa_interface(methods=spa_sys::spa_loop_utils_methods)]
#[repr(transparent)]
pub struct LoopUtilsRef {
    #[raw]
    raw: spa_sys::spa_loop_utils,
}

impl LoopUtilsRef {
    pub fn add_io<'l, F>(
        &self,
        loop_: &'l dyn AsLoopRef,
        fd: RawFd,
        mask: u32,
        callback: Box<F>,
    ) -> crate::Result<IOSource<'l>>
    where
        F: FnMut(RawFd, u32),
        F: 'l,
    {
        unsafe extern "C" fn callback_call<F>(
            data: *mut ::std::os::raw::c_void,
            fd: ::std::os::raw::c_int,
            mask: u32,
        ) where
            F: FnMut(RawFd, u32),
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback(fd, mask);
            }
        }
        let data = &*callback as *const F as *mut _;
        let func = callback_call::<F>;
        let source = spa_interface_call!(self, add_io, fd, mask, false, Some(func), data)?;

        Ok(IOSource {
            ref_: NonNull::new(source as *mut SourceRef).unwrap(),
            loop_,
            callback,
        })
    }

    pub fn update_io(&self, source: &IOSource, mask: u32) -> crate::Result<()> {
        let result = spa_interface_call!(self, update_io, source.as_raw(), mask)?;
        i32_as_void_result(result)
    }

    pub fn add_idle<'l, F>(
        &self,
        loop_: &'l dyn AsLoopRef,
        enabled: bool,
        callback: Box<F>,
    ) -> crate::Result<IdleSource<'l>>
    where
        F: FnMut(),
        F: 'l,
    {
        unsafe extern "C" fn callback_call<F>(data: *mut ::std::os::raw::c_void)
        where
            F: FnMut(),
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback();
            }
        }
        let data = &*callback as *const F as *mut _;
        let source = spa_interface_call!(self, add_idle, enabled, Some(callback_call::<F>), data)?;

        Ok(IdleSource {
            ref_: NonNull::new(source as *mut SourceRef).unwrap(),
            loop_,
            callback,
        })
    }

    pub fn enable_idle(&self, source: &IdleSource, enabled: bool) -> crate::Result<()> {
        let result = spa_interface_call!(self, enable_idle, source.as_raw(), enabled)?;
        i32_as_void_result(result)
    }

    pub fn add_event<'l, F>(
        &self,
        loop_: &'l dyn AsLoopRef,
        callback: Box<F>,
    ) -> crate::Result<EventSource<'l>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        unsafe extern "C" fn callback_call<F>(data: *mut ::std::os::raw::c_void, count: u64)
        where
            F: FnMut(u64),
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback(count);
            }
        }
        let data = &*callback as *const F as *mut _;
        let source = spa_interface_call!(self, add_event, Some(callback_call::<F>), data)?;

        Ok(EventSource {
            ref_: NonNull::new(source as *mut SourceRef).unwrap(),
            loop_,
            callback,
        })
    }

    pub fn signal_event(&self, source: &EventSource) -> crate::Result<()> {
        let result = spa_interface_call!(self, signal_event, source.as_raw())?;
        i32_as_void_result(result)
    }

    pub fn add_timer<'l, F>(
        &self,
        loop_: &'l dyn AsLoopRef,
        callback: Box<F>,
    ) -> crate::Result<TimerSource<'l>>
    where
        F: FnMut(u64),
        F: 'l,
    {
        unsafe extern "C" fn callback_call<F>(data: *mut ::std::os::raw::c_void, count: u64)
        where
            F: FnMut(u64),
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback(count);
            }
        }
        let data = &*callback as *const F as *mut _;
        let source = spa_interface_call!(self, add_timer, Some(callback_call::<F>), data)?;

        Ok(TimerSource {
            ref_: NonNull::new(source as *mut SourceRef).unwrap(),
            loop_,
            callback,
        })
    }

    pub fn update_timer(
        &self,
        source: &TimerSource,
        value: Duration,
        interval: Duration,
        absolute: bool,
    ) -> crate::Result<()> {
        let value: TimespecRef = value.try_into()?;
        let interval: TimespecRef = interval.try_into()?;
        let result = spa_interface_call!(
            self,
            update_timer,
            source.as_raw(),
            value.as_raw_ptr(),
            interval.as_raw_ptr(),
            absolute
        )?;
        i32_as_void_result(result)
    }

    pub fn disable_timer(&self, source: &TimerSource) -> crate::Result<()> {
        let result = spa_interface_call!(
            self,
            update_timer,
            source.as_raw(),
            null_mut(),
            null_mut(),
            false
        )?;
        i32_as_void_result(result)
    }

    pub fn add_signal<'l, F>(
        &self,
        loop_: &'l dyn AsLoopRef,
        signal_number: i32,
        callback: Box<F>,
    ) -> crate::Result<SignalSource<'l>>
    where
        F: FnMut(i32),
        F: 'l,
    {
        unsafe extern "C" fn callback_call<F>(
            data: *mut ::std::os::raw::c_void,
            signal_number: ::std::os::raw::c_int,
        ) where
            F: FnMut(i32),
        {
            if let Some(callback) = (data as *mut F).as_mut() {
                callback(signal_number);
            }
        }
        let data = &*callback as *const F as *mut _;
        let source = spa_interface_call!(
            self,
            add_signal,
            signal_number,
            Some(callback_call::<F>),
            data
        )?;

        Ok(SignalSource {
            ref_: NonNull::new(source as *mut SourceRef).unwrap(),
            loop_,
            callback,
        })
    }

    pub fn destroy_source(&self, source: &SourceRef) -> crate::Result<()> {
        spa_interface_call!(self, destroy_source, source.as_raw_ptr())?;
        Ok(())
    }
}
