use std::any::TypeId;
use std::fmt::{Debug, Formatter};
use std::mem::size_of;
use std::ptr::addr_of;
use std::time::Duration;

use spa_sys::spa_pod;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::*;
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct TimespecRef {
    #[raw]
    raw: spa_sys::timespec,
}

impl TryFrom<Duration> for TimespecRef {
    type Error = crate::error::Error;

    fn try_from(value: Duration) -> Result<Self, Self::Error> {
        Ok(Self {
            raw: spa_sys::timespec {
                tv_sec: value
                    .as_secs()
                    .try_into()
                    .map_err(|_| crate::error::Error::WrongTimeFormat)?,
                tv_nsec: value
                    .subsec_nanos()
                    .try_into()
                    .map_err(|_| crate::error::Error::WrongTimeFormat)?,
            },
        })
    }
}

#[derive(RawWrapper, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct RectangleRef {
    #[raw]
    raw: spa_sys::spa_rectangle,
}

impl From<(u32, u32)> for RectangleRef {
    fn from(value: (u32, u32)) -> Self {
        RectangleRef::from_raw(spa_sys::spa_rectangle {
            width: value.0,
            height: value.1,
        })
    }
}

impl RectangleRef {
    pub fn width(&self) -> u32 {
        self.raw.width
    }

    pub fn height(&self) -> u32 {
        self.raw.height
    }
}

#[derive(RawWrapper, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct PointRef {
    #[raw]
    raw: spa_sys::spa_point,
}

impl From<(i32, i32)> for PointRef {
    fn from(value: (i32, i32)) -> Self {
        PointRef::from_raw(spa_sys::spa_point {
            x: value.0,
            y: value.1,
        })
    }
}

impl PointRef {
    pub fn x(&self) -> i32 {
        self.raw.x
    }

    pub fn y(&self) -> i32 {
        self.raw.y
    }
}

#[derive(RawWrapper, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct RegionRef {
    #[raw]
    raw: spa_sys::spa_region,
}

impl From<(PointRef, RectangleRef)> for RegionRef {
    fn from(value: (PointRef, RectangleRef)) -> Self {
        RegionRef::from_raw(spa_sys::spa_region {
            position: value.0.raw,
            size: value.1.raw,
        })
    }
}

impl RegionRef {
    pub fn position(&self) -> PointRef {
        PointRef::from_raw(self.raw.position)
    }

    pub fn size(&self) -> RectangleRef {
        RectangleRef::from_raw(self.raw.size)
    }
}

#[derive(RawWrapper, Debug, Copy, Clone)]
#[repr(transparent)]
pub struct FractionRef {
    #[raw]
    raw: spa_sys::spa_fraction,
}

impl From<(u32, u32)> for FractionRef {
    fn from(value: (u32, u32)) -> Self {
        FractionRef::from_raw(spa_sys::spa_fraction {
            num: value.0,
            denom: value.1,
        })
    }
}

impl FractionRef {
    pub fn num(&self) -> u32 {
        self.raw.num
    }

    pub fn denom(&self) -> u32 {
        self.raw.denom
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaRef {
    #[raw]
    raw: spa_sys::spa_meta,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaHeaderRef {
    #[raw]
    raw: spa_sys::spa_meta_header,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaRegionRef {
    #[raw]
    raw: spa_sys::spa_meta_region,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaBitmapRef {
    #[raw]
    raw: spa_sys::spa_meta_bitmap,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaCursorRef {
    #[raw]
    raw: spa_sys::spa_meta_cursor,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaControlRef {
    #[raw]
    raw: spa_sys::spa_meta_control,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct MetaBusyRef {
    #[raw]
    raw: spa_sys::spa_meta_busy,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct ChunkRef {
    #[raw]
    raw: spa_sys::spa_chunk,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct DataRef {
    #[raw]
    raw: spa_sys::spa_data,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct BufferRef {
    #[raw]
    raw: spa_sys::spa_buffer,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct BufferAllocInfoRef {
    #[raw]
    raw: spa_sys::spa_buffer_alloc_info,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct EventBodyRef {
    #[raw]
    raw: spa_sys::spa_event_body,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct EventRef {
    #[raw]
    raw: spa_sys::spa_event,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct CommandBodyRef {
    #[raw]
    raw: spa_sys::spa_command_body,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct CommandRef {
    #[raw]
    raw: spa_sys::spa_command,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOBuffersRef {
    #[raw]
    raw: spa_sys::spa_io_buffers,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOMemoryRef {
    #[raw]
    raw: spa_sys::spa_io_memory,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IORangeRef {
    #[raw]
    raw: spa_sys::spa_io_range,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOClockRef {
    #[raw]
    raw: spa_sys::spa_io_clock,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOVideoSizeRef {
    #[raw]
    raw: spa_sys::spa_io_video_size,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOLatencyRef {
    #[raw]
    raw: spa_sys::spa_io_latency,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSequenceRef {
    #[raw]
    raw: spa_sys::spa_io_sequence,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSegmentBarRef {
    #[raw]
    raw: spa_sys::spa_io_segment_bar,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSegmentVideoRef {
    #[raw]
    raw: spa_sys::spa_io_segment_video,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSegmentRef {
    #[raw]
    raw: spa_sys::spa_io_segment,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOPositionRef {
    #[raw]
    raw: spa_sys::spa_io_position,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IORateMatchRef {
    #[raw]
    raw: spa_sys::spa_io_rate_match,
}

// #[derive(RawWrapper)]
// #[repr(transparent)]
// pub struct Ref {
//     #[raw]
//     raw: spa_sys::spa_,
// }

// todo ...

enum_wrapper!(
    Meta,
    spa_sys::spa_meta_type,
    INVALID: spa_sys::SPA_META_Invalid,
    HEADER: spa_sys::SPA_META_Header,
    VIDEO_CROP: spa_sys::SPA_META_VideoCrop,
    VIDEO_DAMAGE: spa_sys::SPA_META_VideoDamage,
    BITMAP: spa_sys::SPA_META_Bitmap,
    CURSOR: spa_sys::SPA_META_Cursor,
    CONTROL: spa_sys::SPA_META_Control,
    BUSY: spa_sys::SPA_META_Busy,
    _LAST: spa_sys::_SPA_META_LAST,
);

enum_wrapper!(
    Data,
    spa_sys::spa_data_type,
    INVALID: spa_sys::SPA_DATA_Invalid,
    MEM_PTR: spa_sys::SPA_DATA_MemPtr,
    MEM_FD: spa_sys::SPA_DATA_MemFd,
    DMA_BUF: spa_sys::SPA_DATA_DmaBuf,
    MEM_ID: spa_sys::SPA_DATA_MemId,
    _LAST: spa_sys::_SPA_DATA_LAST,
);

enum_wrapper!(
    Control,
    spa_sys::spa_control_type,
    INVALID: spa_sys::SPA_CONTROL_Invalid,
    PROPERTIES: spa_sys::SPA_CONTROL_Properties,
    MIDI: spa_sys::SPA_CONTROL_Midi,
    OSC: spa_sys::SPA_CONTROL_OSC,
    _LAST: spa_sys::_SPA_CONTROL_LAST,
);

enum_wrapper!(
    EventDevice,
    spa_sys::spa_device_event,
    _START: spa_sys::SPA_EVENT_DEVICE_START,
    OBJECT: spa_sys::SPA_EVENT_DEVICE_Object,
    PROPS: spa_sys::SPA_EVENT_DEVICE_Props,
);

enum_wrapper!(
    NodeCommand,
    spa_sys::spa_node_command,
    SUSPEND: spa_sys::SPA_NODE_COMMAND_Suspend,
    PAUSE: spa_sys::SPA_NODE_COMMAND_Pause,
    START: spa_sys::SPA_NODE_COMMAND_Start,
    ENABLE: spa_sys::SPA_NODE_COMMAND_Enable,
    DISABLE: spa_sys::SPA_NODE_COMMAND_Disable,
    FLUSH: spa_sys::SPA_NODE_COMMAND_Flush,
    DRAIN: spa_sys::SPA_NODE_COMMAND_Drain,
    MARKER: spa_sys::SPA_NODE_COMMAND_Marker,
    PARAM_BEGIN: spa_sys::SPA_NODE_COMMAND_ParamBegin,
    PARAM_END: spa_sys::SPA_NODE_COMMAND_ParamEnd,
    REQUEST_PROCESS: spa_sys::SPA_NODE_COMMAND_RequestProcess,
);

enum_wrapper!(
    NodeEvent,
    spa_sys::spa_node_event,
    ERROR: spa_sys::SPA_NODE_EVENT_Error,
    BUFFERING: spa_sys::SPA_NODE_EVENT_Buffering,
    REQUEST_REFRESH: spa_sys::SPA_NODE_EVENT_RequestRefresh,
    REQUEST_PROCESS: spa_sys::SPA_NODE_EVENT_RequestProcess,
);

enum_wrapper!(
    EventNode,
    spa_sys::_bindgen_ty_10,
    START: spa_sys::SPA_EVENT_NODE_START,
);

enum_wrapper!(
    IO,
    spa_sys::spa_io_type,
    INVALID: spa_sys::SPA_IO_Invalid,
    BUFFERS: spa_sys::SPA_IO_Buffers,
    RANGE: spa_sys::SPA_IO_Range,
    CLOCK: spa_sys::SPA_IO_Clock,
    LATENCY: spa_sys::SPA_IO_Latency,
    CONTROL: spa_sys::SPA_IO_Control,
    NOTIFY: spa_sys::SPA_IO_Notify,
    POSITION: spa_sys::SPA_IO_Position,
    RATE_MATCH: spa_sys::SPA_IO_RateMatch,
    MEMORY: spa_sys::SPA_IO_Memory,
);

enum_wrapper!(
    Type,
    spa_sys::_bindgen_ty_10,
    // Basic types
    _START: spa_sys::SPA_TYPE_START,
    NONE: spa_sys::SPA_TYPE_None,
    BOOL: spa_sys::SPA_TYPE_Bool,
    ID: spa_sys::SPA_TYPE_Id,
    INT: spa_sys::SPA_TYPE_Int,
    LONG: spa_sys::SPA_TYPE_Long,
    FLOAT: spa_sys::SPA_TYPE_Float,
    DOUBLE: spa_sys::SPA_TYPE_Double,
    STRING: spa_sys::SPA_TYPE_String,
    BYTES: spa_sys::SPA_TYPE_Bytes,
    RECTANGLE: spa_sys::SPA_TYPE_Rectangle,
    FRACTION: spa_sys::SPA_TYPE_Fraction,
    BITMAP: spa_sys::SPA_TYPE_Bitmap,
    ARRAY: spa_sys::SPA_TYPE_Array,
    STRUCT: spa_sys::SPA_TYPE_Struct,
    OBJECT: spa_sys::SPA_TYPE_Object,
    SEQUENCE: spa_sys::SPA_TYPE_Sequence,
    POINTER: spa_sys::SPA_TYPE_Pointer,
    FD: spa_sys::SPA_TYPE_Fd,
    CHOICE: spa_sys::SPA_TYPE_Choice,
    POD: spa_sys::SPA_TYPE_Pod,
    _LAST: spa_sys::_SPA_TYPE_LAST,
    // Pointers
    _POINTER_START: spa_sys::SPA_TYPE_POINTER_START,
    POINTER_BUFFER: spa_sys::SPA_TYPE_POINTER_Buffer,
    POINTER_META: spa_sys::SPA_TYPE_POINTER_Meta,
    POINTER_DICT: spa_sys::SPA_TYPE_POINTER_Dict,
    _POINTER_LAST: spa_sys::_SPA_TYPE_POINTER_LAST,
    // Events
    _EVENT_START: spa_sys::SPA_TYPE_EVENT_START,
    EVENT_DEVICE: spa_sys::SPA_TYPE_EVENT_Device,
    EVENT_NODE: spa_sys::SPA_TYPE_EVENT_Node,
    _EVENT_LAST: spa_sys::_SPA_TYPE_EVENT_LAST,
    // Commands
    _COMMAND_START: spa_sys::SPA_TYPE_COMMAND_START,
    COMMAND_DEVICE: spa_sys::SPA_TYPE_COMMAND_Device,
    COMMAND_NODE: spa_sys::SPA_TYPE_COMMAND_Node,
    _COMMAND_LAST: spa_sys::_SPA_TYPE_COMMAND_LAST,
    // Objects
    _OBJECT_START: spa_sys::SPA_TYPE_OBJECT_START,
    OBJECT_PROP_INFO: spa_sys::SPA_TYPE_OBJECT_PropInfo,
    OBJECT_PROPS: spa_sys::SPA_TYPE_OBJECT_Props,
    OBJECT_FORMAT: spa_sys::SPA_TYPE_OBJECT_Format,
    OBJECT_PARAM_BUFFERS: spa_sys::SPA_TYPE_OBJECT_ParamBuffers,
    OBJECT_PARAM_META: spa_sys::SPA_TYPE_OBJECT_ParamMeta,
    OBJECT_PARAM_IO: spa_sys::SPA_TYPE_OBJECT_ParamIO,
    OBJECT_PARAM_PROFILE: spa_sys::SPA_TYPE_OBJECT_ParamProfile,
    OBJECT_PARAM_PORT_CONFIG: spa_sys::SPA_TYPE_OBJECT_ParamPortConfig,
    OBJECT_PARAM_ROUTE: spa_sys::SPA_TYPE_OBJECT_ParamRoute,
    OBJECT_PROFILER: spa_sys::SPA_TYPE_OBJECT_Profiler,
    OBJECT_PARAM_LATENCY: spa_sys::SPA_TYPE_OBJECT_ParamLatency,
    OBJECT_PARAM_PROCESS_LATENCY: spa_sys::SPA_TYPE_OBJECT_ParamProcessLatency,
    _OBJECT_LAST: spa_sys::_SPA_TYPE_OBJECT_LAST,
    // Vendor
    VENDOR_PIPEWIRE: spa_sys::SPA_TYPE_VENDOR_PipeWire,
    VENDOR_OTHER: spa_sys::SPA_TYPE_VENDOR_Other,
);
