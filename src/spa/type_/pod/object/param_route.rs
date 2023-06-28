use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::object::param_port_config::Direction;
use crate::spa::type_::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::restricted::PodSubtype;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{PodBoolRef, PodError, PodIntRef};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
pub enum ParamRouteType<'a> {
    INDEX(&'a PodIntRef) = ParamRoute::INDEX.raw,
    DIRECTION(&'a PodIdRef<Direction>) = ParamRoute::DIRECTION.raw,
    DEVICE(&'a PodIntRef) = ParamRoute::DEVICE.raw,
    NAME(&'a PodStringRef) = ParamRoute::NAME.raw,
    DESCRIPTION(&'a PodStringRef) = ParamRoute::DESCRIPTION.raw,
    PRIORITY(&'a PodIntRef) = ParamRoute::PRIORITY.raw,
    AVAILABLE(&'a PodIdRef<ParamAvailability>) = ParamRoute::AVAILABLE.raw,
    INFO(&'a PodStructRef) = ParamRoute::INFO.raw,
    PROFILES(&'a PodIntRef) = ParamRoute::PROFILES.raw,
    PROPS(&'a PodObjectRef) = ParamRoute::PROPS.raw,
    DEVICES(&'a PodIntRef) = ParamRoute::DEVICES.raw,
    PROFILE(&'a PodIntRef) = ParamRoute::PROFILE.raw,
    SAVE(&'a PodBoolRef) = ParamRoute::SAVE.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamRouteType<'a>>> for ParamRouteType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamRouteType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamRoute::from_raw(value.raw.key) {
                ParamRoute::INDEX => Ok(ParamRouteType::INDEX(value.pod().cast()?)),
                ParamRoute::DIRECTION => Ok(ParamRouteType::DIRECTION(value.pod().cast()?)),
                ParamRoute::DEVICE => Ok(ParamRouteType::DEVICE(value.pod().cast()?)),
                ParamRoute::DESCRIPTION => Ok(ParamRouteType::DESCRIPTION(value.pod().cast()?)),
                ParamRoute::PRIORITY => Ok(ParamRouteType::PRIORITY(value.pod().cast()?)),
                ParamRoute::AVAILABLE => Ok(ParamRouteType::AVAILABLE(value.pod().cast()?)),
                ParamRoute::INFO => Ok(ParamRouteType::INFO(value.pod().cast()?)),
                ParamRoute::PROFILES => Ok(ParamRouteType::PROFILES(value.pod().cast()?)),
                ParamRoute::PROPS => Ok(ParamRouteType::PROPS(value.pod().cast()?)),
                ParamRoute::DEVICES => Ok(ParamRouteType::DEVICES(value.pod().cast()?)),
                ParamRoute::PROFILE => Ok(ParamRouteType::PROFILE(value.pod().cast()?)),
                ParamRoute::SAVE => Ok(ParamRouteType::SAVE(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamRouteType<'a> {}

enum_wrapper!(
    ParamRoute,
    spa_sys::spa_param_route,
    _START: spa_sys::SPA_PARAM_ROUTE_START,
    INDEX: spa_sys::SPA_PARAM_ROUTE_index,
    DIRECTION: spa_sys::SPA_PARAM_ROUTE_direction,
    DEVICE: spa_sys::SPA_PARAM_ROUTE_device,
    NAME: spa_sys::SPA_PARAM_ROUTE_name,
    DESCRIPTION: spa_sys::SPA_PARAM_ROUTE_description,
    PRIORITY: spa_sys::SPA_PARAM_ROUTE_priority,
    AVAILABLE: spa_sys::SPA_PARAM_ROUTE_available,
    INFO: spa_sys::SPA_PARAM_ROUTE_info,
    PROFILES: spa_sys::SPA_PARAM_ROUTE_profiles,
    PROPS: spa_sys::SPA_PARAM_ROUTE_props,
    DEVICES: spa_sys::SPA_PARAM_ROUTE_devices,
    PROFILE: spa_sys::SPA_PARAM_ROUTE_profile,
    SAVE: spa_sys::SPA_PARAM_ROUTE_save,
);

enum_wrapper!(
    ParamAvailability,
    spa_sys::spa_param_availability,
    UNKNOWN: spa_sys::SPA_PARAM_AVAILABILITY_unknown,
    NO: spa_sys::SPA_PARAM_AVAILABILITY_no,
    YES: spa_sys::SPA_PARAM_AVAILABILITY_yes,
);
impl PodIdType for ParamAvailability {}
