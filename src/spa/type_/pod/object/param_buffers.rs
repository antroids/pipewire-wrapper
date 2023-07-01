use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::{BasicTypePod, PodError, PodIntRef};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ParamBuffersType<'a> {
    BUFFERS(&'a PodIntRef) = ParamBuffers::BUFFERS.raw,
    BLOCKS(&'a PodIntRef) = ParamBuffers::BLOCKS.raw,
    SIZE(&'a PodIntRef) = ParamBuffers::SIZE.raw,
    STRIDE(&'a PodIntRef) = ParamBuffers::STRIDE.raw,
    ALIGN(&'a PodIntRef) = ParamBuffers::ALIGN.raw,
    DATATYPE(&'a PodIntRef) = ParamBuffers::DATATYPE.raw,
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
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamBuffersType<'a> {}

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
