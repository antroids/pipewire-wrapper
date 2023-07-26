/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::array::PodArrayRef;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{BasicTypePod, PodBoolRef, PodError, PodIntRef, PodResult};
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
    PROFILES(&'a PodArrayRef<PodIntRef>) = ParamRoute::PROFILES.raw,
    PROPS(&'a PodObjectRef) = ParamRoute::PROPS.raw,
    DEVICES(&'a PodArrayRef<PodIntRef>) = ParamRoute::DEVICES.raw,
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
                ParamRoute::NAME => Ok(ParamRouteType::NAME(value.pod().cast()?)),
                ParamRoute::DESCRIPTION => Ok(ParamRouteType::DESCRIPTION(value.pod().cast()?)),
                ParamRoute::PRIORITY => Ok(ParamRouteType::PRIORITY(value.pod().cast()?)),
                ParamRoute::AVAILABLE => Ok(ParamRouteType::AVAILABLE(value.pod().cast()?)),
                ParamRoute::INFO => Ok(ParamRouteType::INFO(value.pod().cast()?)),
                ParamRoute::PROFILES => Ok(ParamRouteType::PROFILES(value.pod().cast()?)),
                ParamRoute::PROPS => Ok(ParamRouteType::PROPS(value.pod().cast()?)),
                ParamRoute::DEVICES => Ok(ParamRouteType::DEVICES(value.pod().cast()?)),
                ParamRoute::PROFILE => Ok(ParamRouteType::PROFILE(value.pod().cast()?)),
                ParamRoute::SAVE => Ok(ParamRouteType::SAVE(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamRouteType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<()>
    where
        W: Write + Seek,
    {
        match self {
            ParamRouteType::INDEX(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::INDEX.raw, 0, pod)
            }
            ParamRouteType::DIRECTION(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::DIRECTION.raw, 0, pod)
            }
            ParamRouteType::DEVICE(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::DEVICE.raw, 0, pod)
            }
            ParamRouteType::NAME(pod) => Self::write_pod_prop(buffer, ParamRoute::NAME.raw, 0, pod),
            ParamRouteType::DESCRIPTION(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::DESCRIPTION.raw, 0, pod)
            }
            ParamRouteType::PRIORITY(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::PRIORITY.raw, 0, pod)
            }
            ParamRouteType::AVAILABLE(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::AVAILABLE.raw, 0, pod)
            }
            ParamRouteType::INFO(pod) => Self::write_pod_prop(buffer, ParamRoute::INFO.raw, 0, pod),
            ParamRouteType::PROFILES(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::PROFILES.raw, 0, pod)
            }
            ParamRouteType::PROPS(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::PROPS.raw, 0, pod)
            }
            ParamRouteType::DEVICES(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::DEVICES.raw, 0, pod)
            }
            ParamRouteType::PROFILE(pod) => {
                Self::write_pod_prop(buffer, ParamRoute::PROFILE.raw, 0, pod)
            }
            ParamRouteType::SAVE(pod) => Self::write_pod_prop(buffer, ParamRoute::SAVE.raw, 0, pod),
        }
    }
}

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
