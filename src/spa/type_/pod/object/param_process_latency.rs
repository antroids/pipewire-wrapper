use std::io::{Seek, Write};

use pipewire_macro_impl::enum_wrapper;

use crate::spa::type_::pod::id::{PodIdRef, PodIdType};
use crate::spa::type_::pod::object::param_port_config::Direction;
use crate::spa::type_::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{
    BasicTypePod, PodBoolRef, PodError, PodFloatRef, PodIntRef, PodLongRef, PodResult,
};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ParamProcessLatencyType<'a> {
    QUANTUM(&'a PodFloatRef) = ParamProcessLatency::QUANTUM.raw,
    RATE(&'a PodIntRef) = ParamProcessLatency::RATE.raw,
    NS(&'a PodLongRef) = ParamProcessLatency::NS.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ParamProcessLatencyType<'a>>> for ParamProcessLatencyType<'a> {
    type Error = PodError;

    fn try_from(
        value: &'a PodPropRef<'a, ParamProcessLatencyType<'a>>,
    ) -> Result<Self, Self::Error> {
        unsafe {
            match ParamProcessLatency::from_raw(value.raw.key) {
                ParamProcessLatency::QUANTUM => {
                    Ok(ParamProcessLatencyType::QUANTUM(value.pod().cast()?))
                }
                ParamProcessLatency::RATE => Ok(ParamProcessLatencyType::RATE(value.pod().cast()?)),
                ParamProcessLatency::NS => Ok(ParamProcessLatencyType::NS(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ParamProcessLatencyType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ParamProcessLatencyType::QUANTUM(pod) => {
                Self::write_pod_prop(buffer, ParamProcessLatency::QUANTUM.raw, 0, pod)
            }
            ParamProcessLatencyType::RATE(pod) => {
                Self::write_pod_prop(buffer, ParamProcessLatency::RATE.raw, 0, pod)
            }
            ParamProcessLatencyType::NS(pod) => {
                Self::write_pod_prop(buffer, ParamProcessLatency::NS.raw, 0, pod)
            }
        }
    }
}

enum_wrapper!(
    ParamProcessLatency,
    spa_sys::spa_param_process_latency,
    _START: spa_sys::SPA_PARAM_PROCESS_LATENCY_START,
    QUANTUM: spa_sys::SPA_PARAM_PROCESS_LATENCY_quantum,
    RATE: spa_sys::SPA_PARAM_PROCESS_LATENCY_rate,
    NS: spa_sys::SPA_PARAM_PROCESS_LATENCY_ns,
);
