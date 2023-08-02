/*
 * SPDX-License-Identifier: MIT
 */
use std::ffi::CStr;
use std::ptr::addr_of;
use std::slice;

use bitflags::{bitflags, Flags};

use pipewire_wrapper_proc_macro::RawWrapper;

use crate::enum_wrapper;
use crate::spa::type_::{FractionRef, RectangleRef};
use crate::wrapper::RawWrapper;

use super::pod::object::param_io::IOType;
use super::type_::IOBuffersRef;

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

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct ClockFlags: u32 {
        const FREEWHEEL = spa_sys::SPA_IO_CLOCK_FLAG_FREEWHEEL;
    }
}

impl IOClockRef {
    pub fn flags(&self) -> ClockFlags {
        ClockFlags::from_bits_retain(self.raw.flags)
    }

    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr(self.raw.name.as_ptr_range().start) }
    }

    pub fn nsec(&self) -> u64 {
        self.raw.nsec
    }

    pub fn rate(&self) -> &FractionRef {
        unsafe { FractionRef::from_raw_ptr(addr_of!(self.raw.rate)) }
    }

    pub fn position(&self) -> u64 {
        self.raw.position
    }

    pub fn duration(&self) -> u64 {
        self.raw.duration
    }

    pub fn delay(&self) -> i64 {
        self.raw.delay
    }

    pub fn rate_diff(&self) -> f64 {
        self.raw.rate_diff
    }

    pub fn next_nsec(&self) -> u64 {
        self.raw.next_nsec
    }
}

#[derive(RawWrapper, Clone)]
#[repr(transparent)]
pub struct IOVideoSizeRef {
    #[raw]
    raw: spa_sys::spa_io_video_size,
}

impl IOVideoSizeRef {
    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn stride(&self) -> u32 {
        self.raw.stride
    }

    pub fn size(&self) -> &RectangleRef {
        unsafe { RectangleRef::from_raw_ptr(addr_of!(self.raw.size)) }
    }

    pub fn framerate(&self) -> &FractionRef {
        unsafe { FractionRef::from_raw_ptr(addr_of!(self.raw.framerate)) }
    }
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

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct SegmentBarFlags: u32 {
        const VALID = spa_sys::SPA_IO_SEGMENT_BAR_FLAG_VALID;
    }
}

impl IOSegmentBarRef {
    pub fn flags(&self) -> SegmentBarFlags {
        SegmentBarFlags::from_bits_retain(self.raw.flags)
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn signature_num(&self) -> f32 {
        self.raw.signature_num
    }

    pub fn signature_denom(&self) -> f32 {
        self.raw.signature_denom
    }

    pub fn bpm(&self) -> f64 {
        self.raw.bpm
    }

    pub fn beat(&self) -> f64 {
        self.raw.beat
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSegmentVideoRef {
    #[raw]
    raw: spa_sys::spa_io_segment_video,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct SegmentVideoFlags: u32 {
        const VALID = spa_sys::SPA_IO_SEGMENT_VIDEO_FLAG_VALID;
        const INTERLACED = spa_sys::SPA_IO_SEGMENT_VIDEO_FLAG_INTERLACED;
        const DROP_FRAME = spa_sys::SPA_IO_SEGMENT_VIDEO_FLAG_DROP_FRAME;
        const PULL_DOWN = spa_sys::SPA_IO_SEGMENT_VIDEO_FLAG_PULL_DOWN;
    }
}

impl IOSegmentVideoRef {
    pub fn flags(&self) -> SegmentVideoFlags {
        SegmentVideoFlags::from_bits_retain(self.raw.flags)
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn framerate(&self) -> &FractionRef {
        unsafe { FractionRef::from_raw_ptr(addr_of!(self.raw.framerate)) }
    }

    pub fn hours(&self) -> u32 {
        self.raw.hours
    }

    pub fn minutes(&self) -> u32 {
        self.raw.minutes
    }

    pub fn seconds(&self) -> u32 {
        self.raw.seconds
    }

    pub fn frames(&self) -> u32 {
        self.raw.frames
    }

    pub fn field_count(&self) -> u32 {
        self.raw.field_count
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOSegmentRef {
    #[raw]
    raw: spa_sys::spa_io_segment,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct SegmentFlags: u32 {
        const NO_POSITION = spa_sys::SPA_IO_SEGMENT_FLAG_NO_POSITION;
        const LOOPING = spa_sys::SPA_IO_SEGMENT_FLAG_LOOPING;
    }
}

impl IOSegmentRef {
    pub fn version(&self) -> u32 {
        self.raw.version
    }

    pub fn flags(&self) -> SegmentFlags {
        SegmentFlags::from_bits_retain(self.raw.flags)
    }

    pub fn start(&self) -> u64 {
        self.raw.start
    }

    pub fn duration(&self) -> u64 {
        self.raw.duration
    }

    pub fn rate(&self) -> f64 {
        self.raw.rate
    }

    pub fn position(&self) -> u64 {
        self.raw.position
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IOPositionRef {
    #[raw]
    raw: spa_sys::spa_io_position,
}

enum_wrapper!(
    PositionState,
    spa_sys::spa_io_position_state,
    STOPPED: spa_sys::SPA_IO_POSITION_STATE_STOPPED,
    STARTING: spa_sys::SPA_IO_POSITION_STATE_STARTING,
    RUNNING: spa_sys::SPA_IO_POSITION_STATE_RUNNING,
);

impl IOPositionRef {
    pub fn clock(&self) -> &IOClockRef {
        unsafe { IOClockRef::from_raw_ptr(addr_of!(self.raw.clock)) }
    }

    pub fn video(&self) -> &IOVideoSizeRef {
        unsafe { IOVideoSizeRef::from_raw_ptr(addr_of!(self.raw.video)) }
    }

    pub fn offset(&self) -> i64 {
        self.raw.offset
    }

    pub fn state(&self) -> PositionState {
        PositionState::from_raw(self.raw.state)
    }

    pub fn segments(&self) -> &[IOSegmentRef] {
        unsafe {
            slice::from_raw_parts(
                addr_of!(self.raw.segments) as *const IOSegmentRef,
                self.raw.n_segments.min(8) as usize,
            )
        }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct IORateMatchRef {
    #[raw]
    raw: spa_sys::spa_io_rate_match,
}

#[repr(u32)]
pub enum IOValue<'a> {
    INVALID,
    BUFFERS(&'a mut IOBuffersRef),
    RANGE(&'a mut IORangeRef),
    CLOCK(&'a mut IOClockRef),
    LATENCY(&'a mut IOLatencyRef),
    CONTROL(&'a mut IOSequenceRef),
    NOTIFY(&'a mut IOSequenceRef),
    POSITION(&'a mut IOPositionRef),
    RATEMATCH(&'a mut IORateMatchRef),
    MEMORY(&'a mut IOMemoryRef),
}

impl<'a> IOValue<'a> {
    pub(crate) unsafe fn from_type_and_ptr(
        io_type: IOType,
        ptr: *mut ::std::os::raw::c_void,
    ) -> Self {
        match io_type {
            IOType::BUFFERS => IOValue::BUFFERS(IOBuffersRef::mut_from_raw_ptr(ptr.cast())),
            IOType::RANGE => IOValue::RANGE(IORangeRef::mut_from_raw_ptr(ptr.cast())),
            IOType::CLOCK => IOValue::CLOCK(IOClockRef::mut_from_raw_ptr(ptr.cast())),
            IOType::LATENCY => IOValue::LATENCY(IOLatencyRef::mut_from_raw_ptr(ptr.cast())),
            IOType::CONTROL => IOValue::CONTROL(IOSequenceRef::mut_from_raw_ptr(ptr.cast())),
            IOType::NOTIFY => IOValue::NOTIFY(IOSequenceRef::mut_from_raw_ptr(ptr.cast())),
            IOType::POSITION => IOValue::POSITION(IOPositionRef::mut_from_raw_ptr(ptr.cast())),
            IOType::RATEMATCH => IOValue::RATEMATCH(IORateMatchRef::mut_from_raw_ptr(ptr.cast())),
            IOType::MEMORY => IOValue::MEMORY(IOMemoryRef::mut_from_raw_ptr(ptr.cast())),
            _ => IOValue::INVALID,
        }
    }
}
