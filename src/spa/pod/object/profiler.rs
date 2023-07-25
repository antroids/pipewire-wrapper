use std::io::{Seek, Write};

use crate::enum_wrapper;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::object::param_port_config::Direction;
use crate::spa::pod::object::{PodObjectRef, PodPropKeyType, PodPropRef};
use crate::spa::pod::string::PodStringRef;
use crate::spa::pod::struct_::PodStructRef;
use crate::spa::pod::{BasicTypePod, PodBoolRef, PodError, PodIntRef, PodResult};
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ProfilerType<'a> {
    INFO(&'a PodStructRef) = Profiler::INFO.raw,
    CLOCK(&'a PodStructRef) = Profiler::CLOCK.raw,
    DRIVER_BLOCK(&'a PodStructRef) = Profiler::DRIVER_BLOCK.raw,
    FOLLOWER_BLOCK(&'a PodStructRef) = Profiler::FOLLOWER_BLOCK.raw,
}

impl<'a> TryFrom<&'a PodPropRef<'a, ProfilerType<'a>>> for ProfilerType<'a> {
    type Error = PodError;

    fn try_from(value: &'a PodPropRef<'a, ProfilerType<'a>>) -> Result<Self, Self::Error> {
        unsafe {
            match Profiler::from_raw(value.raw.key) {
                Profiler::INFO => Ok(ProfilerType::INFO(value.pod().cast()?)),
                Profiler::CLOCK => Ok(ProfilerType::CLOCK(value.pod().cast()?)),
                Profiler::DRIVER_BLOCK => Ok(ProfilerType::DRIVER_BLOCK(value.pod().cast()?)),
                Profiler::FOLLOWER_BLOCK => Ok(ProfilerType::FOLLOWER_BLOCK(value.pod().cast()?)),
                _ => return Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodPropKeyType<'a> for ProfilerType<'a> {
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<usize>
    where
        W: Write + Seek,
    {
        match self {
            ProfilerType::INFO(pod) => Self::write_pod_prop(buffer, Profiler::INFO.raw, 0, pod),
            ProfilerType::CLOCK(pod) => Self::write_pod_prop(buffer, Profiler::CLOCK.raw, 0, pod),
            ProfilerType::DRIVER_BLOCK(pod) => {
                Self::write_pod_prop(buffer, Profiler::DRIVER_BLOCK.raw, 0, pod)
            }
            ProfilerType::FOLLOWER_BLOCK(pod) => {
                Self::write_pod_prop(buffer, Profiler::FOLLOWER_BLOCK.raw, 0, pod)
            }
        }
    }
}

enum_wrapper!(
    Profiler,
    spa_sys::spa_profiler,
    _START: spa_sys::SPA_PROFILER_START,
    _START_DRIVER: spa_sys::SPA_PROFILER_START_Driver,
    INFO: spa_sys::SPA_PROFILER_info,
    CLOCK: spa_sys::SPA_PROFILER_clock,
    DRIVER_BLOCK: spa_sys::SPA_PROFILER_driverBlock,
    _START_FOLLOWER: spa_sys::SPA_PROFILER_START_Follower,
    FOLLOWER_BLOCK: spa_sys::SPA_PROFILER_followerBlock,
    _START_CUSTOM: spa_sys::SPA_PROFILER_START_CUSTOM,
);
