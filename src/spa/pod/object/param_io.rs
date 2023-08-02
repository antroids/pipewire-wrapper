/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};

use pipewire_wrapper_proc_macro::object_type_impl;

use crate::enum_wrapper;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::{BasicTypePod, PodError, PodIntRef, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[object_type_impl(OBJECT_PARAM_IO)]
pub enum ParamIOType<'a> {
    ID(&'a PodIdRef<IOType>) = ParamIO::ID.raw,
    SIZE(&'a PodIntRef) = ParamIO::SIZE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamIOType<'a>>> for ParamIOType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamIOType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamIO::from_raw(value.raw.key) {
                ParamIO::ID => Ok(ParamIOType::ID(value.pod().cast()?)),
                ParamIO::SIZE => Ok(ParamIOType::SIZE(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamIOType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<()>
    where
        W: Write + Seek,
    {
        match self {
            ParamIOType::ID(pod) => Self::write_pod_prop(buffer, ParamIO::ID.raw, 0, pod),
            ParamIOType::SIZE(pod) => Self::write_pod_prop(buffer, ParamIO::SIZE.raw, 0, pod),
        }
    }
}

enum_wrapper!(
    ParamIO,
    spa_sys::spa_param_io,
    _START: spa_sys::SPA_PARAM_IO_START,
    ID: spa_sys::SPA_PARAM_IO_id,
    SIZE: spa_sys::SPA_PARAM_IO_size,
);

enum_wrapper!(
    IOType,
    spa_sys::spa_io_type,
    INVALID: spa_sys::SPA_IO_Invalid,
    BUFFERS: spa_sys::SPA_IO_Buffers,
    RANGE: spa_sys::SPA_IO_Range,
    CLOCK: spa_sys::SPA_IO_Clock,
    LATENCY: spa_sys::SPA_IO_Latency,
    CONTROL: spa_sys::SPA_IO_Control,
    NOTIFY: spa_sys::SPA_IO_Notify,
    POSITION: spa_sys::SPA_IO_Position,
    RATEMATCH: spa_sys::SPA_IO_RateMatch,
    MEMORY: spa_sys::SPA_IO_Memory,
);
impl PodIdType for IOType {}
