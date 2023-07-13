use std::fmt::{Debug, Formatter};

use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::{RawWrapper, Wrapper};

use crate::wrapper::RawWrapper;

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct Flags: u32 {
        const SERIAL = spa_sys::SPA_PARAM_INFO_SERIAL;
        const READ = spa_sys::SPA_PARAM_INFO_READ;
        const WRITE = spa_sys::SPA_PARAM_INFO_WRITE;
        const READWRITE = spa_sys::SPA_PARAM_INFO_READWRITE;
    }
}

enum_wrapper!(
    ParamType,
    spa_sys::spa_param_type,
    INVALID: spa_sys::SPA_PARAM_Invalid,
    PROP_INFO: spa_sys::SPA_PARAM_PropInfo,
    PROPS: spa_sys::SPA_PARAM_Props,
    ENUM_FORMAT: spa_sys::SPA_PARAM_EnumFormat,
    FORMAT: spa_sys::SPA_PARAM_Format,
    BUFFERS: spa_sys::SPA_PARAM_Buffers,
    META: spa_sys::SPA_PARAM_Meta,
    IO: spa_sys::SPA_PARAM_IO,
    ENUM_PROFILE: spa_sys::SPA_PARAM_EnumProfile,
    PROFILE: spa_sys::SPA_PARAM_Profile,
    ENUM_PORT_CONFIG: spa_sys::SPA_PARAM_EnumPortConfig,
    PORT_CONFIG: spa_sys::SPA_PARAM_PortConfig,
    ENUM_ROUTE: spa_sys::SPA_PARAM_EnumRoute,
    ROUTE: spa_sys::SPA_PARAM_Route,
    CONTROL: spa_sys::SPA_PARAM_Control,
    LATENCY: spa_sys::SPA_PARAM_Latency,
    PROCESS_LATENCY: spa_sys::SPA_PARAM_ProcessLatency,
);

// todo ...

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct ParamInfoRef {
    #[raw]
    raw: spa_sys::spa_param_info,
}

impl ParamInfoRef {
    pub fn id(&self) -> ParamType {
        ParamType::from_raw(self.raw.id)
    }

    pub fn flags(&self) -> Flags {
        Flags::from_bits_retain(self.raw.flags)
    }
}

impl Debug for ParamInfoRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParamInfoRef")
            .field("id", &self.id())
            .field("flags", &self.flags())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    id: ParamType,
    flags: Flags,
}

impl ParamInfo {
    pub fn from_ref(ref_: &ParamInfoRef) -> Self {
        Self {
            id: ref_.id(),
            flags: ref_.flags(),
        }
    }

    pub fn id(&self) -> ParamType {
        self.id
    }
    pub fn flags(&self) -> Flags {
        self.flags
    }
}
