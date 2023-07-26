/*
 * SPDX-License-Identifier: MIT
 */
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
use std::mem::size_of;
use std::pin::Pin;
use std::ptr::{addr_of, addr_of_mut, null_mut, NonNull};
use std::slice;

use bitflags::{bitflags, Flags};

use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::core_api::core::{Core, CoreRef};
use crate::core_api::properties::{Properties, PropertiesRef};
use crate::enum_wrapper;
use crate::filter::events::FilterEvents;
use crate::listeners::{AddListener, Listeners, OwnListeners};
use crate::spa::dict::DictRef;
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::pod::PodRef;
use crate::stream::buffer::BufferRef;
use crate::wrapper::{RawWrapper, Wrapper};
use crate::{i32_as_result, i32_as_void_result, new_instance_raw_wrapper};

pub mod events;

/// Identifier for the port in the filter.
#[derive(Debug)]
pub struct FilterPortId<T> {
    ptr: NonNull<*mut T>,
}

impl<T> PartialEq for FilterPortId<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr.eq(&other.ptr)
    }
}

impl<T> Eq for FilterPortId<T> {}

impl<T> Clone for FilterPortId<T> {
    fn clone(&self) -> Self {
        Self { ptr: self.ptr }
    }
}

impl<T> Hash for FilterPortId<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ptr.hash(state)
    }
}

impl<T> FilterPortId<T> {
    fn new(ptr: NonNull<*mut T>) -> Self {
        Self { ptr }
    }

    fn as_ptr(&self) -> *mut *mut T {
        self.ptr.as_ptr()
    }
}

/// Wrapper for the external [pw_sys::pw_filter] value.
/// The filter object provides a convenient way to implement processing filters.
#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct FilterRef<T> {
    #[raw]
    raw: pw_sys::pw_filter,
    phantom: PhantomData<T>,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct FilterFlags: u32 {
        /// no flags
        const NONE = pw_sys::pw_filter_flags_PW_FILTER_FLAG_NONE;
        /// start the filter inactive, pw_filter_set_active() needs to be called explicitly
        const INACTIVE = pw_sys::pw_filter_flags_PW_FILTER_FLAG_INACTIVE;
        /// be a driver
        const DRIVER = pw_sys::pw_filter_flags_PW_FILTER_FLAG_DRIVER;
        /// call process from the realtime thread
        const RT_PROCESS = pw_sys::pw_filter_flags_PW_FILTER_FLAG_RT_PROCESS;
        /// don't call the default latency algorithm but emit the param_changed event
        /// for the ports when Latency params are received.
        const CUSTOM_LATENCY = pw_sys::pw_filter_flags_PW_FILTER_FLAG_CUSTOM_LATENCY;
    }
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct PortFlags: u32 {
        /// no flags
        const NONE = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_NONE;
        /// the application will allocate buffer memory. In the add_buffer event, the data
        /// of the buffer should be set
        const ALLOC_BUFFERS = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_ALLOC_BUFFERS;
        /// map the buffers except DmaBuf
        const MAP_BUFFERS = pw_sys::pw_filter_port_flags_PW_FILTER_PORT_FLAG_MAP_BUFFERS;
    }
}

enum_wrapper!(
    FilterState,
    pw_sys::pw_filter_state,
    ERROR: pw_sys::pw_filter_state_PW_FILTER_STATE_ERROR,
    UNCONNECTED: pw_sys::pw_filter_state_PW_FILTER_STATE_UNCONNECTED,
    CONNECTING: pw_sys::pw_filter_state_PW_FILTER_STATE_CONNECTING,
    PAUSED: pw_sys::pw_filter_state_PW_FILTER_STATE_PAUSED,
    STREAMING: pw_sys::pw_filter_state_PW_FILTER_STATE_STREAMING,
);

impl FilterState {
    /// Convert a filter state to a readable string
    pub fn state_as_string(&self) -> &CStr {
        unsafe { CStr::from_ptr(pw_sys::pw_filter_state_as_string(self.raw)) }
    }
}

impl<T> FilterRef<T> {
    pub fn get_state_and_error(&self) -> (FilterState, Option<CString>) {
        let mut error_ptr = null_mut();
        unsafe {
            let state =
                FilterState::from_raw(pw_sys::pw_filter_get_state(self.as_raw_ptr(), error_ptr));
            let error_c_str = error_ptr.as_mut().map(|ptr| CStr::from_ptr(*ptr));
            let error = error_c_str.map(CString::from);
            (state, error)
        }
    }

    pub fn get_state(&self) -> FilterState {
        self.get_state_and_error().0
    }

    pub fn get_error(&self) -> Option<CString> {
        self.get_state_and_error().1
    }

    pub fn get_name(&self) -> Option<&CStr> {
        unsafe {
            pw_sys::pw_filter_get_name(self.as_raw_ptr())
                .as_ref()
                .map(|ptr| CStr::from_ptr(ptr))
        }
    }

    pub fn get_core(&self) -> &CoreRef {
        unsafe { CoreRef::from_raw_ptr(pw_sys::pw_filter_get_core(self.as_raw_ptr())) }
    }

    /// Connect a filter for processing.
    pub fn connect(&self, flags: FilterFlags, params: &[&PodRef]) -> crate::Result<()> {
        let result = unsafe {
            pw_sys::pw_filter_connect(
                self.as_raw_ptr(),
                flags.bits(),
                params.as_ptr() as *const *const spa_sys::spa_pod as *mut *const spa_sys::spa_pod,
                params.len() as u32,
            )
        };
        i32_as_void_result(result)
    }

    pub fn get_node_id(&self) -> u32 {
        unsafe { pw_sys::pw_filter_get_node_id(self.as_raw_ptr()) }
    }

    /// Disconnect the filter and stop processing.
    pub fn disconnect(&self) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_filter_disconnect(self.as_raw_ptr()) };
        i32_as_void_result(result)
    }

    /// Add port to the filter.
    ///
    /// # Arguments
    ///
    /// * `direction` - port direction
    /// * `flags` - port flags
    /// * `props` - port properties
    /// * `params` - port parameters
    ///
    /// Returns a mut pointer to user data.
    ///
    /// # Safety
    ///
    /// Result is the pointer to the port structure and it should not be modified.
    pub unsafe fn add_port(
        &self,
        direction: Direction,
        flags: PortFlags,
        props: Properties,
        params: Option<&[&PodRef]>,
    ) -> crate::Result<*mut *mut T> {
        let result = unsafe {
            let param_ptr = params.map_or(null_mut(), |p| {
                p.as_ptr() as *const *const spa_sys::spa_pod as *mut *const spa_sys::spa_pod
            });
            let param_len = params.map_or(0, |p| p.len() as u32);
            pw_sys::pw_filter_add_port(
                self.as_raw_ptr(),
                direction.raw,
                flags.bits(),
                size_of::<*mut T>(),
                props.into_raw(),
                param_ptr,
                param_len,
            )
        };
        if result.is_null() {
            Err(crate::Error::NullPointer)
        } else {
            Ok(result.cast())
        }
    }

    /// Remove the port.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn remove_port(&self, port: *mut *mut T) -> crate::Result<()> {
        i32_as_void_result(pw_sys::pw_filter_remove_port(port.cast()))
    }

    /// Get port properties or global properties when `port` is `null`
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn get_properties(&self, port: *mut *mut T) -> &PropertiesRef {
        PropertiesRef::from_raw_ptr(pw_sys::pw_filter_get_properties(
            self.as_raw_ptr(),
            port.cast(),
        ))
    }

    /// Update port properties or global properties when `port` is `null`.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn update_properties(&self, port: *mut *mut T, properties: &DictRef) -> i32 {
        pw_sys::pw_filter_update_properties(self.as_raw_ptr(), port.cast(), properties.as_raw_ptr())
    }

    pub fn set_error(&self, res: i32, error: &CStr) {
        unsafe { pw_sys::pw_filter_set_error(self.as_raw_ptr(), res, error.as_ptr()) };
    }

    /// Update port parameters or global parameters when `port` is `null`.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn update_params(&self, port: *mut *mut T, params: &[&PodRef]) -> crate::Result<()> {
        let params_ptr = params as *const [&PodRef] as *mut *const spa_sys::spa_pod;
        let result = pw_sys::pw_filter_update_params(
            self.as_raw_ptr(),
            port.cast(),
            params_ptr,
            params.len() as u32,
        );
        i32_as_void_result(result)
    }

    /// Get a buffer that can be filled for output ports or consumed for input ports.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn dequeue_buffer(&self, port: *mut *mut T) -> Option<&mut BufferRef> {
        pw_sys::pw_filter_dequeue_buffer(port.cast())
            .as_mut()
            .map(|ptr| BufferRef::mut_from_raw_ptr(ptr))
    }

    /// Submit a buffer for playback or recycle a buffer for capture.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn queue_buffer(&self, port: *mut *mut T, buffer: &BufferRef) -> crate::Result<()> {
        let result = pw_sys::pw_filter_queue_buffer(port.cast(), buffer.as_raw_ptr());
        i32_as_void_result(result)
    }

    /// Get a data pointer to the buffer data.
    ///
    /// # Safety
    ///
    /// `port` is a pointer to the port structure and it should not be modified.
    pub unsafe fn get_dsp_buffer<S: Sized>(
        &self,
        port: *mut *mut T,
        n_samples: u32,
    ) -> crate::Result<&mut [S]> {
        let mut result = pw_sys::pw_filter_get_dsp_buffer(port.cast(), n_samples);
        if result.is_null() {
            Err(crate::Error::NullPointer)
        } else {
            Ok(slice::from_raw_parts_mut(result.cast(), n_samples as usize))
        }
    }

    /// Activate or deactivate the filter
    pub fn set_active(&self, active: bool) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_filter_set_active(self.as_raw_ptr(), active) };
        i32_as_void_result(result)
    }

    /// Flush a filter.
    /// When drain is true, the drained callback will be called when all data is played or recorded
    pub fn flush(&self, drain: bool) -> crate::Result<()> {
        let result = unsafe { pw_sys::pw_filter_flush(self.as_raw_ptr(), drain) };
        i32_as_void_result(result)
    }
}

impl<'a, T: 'a> AddListener<'a> for FilterRef<T> {
    type Events = FilterEvents<'a, T>;

    fn add_listener(&self, events: Pin<Box<Self::Events>>) -> Pin<Box<Self::Events>> {
        unsafe {
            pw_sys::pw_filter_add_listener(
                self.as_raw_ptr(),
                events.hook().as_raw_ptr(),
                events.as_raw_ptr(),
                &*events as *const _ as *mut _,
            )
        };

        events
    }
}

#[derive(Wrapper, Debug)]
pub struct Filter<'a, T> {
    #[raw_wrapper]
    ref_: NonNull<FilterRef<T>>,

    core: &'a Core,
    listeners: Listeners<Pin<Box<FilterEvents<'a, T>>>>,
    ports: HashMap<FilterPortId<T>, Pin<Box<T>>>,
}

impl<'a, T> Filter<'a, T> {
    /// Create an unconnected filter
    pub fn new(core: &'a Core, name: &'a CStr, properties: Properties) -> crate::Result<Self> {
        let result = unsafe {
            pw_sys::pw_filter_new(core.as_raw_ptr(), name.as_ptr(), properties.into_raw())
        };
        new_instance_raw_wrapper(result).map(|ref_| Self {
            ref_,
            core,
            listeners: Default::default(),
            ports: Default::default(),
        })
    }

    pub fn add_port(
        &mut self,
        port_data: T,
        direction: Direction,
        flags: PortFlags,
        props: Properties,
        params: Option<&[&PodRef]>,
    ) -> crate::Result<FilterPortId<T>> {
        let result = unsafe { self.as_ref().add_port(direction, flags, props, params) };
        let mut port_data = Box::new(port_data);
        unsafe {
            result.map(|mut ptr| {
                let box_ptr = (&mut *port_data) as *mut T; // Is this safe to take the reference before pin?
                ptr.write(box_ptr);
                let key = FilterPortId::new(NonNull::new_unchecked(ptr));
                self.ports
                    .insert(key.clone(), Pin::new_unchecked(port_data));
                key
            })
        }
    }

    pub fn remove_port(&mut self, port_id: &FilterPortId<T>) -> crate::Result<Pin<Box<T>>> {
        unsafe {
            self.as_ref().remove_port(port_id.as_ptr()).and_then(|_| {
                if let Some(port_data) = self.ports.remove(port_id) {
                    Ok(port_data)
                } else {
                    Err(crate::Error::ErrorMessage("Cannot find port in filter"))
                }
            })
        }
    }

    pub fn contains_port(&self, port_id: &FilterPortId<T>) -> bool {
        self.ports.contains_key(port_id)
    }

    fn try_port_as_ptr(&self, port_id: Option<&FilterPortId<T>>) -> crate::Result<*mut *mut T> {
        Ok(if let Some(port_id) = port_id {
            if self.contains_port(port_id) {
                port_id.as_ptr()
            } else {
                return Err(crate::Error::ErrorMessage("Port not found in the filter"));
            }
        } else {
            null_mut()
        })
    }

    pub fn get_properties(
        &self,
        port_id: Option<&FilterPortId<T>>,
    ) -> crate::Result<&PropertiesRef> {
        unsafe { Ok(self.as_ref().get_properties(self.try_port_as_ptr(port_id)?)) }
    }

    pub fn update_properties(
        &self,
        port_id: Option<&FilterPortId<T>>,
        properties: &DictRef,
    ) -> crate::Result<i32> {
        unsafe {
            Ok(self
                .as_ref()
                .update_properties(self.try_port_as_ptr(port_id)?, properties))
        }
    }

    pub fn update_params(
        &self,
        port_id: Option<&FilterPortId<T>>,
        params: &[&PodRef],
    ) -> crate::Result<()> {
        unsafe {
            self.as_ref()
                .update_params(self.try_port_as_ptr(port_id)?, params)
        }
    }

    pub fn dequeue_buffer(&self, port_id: &FilterPortId<T>) -> Option<&mut BufferRef> {
        if self.contains_port(port_id) {
            unsafe { self.as_ref().dequeue_buffer(port_id.as_ptr()) }
        } else {
            None
        }
    }

    pub fn queue_buffer(&self, port_id: &FilterPortId<T>, buffer: &BufferRef) -> crate::Result<()> {
        unsafe {
            self.as_ref()
                .queue_buffer(self.try_port_as_ptr(Some(port_id))?, buffer)
        }
    }

    pub fn get_dsp_buffer<S: Sized>(
        &self,
        port_id: &FilterPortId<T>,
        n_samples: u32,
    ) -> crate::Result<&mut [S]> {
        unsafe {
            self.as_ref()
                .get_dsp_buffer(self.try_port_as_ptr(Some(port_id))?, n_samples)
        }
    }

    pub fn get_port(&self, port_id: &FilterPortId<T>) -> Option<&Pin<Box<T>>> {
        self.ports.get(port_id)
    }

    pub fn get_port_mut(&mut self, port_id: &FilterPortId<T>) -> Option<&mut Pin<Box<T>>> {
        self.ports.get_mut(port_id)
    }
}

impl<'a, T> Drop for Filter<'a, T> {
    fn drop(&mut self) {
        unsafe { pw_sys::pw_filter_destroy(self.as_raw_ptr()) }
    }
}

impl<'a, T: 'a> OwnListeners<'a> for Filter<'a, T> {
    fn listeners(
        &self,
    ) -> &Listeners<Pin<Box<<<Self as Wrapper>::RawWrapperType as AddListener<'a>>::Events>>> {
        &self.listeners
    }
}
