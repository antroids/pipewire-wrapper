/*
 * SPDX-License-Identifier: MIT
 */
use core::slice;

use pipewire_wrapper_proc_macro::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct ControlRef {
    #[raw]
    raw: pw_sys::pw_stream_control,
}

impl ControlRef {
    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn def(&self) -> f32 {
        self.raw.def
    }

    pub fn min(&self) -> f32 {
        self.raw.min
    }

    pub fn max(&self) -> f32 {
        self.raw.max
    }

    pub fn values(&self) -> &[f32] {
        unsafe { slice::from_raw_parts(self.raw.values, self.raw.n_values as usize) }
    }

    pub fn max_values(&self) -> u32 {
        self.raw.max_values
    }
}

#[derive(Clone, Debug)]
pub struct Control {
    flags: u32,
    def: f32,
    min: f32,
    max: f32,
    values: Vec<f32>,
    max_values: u32,
}

impl Control {
    pub fn from_ref(ref_: &ControlRef) -> Self {
        Self {
            flags: ref_.flags(),
            def: ref_.def(),
            min: ref_.min(),
            max: ref_.max(),
            values: Vec::from(ref_.values()),
            max_values: ref_.max_values(),
        }
    }

    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn def(&self) -> f32 {
        self.def
    }
    pub fn min(&self) -> f32 {
        self.min
    }
    pub fn max(&self) -> f32 {
        self.max
    }
    pub fn values(&self) -> &Vec<f32> {
        &self.values
    }
    pub fn max_values(&self) -> u32 {
        self.max_values
    }
}

impl From<&ControlRef> for Control {
    fn from(value: &ControlRef) -> Self {
        Control::from_ref(value)
    }
}
