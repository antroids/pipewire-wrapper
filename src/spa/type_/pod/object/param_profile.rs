use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::object::{PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{BasicTypePod, PodBoolRef, PodError, PodIntRef};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
pub enum ParamProfileType<'a> {
    INDEX(&'a PodIntRef) = ParamProfile::INDEX.raw,
    NAME(&'a PodStringRef) = ParamProfile::NAME.raw,
    DESCRIPTION(&'a PodStringRef) = ParamProfile::DESCRIPTION.raw,
    PRIORITY(&'a PodIntRef) = ParamProfile::PRIORITY.raw,
    AVAILABLE(&'a PodIdRef<ParamAvailability>) = ParamProfile::AVAILABLE.raw,
    INFO(&'a PodStructRef) = ParamProfile::INFO.raw,
    CLASSES(&'a PodStructRef) = ParamProfile::CLASSES.raw,
    SAVE(&'a PodBoolRef) = ParamProfile::SAVE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamProfileType<'a>>> for ParamProfileType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamProfileType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamProfile::from_raw(value.raw.key) {
                ParamProfile::INDEX => Ok(ParamProfileType::INDEX(value.pod().cast()?)),
                ParamProfile::NAME => Ok(ParamProfileType::NAME(value.pod().cast()?)),
                ParamProfile::DESCRIPTION => Ok(ParamProfileType::DESCRIPTION(value.pod().cast()?)),
                ParamProfile::PRIORITY => Ok(ParamProfileType::PRIORITY(value.pod().cast()?)),
                ParamProfile::AVAILABLE => Ok(ParamProfileType::AVAILABLE(value.pod().cast()?)),
                ParamProfile::INFO => Ok(ParamProfileType::INFO(value.pod().cast()?)),
                ParamProfile::CLASSES => Ok(ParamProfileType::CLASSES(value.pod().cast()?)),
                ParamProfile::SAVE => Ok(ParamProfileType::SAVE(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamProfileType<'a> {}

enum_wrapper!(
    ParamProfile,
    spa_sys::spa_param_profile,
    _START: spa_sys::SPA_PARAM_PROFILE_START,
    INDEX: spa_sys::SPA_PARAM_PROFILE_index,
    NAME: spa_sys::SPA_PARAM_PROFILE_name,
    DESCRIPTION: spa_sys::SPA_PARAM_PROFILE_description,
    PRIORITY: spa_sys::SPA_PARAM_PROFILE_priority,
    AVAILABLE: spa_sys::SPA_PARAM_PROFILE_available,
    INFO: spa_sys::SPA_PARAM_PROFILE_info,
    CLASSES: spa_sys::SPA_PARAM_PROFILE_classes,
    SAVE: spa_sys::SPA_PARAM_PROFILE_save,
);

enum_wrapper!(
    ParamAvailability,
    spa_sys::spa_param_availability,
    UNKNOWN: spa_sys::SPA_PARAM_AVAILABILITY_unknown,
    NO: spa_sys::SPA_PARAM_AVAILABILITY_no,
    YES: spa_sys::SPA_PARAM_AVAILABILITY_yes,
);
impl PodIdType for ParamAvailability {}
