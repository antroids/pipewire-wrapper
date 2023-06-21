use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodRef {
    #[raw]
    raw: spa_sys::spa_pod,
}

impl PodRef {
    pub fn size(&self) -> u32 {
        self.raw.size
    }

    //todo type
}

enum_wrapper!(
    Type,
    u32,
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
