use bitflags::bitflags;
use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;
use std::fmt::{Debug, Formatter};

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
    ALL: spa_sys::SPA_PARAM_Invalid,
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

enum_wrapper!(
    ParamBuffers,
    spa_sys::spa_param_buffers,
    START: spa_sys::SPA_PARAM_BUFFERS_START,
    BUFFERS: spa_sys::SPA_PARAM_BUFFERS_buffers,
    BLOCKS: spa_sys::SPA_PARAM_BUFFERS_blocks,
    SIZE: spa_sys::SPA_PARAM_BUFFERS_size,
    STRIDE: spa_sys::SPA_PARAM_BUFFERS_stride,
    ALIGN: spa_sys::SPA_PARAM_BUFFERS_align,
    DATA_TYPE: spa_sys::SPA_PARAM_BUFFERS_dataType,
);

enum_wrapper!(
    ParamMeta,
    spa_sys::spa_param_meta,
    START: spa_sys::SPA_PARAM_META_START,
    TYPE: spa_sys::SPA_PARAM_META_type,
    SIZE: spa_sys::SPA_PARAM_META_size,
);

enum_wrapper!(
    ParamIO,
    spa_sys::spa_param_io,
    START: spa_sys::SPA_PARAM_IO_START,
    ID: spa_sys::SPA_PARAM_IO_id,
    SIZE: spa_sys::SPA_PARAM_IO_size,
);

enum_wrapper!(
    ParamAvailability,
    spa_sys::spa_param_availability,
    UNKNOWN: spa_sys::SPA_PARAM_AVAILABILITY_unknown,
    NO: spa_sys::SPA_PARAM_AVAILABILITY_no,
    YES: spa_sys::SPA_PARAM_AVAILABILITY_yes,
);

enum_wrapper!(
    ParamProfile,
    spa_sys::spa_param_profile,
    START: spa_sys::SPA_PARAM_PROFILE_START,
    INDEX: spa_sys::SPA_PARAM_PROFILE_index,
    NAME: spa_sys::SPA_PARAM_PROFILE_name,
    DESCRIPTION: spa_sys::SPA_PARAM_PROFILE_description,
    PRIORITY: spa_sys::SPA_PARAM_PROFILE_priority,
    AVAILABLE: spa_sys::SPA_PARAM_PROFILE_available,
    INFO: spa_sys::SPA_PARAM_PROFILE_info,
    CLASSES: spa_sys::SPA_PARAM_PROFILE_classes,
    SAVE: spa_sys::SPA_PARAM_PROFILE_save,
);

// todo ...

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct ParamInfoRef {
    #[raw]
    raw: spa_sys::spa_param_info,
}

impl ParamInfoRef {
    pub fn id(&self) -> u32 {
        self.raw.id
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
