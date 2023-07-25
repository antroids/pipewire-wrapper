use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::{BasicTypePod, PodError, PodIntRef, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
pub enum ParamIoType<'a> {
    ID(&'a PodIdRef<IoType>) = ParamIo::ID.raw,
    SIZE(&'a PodIntRef) = ParamIo::SIZE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamIoType<'a>>> for ParamIoType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamIoType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamIo::from_raw(value.raw.key) {
                ParamIo::ID => Ok(ParamIoType::ID(value.pod().cast()?)),
                ParamIo::SIZE => Ok(ParamIoType::SIZE(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamIoType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ParamIoType::ID(pod) => Self::write_pod_prop(buffer, ParamIo::ID.raw, 0, pod),
            ParamIoType::SIZE(pod) => Self::write_pod_prop(buffer, ParamIo::SIZE.raw, 0, pod),
        }
    }
}

enum_wrapper!(
    ParamIo,
    spa_sys::spa_param_io,
    _START: spa_sys::SPA_PARAM_IO_START,
    ID: spa_sys::SPA_PARAM_IO_id,
    SIZE: spa_sys::SPA_PARAM_IO_size,
);

enum_wrapper!(
    IoType,
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
impl PodIdType for IoType {}
