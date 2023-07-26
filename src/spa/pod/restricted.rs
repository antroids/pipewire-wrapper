/*
 * SPDX-License-Identifier: MIT
 */
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
use crate::spa::pod::{
    BasicType, BasicTypeValue, PodError, PodRef, PodResult, PodValue, POD_ALIGN,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

pub trait SizedPod {
    fn pod_size(&self) -> usize;
}

pub trait PrimitiveValue {}

pub trait CloneTo {
    fn clone_to(&self, buffer: &mut impl Write) -> PodResult<()>;
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

pub trait WritePod: PodRawValue + Sized {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: std::io::Write + std::io::Seek;
}

pub trait WriteValue: PodRawValue {
    fn write_raw_value<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
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
    fn cast<T>(&self) -> PodResult<&T>
    where
        T: BasicTypePod,
    {
        let target_type = T::static_type();
        let pod_type = self.pod_type();
        #[allow(clippy::if_same_then_else)]
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

pub trait PodRawValue: PodValue {
    type RawValue;

    fn raw_value_ptr(&self) -> *const Self::RawValue;

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value>;
}

pub fn check_align<W>(buffer: &mut W) -> PodResult<()>
where
    W: std::io::Write + std::io::Seek,
{
    if buffer.stream_position()?.rem(POD_ALIGN as u64) == 0 {
        Ok(())
    } else {
        Err(PodError::PodIsNotAligned)
    }
}

pub fn write_header<W>(buffer: &mut W, size: u32, type_: Type) -> PodResult<()>
where
    W: std::io::Write + std::io::Seek,
{
    check_align(buffer)?;
    write_value(
        buffer,
        &spa_sys::spa_pod {
            size,
            type_: type_.raw,
        },
    )
}

pub fn write_value<W, V>(buffer: &mut W, value: &V) -> PodResult<()>
where
    W: std::io::Write + std::io::Seek,
    V: Sized,
{
    let size = size_of::<V>();
    let ptr = value as *const V as *const u8;
    let slice = unsafe { slice::from_raw_parts(ptr, size) };
    buffer.write_all(slice)?;
    Ok(())
}

pub fn write_align_padding<W>(buffer: &mut W) -> PodResult<()>
where
    W: std::io::Write + std::io::Seek,
{
    let position = buffer.stream_position()? as usize;
    let rem = position.rem(POD_ALIGN);
    if rem > 0 {
        let padding_len = POD_ALIGN - rem;
        let padding = vec![0u8; padding_len];
        buffer.write_all(padding.as_slice())?;
    }
    Ok(())
}

pub fn write_count_size<W, F>(buffer: &mut W, func: F) -> PodResult<usize>
where
    W: std::io::Write + std::io::Seek,
    F: FnOnce(&mut W) -> PodResult<()>,
{
    let start_pos = buffer.stream_position()?;
    func(buffer)?;
    let end_pos = buffer.stream_position()?;
    Ok((end_pos - start_pos) as usize)
}
