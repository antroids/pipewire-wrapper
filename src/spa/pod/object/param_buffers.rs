use std::io::{Seek, Write};

use pipewire_wrapper_proc_macro::object_info;

use crate::enum_wrapper;
use crate::spa::pod::choice::PodChoiceRef;
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::{BasicTypePod, PodError, PodIntRef, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
#[object_info(OBJECT_PARAM_BUFFERS)]
pub enum ParamBuffersType<'a> {
    BUFFERS(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::BUFFERS.raw,
    BLOCKS(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::BLOCKS.raw,
    SIZE(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::SIZE.raw,
    STRIDE(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::STRIDE.raw,
    ALIGN(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::ALIGN.raw,
    DATATYPE(&'a PodChoiceRef<PodIntRef>) = ParamBuffers::DATATYPE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamBuffersType<'a>>> for ParamBuffersType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamBuffersType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamBuffers::from_raw(value.raw.key) {
                ParamBuffers::BUFFERS => Ok(ParamBuffersType::BUFFERS(value.pod().cast()?)),
                ParamBuffers::BLOCKS => Ok(ParamBuffersType::BLOCKS(value.pod().cast()?)),
                ParamBuffers::SIZE => Ok(ParamBuffersType::SIZE(value.pod().cast()?)),
                ParamBuffers::STRIDE => Ok(ParamBuffersType::STRIDE(value.pod().cast()?)),
                ParamBuffers::ALIGN => Ok(ParamBuffersType::ALIGN(value.pod().cast()?)),
                ParamBuffers::DATATYPE => Ok(ParamBuffersType::DATATYPE(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamBuffersType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<()>
    where
        W: Write + Seek,
    {
        match self {
            ParamBuffersType::BUFFERS(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::BUFFERS.raw, 0, pod)
            }
            ParamBuffersType::BLOCKS(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::BLOCKS.raw, 0, pod)
            }
            ParamBuffersType::SIZE(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::SIZE.raw, 0, pod)
            }
            ParamBuffersType::STRIDE(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::STRIDE.raw, 0, pod)
            }
            ParamBuffersType::ALIGN(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::ALIGN.raw, 0, pod)
            }
            ParamBuffersType::DATATYPE(pod) => {
                Self::write_pod_prop(buffer, ParamBuffers::DATATYPE.raw, 0, pod)
            }
        }
    }
}

enum_wrapper!(
    ParamBuffers,
    spa_sys::spa_param_buffers,
    _START: spa_sys::SPA_PARAM_BUFFERS_START,
    BUFFERS: spa_sys::SPA_PARAM_BUFFERS_buffers,
    BLOCKS: spa_sys::SPA_PARAM_BUFFERS_blocks,
    SIZE: spa_sys::SPA_PARAM_BUFFERS_size,
    STRIDE: spa_sys::SPA_PARAM_BUFFERS_stride,
    ALIGN: spa_sys::SPA_PARAM_BUFFERS_align,
    DATATYPE: spa_sys::SPA_PARAM_BUFFERS_dataType,
);
