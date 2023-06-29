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

impl<'a> PodPropKeyType<'a> for ProfilerType<'a> {}

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