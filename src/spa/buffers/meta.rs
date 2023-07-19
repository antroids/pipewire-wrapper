use std::mem::size_of;
use std::ptr::{addr_of, addr_of_mut};
use std::slice;

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

    pub fn set_type(&mut self, type_: MetaType) {
        self.raw.type_ = type_.into();
    }

    pub fn size(&self) -> u32 {
        self.raw.size
    }

    pub fn set_size(&mut self, size: u32) {
        self.raw.size = size;
    }

    fn data_ptr(&self) -> *mut ::std::os::raw::c_void {
        self.raw.data
    }

    unsafe fn data_slice<T>(&self) -> &mut [T]
    where
        T: RawWrapper,
    {
        let element_size = size_of::<T>();
        let len = self.size() as usize / element_size;
        slice::from_raw_parts_mut(self.data_ptr() as *mut T, len)
    }

    pub fn data(&self) -> MetaData {
        unsafe {
            match self.type_() {
                MetaType::HEADER => MetaData::HEADER(self.data_slice()),
                MetaType::VIDEO_CROP => MetaData::VIDEO_CROP(self.data_slice()),
                MetaType::VIDEO_DAMAGE => MetaData::VIDEO_DAMAGE(self.data_slice()),
                MetaType::BITMAP => MetaData::BITMAP(self.data_slice()),
                MetaType::CURSOR => MetaData::CURSOR(self.data_slice()),
                MetaType::CONTROL => MetaData::CONTROL(self.data_slice()),
                MetaType::BUSY => MetaData::BUSY(self.data_slice()),
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
    HEADER(&'a mut [MetaHeaderRef]) = MetaType::HEADER.raw,
    VIDEO_CROP(&'a mut [MetaRegionRef]) = MetaType::VIDEO_CROP.raw,
    VIDEO_DAMAGE(&'a mut [MetaRegionRef]) = MetaType::VIDEO_DAMAGE.raw,
    BITMAP(&'a mut [MetaBitmapRef]) = MetaType::BITMAP.raw,
    CURSOR(&'a mut [MetaCursorRef]) = MetaType::CURSOR.raw,
    CONTROL(&'a mut [MetaControlRef]) = MetaType::CONTROL.raw,
    BUSY(&'a mut [MetaBusyRef]) = MetaType::BUSY.raw,
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

    pub fn set_flags(&mut self, flags: HeaderFlags) {
        self.raw.flags = flags.bits()
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.raw.offset = offset
    }

    pub fn pts(&self) -> i64 {
        self.raw.pts
    }

    pub fn set_pts(&mut self, pts: i64) {
        self.raw.pts = pts
    }

    pub fn dts_offset(&self) -> i64 {
        self.raw.dts_offset
    }

    pub fn set_dts_offset(&mut self, dts_offset: i64) {
        self.raw.dts_offset = dts_offset
    }

    pub fn seq(&self) -> u64 {
        self.raw.seq
    }

    pub fn set_seq(&mut self, seq: u64) {
        self.raw.seq = seq
    }
}

#[derive(RawWrapper, Debug)]
#[repr(transparent)]
pub struct MetaRegionRef {
    #[raw]
    raw: spa_sys::spa_meta_region,
}

impl MetaRegionRef {
    pub fn region(&self) -> &RegionRef {
        unsafe { RegionRef::from_raw_ptr(addr_of!(self.raw.region)) }
    }

    pub fn region_mut(&mut self) -> &mut RegionRef {
        unsafe { RegionRef::mut_from_raw_ptr(addr_of_mut!(self.raw.region)) }
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

    pub fn set_format(&mut self, format: VideoFormat) {
        self.raw.format = format.into()
    }

    pub fn size(&self) -> &RectangleRef {
        unsafe { RectangleRef::from_raw_ptr(addr_of!(self.raw.size)) }
    }

    pub fn size_mut(&mut self) -> &mut RectangleRef {
        unsafe { RectangleRef::mut_from_raw_ptr(addr_of_mut!(self.raw.size)) }
    }

    pub fn stride(&self) -> i32 {
        self.raw.stride
    }

    pub fn set_stride(&mut self, stride: i32) {
        self.raw.stride = stride
    }

    pub fn offset(&self) -> u32 {
        self.raw.offset
    }

    pub fn set_offset(&mut self, offset: u32) {
        self.raw.offset = offset
    }

    pub unsafe fn bitmap<T: Sized>(&self) -> Option<&mut [T]> {
        if self.raw.offset >= size_of::<MetaBitmapRef>() as u32 {
            let bitmap_ptr = (self.as_raw_ptr() as *mut u8).offset(self.raw.offset as isize);
            let len = self.raw.stride as u32 * self.raw.size.height / size_of::<T>() as u32;
            Some(slice::from_raw_parts_mut(bitmap_ptr.cast(), len as usize))
        } else {
            None
        }
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

    pub fn set_id(&mut self, id: u32) {
        self.raw.id = id
    }

    pub fn flags(&self) -> u32 {
        self.raw.flags
    }

    pub fn set_flags(&mut self, flags: u32) {
        self.raw.flags = flags
    }

    pub fn position(&self) -> &PointRef {
        unsafe { PointRef::from_raw_ptr(addr_of!(self.raw.position)) }
    }

    pub fn position_mut(&mut self) -> &mut PointRef {
        unsafe { PointRef::mut_from_raw_ptr(addr_of_mut!(self.raw.position)) }
    }

    pub fn hotspot(&self) -> &PointRef {
        unsafe { PointRef::from_raw_ptr(addr_of!(self.raw.hotspot)) }
    }

    pub fn hotspot_mut(&mut self) -> &mut PointRef {
        unsafe { PointRef::mut_from_raw_ptr(addr_of_mut!(self.raw.hotspot)) }
    }

    pub fn bitmap_offset(&self) -> u32 {
        self.raw.bitmap_offset
    }

    pub fn set_bitmap_offset(&mut self, bitmap_offset: u32) {
        self.raw.bitmap_offset = bitmap_offset
    }

    pub unsafe fn bitmap(&self) -> Option<&mut MetaBitmapRef> {
        unsafe {
            if self.raw.bitmap_offset >= size_of::<MetaCursorRef>() as u32 {
                let bitmap_ptr =
                    (self.as_raw_ptr() as *mut u8).offset(self.bitmap_offset() as isize);
                Some(MetaBitmapRef::mut_from_raw_ptr(bitmap_ptr.cast()))
            } else {
                None
            }
        }
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
