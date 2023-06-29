use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::restricted::PodSubtype;
use crate::spa::type_::pod::{PodError, PodIntRef};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
pub enum ParamMetaType<'a> {
    TYPE(&'a PodIdRef<MetaType>) = ParamMeta::TYPE.raw,
    SIZE(&'a PodIntRef) = ParamMeta::SIZE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamMetaType<'a>>> for ParamMetaType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamMetaType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamMeta::from_raw(value.raw.key) {
                ParamMeta::TYPE => Ok(ParamMetaType::TYPE(value.pod().cast()?)),
                ParamMeta::SIZE => Ok(ParamMetaType::SIZE(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamMetaType<'a> {}

enum_wrapper!(
    ParamMeta,
    spa_sys::spa_param_meta,
    _START: spa_sys::SPA_PARAM_META_START,
    TYPE: spa_sys::SPA_PARAM_META_type,
    SIZE: spa_sys::SPA_PARAM_META_size,
);

enum_wrapper!(
    MetaType,
    spa_sys::spa_meta_type,
    INVALID: spa_sys::SPA_META_Invalid,
    HEADER: spa_sys::SPA_META_Header,
    VIDEOCROP: spa_sys::SPA_META_VideoCrop,
    VIDEODAMAGE: spa_sys::SPA_META_VideoDamage,
    BITMAP: spa_sys::SPA_META_Bitmap,
    CURSOR: spa_sys::SPA_META_Cursor,
    CONTROL: spa_sys::SPA_META_Control,
    BUSY: spa_sys::SPA_META_Busy,
);
impl PodIdType for MetaType {}