use bitflags::bitflags;

use pipewire_macro_impl::enum_wrapper;
use pipewire_proc_macro::RawWrapper;

use crate::spa::pod::object::format::VideoFormat;
use crate::spa::pod::sequence::PodSequenceRef;
use crate::spa::type_::{PointRef, RectangleRef, RegionRef};
use crate::wrapper::RawWrapper;

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaRef {
    #[raw]
    raw: spa_sys::spa_meta,
}

impl MetaRef {
    pub fn type_(&self) -> MetaType {
        MetaType::from_raw(self.raw.type_)
    }

    pub fn size(&self) -> u32 {
        self.raw.size
    }

    fn data_ptr(&self) -> *mut ::std::os::raw::c_void {
        self.raw.data
    }

    pub fn data(&self) -> MetaData {
        unsafe {
            match self.type_() {
                MetaType::HEADER => {
                    MetaData::HEADER(MetaHeaderRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::VIDEO_CROP => {
                    MetaData::VIDEO_CROP(MetaRegionRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::VIDEO_DAMAGE => {
                    MetaData::VIDEO_DAMAGE(MetaRegionRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::BITMAP => {
                    MetaData::BITMAP(MetaBitmapRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::CURSOR => {
                    MetaData::CURSOR(MetaCursorRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::CONTROL => {
                    MetaData::CONTROL(MetaControlRef::from_raw_ptr(self.data_ptr().cast()))
                }
                MetaType::BUSY => MetaData::BUSY(MetaBusyRef::from_raw_ptr(self.data_ptr().cast())),
                _ => MetaData::INVALID,
            }
        }
    }
}

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum MetaData<'a> {
    INVALID = MetaType::INVALID.raw,
    HEADER(&'a MetaHeaderRef) = MetaType::HEADER.raw,
    VIDEO_CROP(&'a MetaRegionRef) = MetaType::VIDEO_CROP.raw,
    VIDEO_DAMAGE(&'a MetaRegionRef) = MetaType::VIDEO_DAMAGE.raw,
    BITMAP(&'a MetaBitmapRef) = MetaType::BITMAP.raw,
    CURSOR(&'a MetaCursorRef) = MetaType::CURSOR.raw,
    CONTROL(&'a MetaControlRef) = MetaType::CONTROL.raw,
    BUSY(&'a MetaBusyRef) = MetaType::BUSY.raw,
}

enum_wrapper!(
    MetaType,
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

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaHeaderRef {
    #[raw]
    raw: spa_sys::spa_meta_header,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy)]
    #[repr(transparent)]
    pub struct HeaderFlags: u32 {
        const DISCONT = spa_sys::SPA_META_HEADER_FLAG_DISCONT;
        const CORRUPTED = spa_sys::SPA_META_HEADER_FLAG_CORRUPTED;
        const MARKER = spa_sys::SPA_META_HEADER_FLAG_MARKER;
        const HEADER = spa_sys::SPA_META_HEADER_FLAG_HEADER;
        const GAP = spa_sys::SPA_META_HEADER_FLAG_GAP;
        const DELTA_UNIT = spa_sys::SPA_META_HEADER_FLAG_DELTA_UNIT;
    }
}

impl MetaHeaderRef {
    pub fn flags(&self) -> HeaderFlags {
        HeaderFlags::from_bits_retain(self.raw.flags)
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn pts(&self) -> i64 {
        self.raw.pts
    }

    pub fn dts_offset(&self) -> i64 {
        self.raw.dts_offset
    }

    pub fn seq(&self) -> u64 {
        self.raw.seq
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaRegionRef {
    #[raw]
    raw: spa_sys::spa_meta_region,
}

impl MetaRegionRef {
    pub fn region(&self) -> RegionRef {
        RegionRef::from_raw(self.raw.region)
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaBitmapRef {
    #[raw]
    raw: spa_sys::spa_meta_bitmap,
}

impl MetaBitmapRef {
    pub fn format(&self) -> VideoFormat {
        VideoFormat::from_raw(self.raw.format)
    }

    pub fn size(&self) -> RectangleRef {
        RectangleRef::from_raw(self.raw.size)
    }

    pub fn strict(&self) -> i32 {
        self.raw.stride
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaCursorRef {
    #[raw]
    raw: spa_sys::spa_meta_cursor,
}

impl MetaCursorRef {
    pub fn id(&self) -> u32 {
        self.raw.id
    }

    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn position(&self) -> PointRef {
        PointRef::from_raw(self.raw.position)
    }

    pub fn hotspot(&self) -> PointRef {
        PointRef::from_raw(self.raw.hotspot)
    }

    pub fn bitmap_offset(&self) -> u32 {
        self.raw.bitmap_offset
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaControlRef {
    #[raw]
    raw: spa_sys::spa_meta_control,
}

impl MetaControlRef {
    pub fn sequence(&self) -> &PodSequenceRef {
        unsafe { PodSequenceRef::from_raw_ptr(&self.raw.sequence) }
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaBusyRef {
    #[raw]
    raw: spa_sys::spa_meta_busy,
}

impl MetaBusyRef {
    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn count(&self) -> u32 {
        self.raw.count
    }
}
