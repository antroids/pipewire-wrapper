use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::io::{SeekFrom, Write};
use std::mem::size_of;
use std::ops::Rem;
use std::ptr::addr_of;
use std::slice;

use spa_sys::spa_pod;

use crate::spa::pod::choice::enum_::PodEnumRef;
use crate::spa::pod::choice::{ChoiceType, PodChoiceRef};
use crate::spa::pod::{PodError, PodRef, PodResult, PodValue, POD_ALIGN};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

pub trait SizedPod {
    fn pod_size(&self) -> usize;
}

pub trait PrimitiveValue {}

pub trait CloneTo {
    fn clone_to(&self, buffer: &mut impl Write) -> PodResult<usize>;
}

pub trait PodHeader {
    fn pod_header(&self) -> &spa_sys::spa_pod;

    fn pod_type(&self) -> Type {
        Type::from_raw(self.pod_header().type_)
    }
}

pub trait StaticTypePod {
    fn static_type() -> Type;
}

pub trait WritePod: PodValue + Sized {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek;

    fn check_align<W>(buffer: &mut W) -> PodResult<()>
    where
        W: std::io::Write + std::io::Seek,
    {
        if buffer.stream_position()?.rem(POD_ALIGN as u64) == 0 {
            Ok(())
        } else {
            Err(PodError::PodIsNotAligned)
        }
    }

    fn write_header<W>(buffer: &mut W, size: u32, type_: Type) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
    {
        Self::check_align(buffer)?;
        Self::write_value(
            buffer,
            &spa_sys::spa_pod {
                size,
                type_: type_.raw,
            },
        )
    }

    fn write_value<W, V>(buffer: &mut W, value: &V) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
        V: Sized,
    {
        let size = size_of::<V>();
        let ptr = value as *const V as *const u8;
        let slice = unsafe { slice::from_raw_parts(ptr, size) };
        buffer.write_all(slice)?;
        Ok(size)
    }

    fn write_align_padding<W>(buffer: &mut W) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
    {
        let position = buffer.stream_position()? as usize;
        let rem = position.rem(POD_ALIGN);
        if rem > 0 {
            let padding_len = POD_ALIGN - rem;
            let padding = vec![0u8; padding_len];
            buffer.write_all(padding.as_slice())?;
            Ok(padding_len)
        } else {
            Ok(0)
        }
    }

    fn write_end_than_start<W, S, E>(
        buffer: &mut W,
        start_size: usize,
        start: S,
        end: E,
    ) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek,
        S: FnOnce(&mut W, usize) -> PodResult<usize>,
        E: FnOnce(&mut W) -> PodResult<usize>,
    {
        let start_pos = buffer.stream_position()?;
        let end_pos = buffer.seek(SeekFrom::Current(start_size as i64))?;
        end(buffer)?;
        let after_end_pos = buffer.stream_position()?;
        let end_size = (after_end_pos - end_pos) as usize;
        buffer.seek(SeekFrom::Start(start_pos))?;
        let actual_start_size = start(buffer, end_size)?;
        buffer.seek(SeekFrom::Start(after_end_pos))?;
        if start_size != actual_start_size {
            Err(PodError::UnexpectedChoiceElementSize(
                start_size,
                actual_start_size,
            ))
        } else {
            Ok(start_size + end_size)
        }
    }
}

pub trait WriteValue: PodValue {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<usize>
    where
        W: std::io::Write + std::io::Seek;
}

pub trait BasicTypePod
where
    Self: StaticTypePod,
    Self: PodHeader,
    Self: RawWrapper,
    Self: Debug,
{
    fn upcast(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(self.pod_header()) }
    }

    fn cast<T>(&self) -> PodResult<&T>
    where
        T: BasicTypePod,
    {
        let target_type = T::static_type();
        let pod_type = self.upcast().type_();
        if target_type == PodRef::static_type() || target_type == pod_type {
            unsafe { Ok(self.cast_unchecked()) }
        } else if target_type == Type::CHOICE
            && (pod_type == Type::BOOL
                || pod_type == Type::ID
                || pod_type == Type::INT
                || pod_type == Type::LONG
                || pod_type == Type::FLOAT
                || pod_type == Type::DOUBLE
                || pod_type == Type::RECTANGLE
                || pod_type == Type::FRACTION)
        {
            unsafe { Ok(self.cast_unchecked()) }
        } else {
            Err(PodError::WrongPodTypeToCast(target_type, pod_type))
        }
    }

    unsafe fn cast_unchecked<T>(&self) -> &T
    where
        T: BasicTypePod,
    {
        T::from_raw_ptr(addr_of!(*self) as *const _)
    }
}
