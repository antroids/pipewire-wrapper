use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::object::prop::Prop;
use crate::spa::type_::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{PodBoolRef, PodError, PodSubtype};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ObjectPropInfoType<'a> {
    ID(&'a PodIdRef<Prop>) = PropInfo::ID.raw,
    NAME(&'a PodStringRef) = PropInfo::NAME.raw,
    TYPE(&'a PodChoiceRef) = PropInfo::TYPE.raw,
    LABELS(&'a PodStructRef) = PropInfo::LABELS.raw,
    CONTAINER(&'a PodIdRef<u32>) = PropInfo::CONTAINER.raw,
    PARAMS(&'a PodBoolRef) = PropInfo::PARAMS.raw,
    DESCRIPTION(&'a PodStringRef) = PropInfo::DESCRIPTION.raw,
}

impl<'a> PodPropKeyType<'a> for ObjectPropInfoType<'a> {}

impl<'a> TryFrom<&'a PodPropRef<'a, ObjectPropInfoType<'a>>> for ObjectPropInfoType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ObjectPropInfoType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match PropInfo::from_raw(value.raw.key) {
                PropInfo::ID => Ok(ObjectPropInfoType::ID(value.pod().cast()?)),
                PropInfo::NAME => Ok(ObjectPropInfoType::NAME(value.pod().cast()?)),
                PropInfo::TYPE => Ok(ObjectPropInfoType::TYPE(value.pod().cast()?)),
                PropInfo::LABELS => Ok(ObjectPropInfoType::LABELS(value.pod().cast()?)),
                PropInfo::CONTAINER => Ok(ObjectPropInfoType::CONTAINER(value.pod().cast()?)),
                PropInfo::PARAMS => Ok(ObjectPropInfoType::PARAMS(value.pod().cast()?)),
                PropInfo::DESCRIPTION => Ok(ObjectPropInfoType::DESCRIPTION(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

enum_wrapper!(
    PropInfo,
    spa_sys::spa_prop_info,
    _START: spa_sys::SPA_PROP_INFO_START,
    ID: spa_sys::SPA_PROP_INFO_id,
    NAME: spa_sys::SPA_PROP_INFO_name,
    TYPE: spa_sys::SPA_PROP_INFO_type,
    LABELS: spa_sys::SPA_PROP_INFO_labels,
    CONTAINER: spa_sys::SPA_PROP_INFO_container,
    PARAMS: spa_sys::SPA_PROP_INFO_params,
    DESCRIPTION: spa_sys::SPA_PROP_INFO_description,
);
