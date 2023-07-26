/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::choice::PodChoiceRef;
use crate::spa::pod::id::PodIdRef;
use crate::spa::pod::object::prop::Prop;
use crate::spa::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{BasicTypePod, PodBoolRef, PodError, PodRef, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ObjectPropInfoType<'a> {
    ID(&'a PodIdRef<Prop>) = PropInfo::ID.raw,
    NAME(&'a PodStringRef) = PropInfo::NAME.raw,
    TYPE(&'a PodRef) = PropInfo::TYPE.raw,
    LABELS(&'a PodStructRef) = PropInfo::LABELS.raw,
    CONTAINER(&'a PodIdRef<u32>) = PropInfo::CONTAINER.raw,
    PARAMS(&'a PodBoolRef) = PropInfo::PARAMS.raw,
    DESCRIPTION(&'a PodStringRef) = PropInfo::DESCRIPTION.raw,
}

impl<'a> PodPropKeyType<'a> for ObjectPropInfoType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ObjectPropInfoType::ID(pod) => Self::write_pod_prop(buffer, PropInfo::ID.raw, 0, pod),
            ObjectPropInfoType::NAME(pod) => {
                Self::write_pod_prop(buffer, PropInfo::NAME.raw, 0, pod)
            }
            ObjectPropInfoType::TYPE(pod) => {
                Self::write_pod_prop(buffer, PropInfo::TYPE.raw, 0, pod)
            }
            ObjectPropInfoType::LABELS(pod) => {
                Self::write_pod_prop(buffer, PropInfo::LABELS.raw, 0, pod)
            }
            ObjectPropInfoType::CONTAINER(pod) => {
                Self::write_pod_prop(buffer, PropInfo::CONTAINER.raw, 0, pod)
            }
            ObjectPropInfoType::PARAMS(pod) => {
                Self::write_pod_prop(buffer, PropInfo::PARAMS.raw, 0, pod)
            }
            ObjectPropInfoType::DESCRIPTION(pod) => {
                Self::write_pod_prop(buffer, PropInfo::DESCRIPTION.raw, 0, pod)
            }
        }
    }
}

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
                _ => Err(PodError::UnknownPodTypeToDowncast),
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
