/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};

use pipewire_wrapper_proc_macro::ObjectInfo;

use crate::enum_wrapper;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::pod::{BasicTypePod, PodBoolRef, PodError, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug, ObjectInfo)]
pub enum ParamPortConfigType<'a> {
    DIRECTION(&'a PodIdRef<Direction>) = ParamPortConfig::DIRECTION.raw,
    MODE(&'a PodIdRef<ParamPortConfigMode>) = ParamPortConfig::MODE.raw,
    MONITOR(&'a PodBoolRef) = ParamPortConfig::MONITOR.raw,
    CONTROL(&'a PodBoolRef) = ParamPortConfig::CONTROL.raw,
    FORMAT(&'a PodObjectRef) = ParamPortConfig::FORMAT.raw, // Format object
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamPortConfigType<'a>>> for ParamPortConfigType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamPortConfigType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamPortConfig::from_raw(value.raw.key) {
                ParamPortConfig::DIRECTION => {
                    Ok(ParamPortConfigType::DIRECTION(value.pod().cast()?))
                }
                ParamPortConfig::MODE => Ok(ParamPortConfigType::MODE(value.pod().cast()?)),
                ParamPortConfig::MONITOR => Ok(ParamPortConfigType::MONITOR(value.pod().cast()?)),
                ParamPortConfig::CONTROL => Ok(ParamPortConfigType::CONTROL(value.pod().cast()?)),
                ParamPortConfig::FORMAT => Ok(ParamPortConfigType::FORMAT(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamPortConfigType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<()>
    where
        W: Write + Seek,
    {
        match self {
            ParamPortConfigType::DIRECTION(pod) => {
                Self::write_pod_prop(buffer, ParamPortConfig::DIRECTION.raw, 0, pod)
            }
            ParamPortConfigType::MODE(pod) => {
                Self::write_pod_prop(buffer, ParamPortConfig::MODE.raw, 0, pod)
            }
            ParamPortConfigType::MONITOR(pod) => {
                Self::write_pod_prop(buffer, ParamPortConfig::MONITOR.raw, 0, pod)
            }
            ParamPortConfigType::CONTROL(pod) => {
                Self::write_pod_prop(buffer, ParamPortConfig::CONTROL.raw, 0, pod)
            }
            ParamPortConfigType::FORMAT(pod) => {
                Self::write_pod_prop(buffer, ParamPortConfig::FORMAT.raw, 0, pod)
            }
        }
    }
}

enum_wrapper!(
    ParamPortConfig,
    spa_sys::spa_param_port_config,
    _START: spa_sys::SPA_PARAM_PORT_CONFIG_START,
    DIRECTION: spa_sys::SPA_PARAM_PORT_CONFIG_direction,
    MODE: spa_sys::SPA_PARAM_PORT_CONFIG_mode,
    MONITOR: spa_sys::SPA_PARAM_PORT_CONFIG_monitor,
    CONTROL: spa_sys::SPA_PARAM_PORT_CONFIG_control,
    FORMAT: spa_sys::SPA_PARAM_PORT_CONFIG_format,
);

enum_wrapper!(
    Direction,
    spa_sys::spa_direction,
    INPUT: spa_sys::SPA_DIRECTION_INPUT,
    OUTPUT: spa_sys::SPA_DIRECTION_OUTPUT,
);
impl PodIdType for Direction {}

enum_wrapper!(
    ParamPortConfigMode,
    spa_sys::spa_param_port_config_mode,
    NONE: spa_sys::SPA_PARAM_PORT_CONFIG_MODE_none,
    PASSTHROUGH: spa_sys::SPA_PARAM_PORT_CONFIG_MODE_passthrough,
    CONVERT: spa_sys::SPA_PARAM_PORT_CONFIG_MODE_convert,
    DSP: spa_sys::SPA_PARAM_PORT_CONFIG_MODE_dsp,
);
impl PodIdType for ParamPortConfigMode {}
