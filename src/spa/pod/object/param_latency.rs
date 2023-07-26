/*
 * SPDX-License-Identifier: MIT
 */
use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{
    BasicTypePod, PodBoolRef, PodError, PodFloatRef, PodIntRef, PodLongRef, PodResult,
};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ParamLatencyType<'a> {
    DIRECTION(&'a PodIdRef<Direction>) = ParamLatency::DIRECTION.raw,
    MIN_QUANTUM(&'a PodFloatRef) = ParamLatency::MIN_QUANTUM.raw,
    MAX_QUANTUM(&'a PodFloatRef) = ParamLatency::MAX_QUANTUM.raw,
    MIN_RATE(&'a PodIntRef) = ParamLatency::MIN_RATE.raw,
    MAX_RATE(&'a PodIntRef) = ParamLatency::MAX_RATE.raw,
    MIN_NS(&'a PodLongRef) = ParamLatency::MIN_NS.raw,
    MAX_NS(&'a PodLongRef) = ParamLatency::MAX_NS.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamLatencyType<'a>>> for ParamLatencyType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ParamLatencyType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match ParamLatency::from_raw(value.raw.key) {
                ParamLatency::DIRECTION => Ok(ParamLatencyType::DIRECTION(value.pod().cast()?)),
                ParamLatency::MIN_QUANTUM => Ok(ParamLatencyType::MIN_QUANTUM(value.pod().cast()?)),
                ParamLatency::MAX_QUANTUM => Ok(ParamLatencyType::MAX_QUANTUM(value.pod().cast()?)),
                ParamLatency::MIN_RATE => Ok(ParamLatencyType::MIN_RATE(value.pod().cast()?)),
                ParamLatency::MAX_RATE => Ok(ParamLatencyType::MAX_RATE(value.pod().cast()?)),
                ParamLatency::MIN_NS => Ok(ParamLatencyType::MIN_NS(value.pod().cast()?)),
                ParamLatency::MAX_NS => Ok(ParamLatencyType::MAX_NS(value.pod().cast()?)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamLatencyType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ParamLatencyType::DIRECTION(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::DIRECTION.raw, 0, pod)
            }
            ParamLatencyType::MIN_QUANTUM(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MIN_QUANTUM.raw, 0, pod)
            }
            ParamLatencyType::MAX_QUANTUM(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MAX_QUANTUM.raw, 0, pod)
            }
            ParamLatencyType::MIN_RATE(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MIN_RATE.raw, 0, pod)
            }
            ParamLatencyType::MAX_RATE(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MAX_RATE.raw, 0, pod)
            }
            ParamLatencyType::MIN_NS(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MIN_NS.raw, 0, pod)
            }
            ParamLatencyType::MAX_NS(pod) => {
                Self::write_pod_prop(buffer, ParamLatency::MAX_NS.raw, 0, pod)
            }
        }
    }
}

enum_wrapper!(
    ParamLatency,
    spa_sys::spa_param_latency,
    _START: spa_sys::SPA_PARAM_LATENCY_START,
    DIRECTION: spa_sys::SPA_PARAM_LATENCY_direction,
    MIN_QUANTUM: spa_sys::SPA_PARAM_LATENCY_minQuantum,
    MAX_QUANTUM: spa_sys::SPA_PARAM_LATENCY_maxQuantum,
    MIN_RATE: spa_sys::SPA_PARAM_LATENCY_minRate,
    MAX_RATE: spa_sys::SPA_PARAM_LATENCY_maxRate,
    MIN_NS: spa_sys::SPA_PARAM_LATENCY_minNs,
    MAX_NS: spa_sys::SPA_PARAM_LATENCY_maxNs,
);
