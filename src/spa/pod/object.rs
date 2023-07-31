/*
 * SPDX-License-Identifier: MIT
 */
use std::fmt::{Debug, Formatter};
use std::io::{Seek, Write};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use bitflags::{bitflags, Flags};
use spa_sys::spa_pod_object_body;

use pipewire_wrapper_proc_macro::RawWrapper;
use prop::ObjectPropType;
use prop_info::ObjectPropInfoType;

use crate::spa::param::ParamType;
use crate::spa::pod::id::{PodIdRef, PodIdType};
use crate::spa::pod::iterator::PodIterator;
use crate::spa::pod::object::enum_format::ObjectEnumFormatType;
use crate::spa::pod::object::format::{MediaSubType, MediaType, ObjectFormatType};
use crate::spa::pod::object::param_buffers::ParamBuffersType;
use crate::spa::pod::object::param_io::ParamIoType;
use crate::spa::pod::object::param_latency::ParamLatencyType;
use crate::spa::pod::object::param_meta::ParamMetaType;
use crate::spa::pod::object::param_port_config::ParamPortConfigType;
use crate::spa::pod::object::param_process_latency::ParamProcessLatencyType;
use crate::spa::pod::object::param_profile::ParamProfileType;
use crate::spa::pod::object::param_route::ParamRouteType;
use crate::spa::pod::object::profiler::ProfilerType;
use crate::spa::pod::pod_buf::{AllocPod, PodBuf};
use crate::spa::pod::restricted::{CloneTo, PodHeader, PodRawValue};
use crate::spa::pod::{
    BasicTypePod, FromValue, PodError, PodRef, PodResult, PodValue, SizedPod, WritePod,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

use super::restricted::{write_align_padding, write_header, write_value};

pub mod enum_format;
pub mod format;
pub mod param_buffers;
pub mod param_io;
pub mod param_latency;
pub mod param_meta;
pub mod param_port_config;
pub mod param_process_latency;
pub mod param_profile;
pub mod param_route;
pub mod profiler;
pub mod prop;
pub mod prop_info;

#[derive(RawWrapper)]
#[repr(transparent)]
struct PodObjectBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_object_body,
}

impl PodObjectBodyRef {
    pub fn type_(&self) -> Type {
        Type::from_raw(self.raw.type_)
    }

    pub fn id(&self) -> u32 {
        self.raw.id
    }
}

impl Debug for PodObjectBodyRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PodObjectBodyRef")
            .field("type", &self.type_())
            .field("id", &self.id())
            .finish()
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodObjectRef {
    #[raw]
    raw: spa_sys::spa_pod_object,
}

impl PodHeader for PodObjectRef {
    fn pod_header(&self) -> &spa_sys::spa_pod {
        &self.raw.pod
    }

    fn static_type() -> Type {
        Type::OBJECT
    }
}

impl Debug for PodObjectRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodObjectRef")
                .field("pod.type", &self.pod_type())
                .field("pod.size", &self.pod_size())
                .field("body_type", &self.body_type())
                .field("body_id", &self.body_id())
                .field(
                    "value",
                    &self.value().map(|v| match v {
                        ObjectType::OBJECT_PROP_INFO(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PROPS(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_FORMAT(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_ENUM_FORMAT(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_BUFFERS(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_META(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_IO(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_PROFILE(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_PORT_CONFIG(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_ROUTE(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PROFILER(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_LATENCY(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                        ObjectType::OBJECT_PARAM_PROCESS_LATENCY(iter) => {
                            iter.map(|p| format!("{:?}", p.value())).collect::<Vec<_>>()
                        }
                    }),
                )
                .finish()
        }
    }
}

impl PodObjectRef {
    fn body(&self) -> &PodObjectBodyRef {
        unsafe { PodObjectBodyRef::from_raw_ptr(&self.raw.body) }
    }

    pub fn body_type(&self) -> Type {
        self.body().type_()
    }

    pub fn body_id(&self) -> u32 {
        self.body().id()
    }

    pub fn set_body_id(&mut self, id: u32) {
        self.raw.body.id = id;
    }

    pub fn from_id_and_value(
        id: impl Into<u32>,
        value: &<&Self as PodValue>::Value,
    ) -> PodResult<AllocPod<Self>> {
        let mut obj = PodBuf::<Self>::from_value(value)?.into_pod();
        obj.as_pod_mut().set_body_id(id.into());
        Ok(obj)
    }

    /// Can be used to retrieve object value with specified body.id value,
    /// because body.id value can be Invalid for some reason.
    pub fn param_value(&self, param_type: ParamType) -> PodResult<ObjectType> {
        PodObjectRef::parse_raw_body(
            self.body().as_raw_ptr(),
            self.pod_header().size as usize,
            self.body_type(),
            param_type.raw,
        )
    }

    fn parse_raw_body<'a>(
        body_ptr: *const spa_pod_object_body,
        size: usize,
        body_type: Type,
        body_id: u32,
    ) -> PodResult<ObjectType<'a>> {
        let first_element_ptr = unsafe { body_ptr.offset(1) };
        let size = size - size_of::<spa_sys::spa_pod_object_body>();

        Ok(match body_type {
            Type::OBJECT_PROP_INFO => {
                ObjectType::OBJECT_PROP_INFO(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PROPS => {
                ObjectType::OBJECT_PROPS(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_FORMAT => {
                if body_id == ParamType::FORMAT.raw {
                    ObjectType::OBJECT_FORMAT(PodIterator::new(first_element_ptr.cast(), size))
                } else {
                    // Parse as EnumFormat by default, because it can handle the both variants
                    ObjectType::OBJECT_ENUM_FORMAT(PodIterator::new(first_element_ptr.cast(), size))
                }
            }
            Type::OBJECT_PARAM_BUFFERS => {
                ObjectType::OBJECT_PARAM_BUFFERS(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_META => {
                ObjectType::OBJECT_PARAM_META(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_IO => {
                ObjectType::OBJECT_PARAM_IO(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_PROFILE => {
                ObjectType::OBJECT_PARAM_PROFILE(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_PORT_CONFIG => ObjectType::OBJECT_PARAM_PORT_CONFIG(
                PodIterator::new(first_element_ptr.cast(), size),
            ),
            Type::OBJECT_PARAM_ROUTE => {
                ObjectType::OBJECT_PARAM_ROUTE(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PROFILER => {
                ObjectType::OBJECT_PROFILER(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_LATENCY => {
                ObjectType::OBJECT_PARAM_LATENCY(PodIterator::new(first_element_ptr.cast(), size))
            }
            Type::OBJECT_PARAM_PROCESS_LATENCY => ObjectType::OBJECT_PARAM_PROCESS_LATENCY(
                PodIterator::new(first_element_ptr.cast(), size),
            ),
            type_ => return Err(PodError::UnexpectedObjectType(type_.raw)),
        })
    }
}

impl<'a> PodRawValue for &'a PodObjectRef {
    type RawValue = spa_sys::spa_pod_object_body;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw.body
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        let body = unsafe { PodObjectBodyRef::from_raw_ptr(ptr) };
        PodObjectRef::parse_raw_body(body.as_raw_ptr(), size, body.type_(), body.id())
    }
}

impl<'a> PodValue for &'a PodObjectRef {
    type Value = ObjectType<'a>;
    fn value(&self) -> PodResult<Self::Value> {
        Self::parse_raw_value(self.raw_value_ptr(), self.pod_header().size as usize)
    }
}

impl<'a> WritePod for &'a PodObjectRef {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        let (type_, content) = unsafe {
            match value {
                ObjectType::OBJECT_PROP_INFO(iter) => (Type::OBJECT_PROP_INFO, iter.as_bytes()),
                ObjectType::OBJECT_PROPS(iter) => (Type::OBJECT_PROPS, iter.as_bytes()),
                ObjectType::OBJECT_FORMAT(iter) => (Type::OBJECT_FORMAT, iter.as_bytes()),
                ObjectType::OBJECT_ENUM_FORMAT(iter) => (Type::OBJECT_FORMAT, iter.as_bytes()),
                ObjectType::OBJECT_PARAM_BUFFERS(iter) => {
                    (Type::OBJECT_PARAM_BUFFERS, iter.as_bytes())
                }
                ObjectType::OBJECT_PARAM_META(iter) => (Type::OBJECT_PARAM_META, iter.as_bytes()),
                ObjectType::OBJECT_PARAM_IO(iter) => (Type::OBJECT_PARAM_IO, iter.as_bytes()),
                ObjectType::OBJECT_PARAM_PROFILE(iter) => {
                    (Type::OBJECT_PARAM_PROFILE, iter.as_bytes())
                }
                ObjectType::OBJECT_PARAM_PORT_CONFIG(iter) => {
                    (Type::OBJECT_PARAM_PORT_CONFIG, iter.as_bytes())
                }
                ObjectType::OBJECT_PARAM_ROUTE(iter) => (Type::OBJECT_PARAM_ROUTE, iter.as_bytes()),
                ObjectType::OBJECT_PROFILER(iter) => (Type::OBJECT_PROFILER, iter.as_bytes()),
                ObjectType::OBJECT_PARAM_LATENCY(iter) => {
                    (Type::OBJECT_PARAM_LATENCY, iter.as_bytes())
                }
                ObjectType::OBJECT_PARAM_PROCESS_LATENCY(iter) => {
                    (Type::OBJECT_PARAM_PROCESS_LATENCY, iter.as_bytes())
                }
            }
        };
        write_header(
            buffer,
            (size_of::<spa_sys::spa_pod_object_body>() + content.len()) as u32,
            Type::OBJECT,
        )?;
        write_value(
            buffer,
            &spa_sys::spa_pod_object_body {
                type_: type_.raw,
                id: 0,
            },
        )?;
        buffer.write_all(content)?;
        write_align_padding(buffer)
    }
}

pub trait PodPropKeyType<'a>
where
    Self: 'a,
    Self: TryFrom<&'a PodPropRef<'a, Self>, Error = PodError>,
    Self: Debug,
{
    fn write_prop<W>(&self, buffer: &mut W) -> PodResult<()>
    where
        W: Write + Seek;

    fn write_pod_prop<W, T>(buffer: &mut W, key: u32, flags: u32, pod: &T) -> PodResult<()>
    where
        W: Write + Seek,
        T: WritePod,
        T: CloneTo,
    {
        buffer.write_all(&key.to_ne_bytes())?;
        buffer.write_all(&flags.to_ne_bytes())?;
        pod.clone_to(buffer)
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodPropRef<'a, T: PodPropKeyType<'a>> {
    #[raw]
    raw: spa_sys::spa_pod_prop,
    phantom_type: PhantomData<&'a T>,
}

bitflags! {
    #[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
    #[repr(transparent)]
    pub struct PodPropFlags: u32 {
        const READONLY = spa_sys::SPA_POD_PROP_FLAG_READONLY;
        const HARDWARE = spa_sys::SPA_POD_PROP_FLAG_HARDWARE;
        const HINT_DICT = spa_sys::SPA_POD_PROP_FLAG_HINT_DICT;
        const MANDATORY = spa_sys::SPA_POD_PROP_FLAG_MANDATORY;
        const DONT_FIXATE = spa_sys::SPA_POD_PROP_FLAG_DONT_FIXATE;
    }
}

impl<'a, T: PodPropKeyType<'a>> SizedPod for PodPropRef<'a, T> {
    fn pod_size(&self) -> usize {
        size_of::<PodPropRef<T>>() + self.pod().size() as usize
    }
}

impl<'a, T: PodPropKeyType<'a>> PodPropRef<'a, T> {
    pub fn key(&self) -> u32 {
        self.raw.key
    }

    pub fn flags(&self) -> PodPropFlags {
        PodPropFlags::from_bits_retain(self.raw.flags)
    }

    pub fn set_flags(&mut self, flags: PodPropFlags) {
        self.raw.flags = flags.bits();
    }

    pub fn pod(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.value) as *const spa_sys::spa_pod) }
    }
}

impl<'a, T: PodPropKeyType<'a>> PodRawValue for &'a PodPropRef<'a, T> {
    type RawValue = spa_sys::spa_pod_prop;

    fn raw_value_ptr(&self) -> *const Self::RawValue {
        &self.raw
    }

    fn parse_raw_value(ptr: *const Self::RawValue, size: usize) -> PodResult<Self::Value> {
        unsafe { PodPropRef::from_raw_ptr(ptr).try_into() }
    }
}

impl<'a, T: PodPropKeyType<'a>> PodValue for &'a PodPropRef<'a, T> {
    type Value = T;
    fn value(&self) -> PodResult<Self::Value> {
        let size = size_of::<<Self as PodRawValue>::RawValue>() + self.raw.value.size as usize;
        Self::parse_raw_value(self.raw_value_ptr(), size)
    }
}

impl<'a, T: PodPropKeyType<'a>> WritePod for &'a PodPropRef<'a, T> {
    fn write_pod<W>(buffer: &mut W, value: &<Self as PodValue>::Value) -> PodResult<()>
    where
        W: Write + Seek,
    {
        value.write_prop(buffer)
    }
}

impl<'a, T: PodPropKeyType<'a>> Debug for &'a PodPropRef<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        unsafe {
            f.debug_struct("PodPropRef")
                .field("key", &self.key())
                .field("flags", &self.flags())
                .field("pod", &self.pod())
                .field("value", &self.value())
                .finish()
        }
    }
}

pub type ObjectPropsIterator<'a, T> = PodIterator<'a, PodPropRef<'a, T>>;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ObjectType<'a> {
    OBJECT_PROP_INFO(ObjectPropsIterator<'a, ObjectPropInfoType<'a>>) = Type::OBJECT_PROP_INFO.raw,
    OBJECT_PROPS(ObjectPropsIterator<'a, ObjectPropType<'a>>) = Type::OBJECT_PROPS.raw,
    OBJECT_FORMAT(ObjectPropsIterator<'a, ObjectFormatType<'a>>) = Type::OBJECT_FORMAT.raw,
    OBJECT_ENUM_FORMAT(ObjectPropsIterator<'a, ObjectEnumFormatType<'a>>) =
        Type::_OBJECT_LAST.raw + Type::OBJECT_FORMAT.raw,
    OBJECT_PARAM_BUFFERS(ObjectPropsIterator<'a, ParamBuffersType<'a>>) =
        Type::OBJECT_PARAM_BUFFERS.raw,
    OBJECT_PARAM_META(ObjectPropsIterator<'a, ParamMetaType<'a>>) = Type::OBJECT_PARAM_META.raw,
    OBJECT_PARAM_IO(ObjectPropsIterator<'a, ParamIoType<'a>>) = Type::OBJECT_PARAM_IO.raw,
    OBJECT_PARAM_PROFILE(ObjectPropsIterator<'a, ParamProfileType<'a>>) =
        Type::OBJECT_PARAM_PROFILE.raw,
    OBJECT_PARAM_PORT_CONFIG(ObjectPropsIterator<'a, ParamPortConfigType<'a>>) =
        Type::OBJECT_PARAM_PORT_CONFIG.raw,
    OBJECT_PARAM_ROUTE(ObjectPropsIterator<'a, ParamRouteType<'a>>) = Type::OBJECT_PARAM_ROUTE.raw,
    OBJECT_PROFILER(ObjectPropsIterator<'a, ProfilerType<'a>>) = Type::OBJECT_PROFILER.raw,
    OBJECT_PARAM_LATENCY(ObjectPropsIterator<'a, ParamLatencyType<'a>>) =
        Type::OBJECT_PARAM_LATENCY.raw,
    OBJECT_PARAM_PROCESS_LATENCY(ObjectPropsIterator<'a, ParamProcessLatencyType<'a>>) =
        Type::OBJECT_PARAM_PROCESS_LATENCY.raw,
}

#[test]
fn from_value() {
    let mut allocated = PodObjectRef::from_value(&ObjectType::OBJECT_FORMAT(
        ObjectPropsIterator::build()
            .push_value(&ObjectFormatType::MEDIA_TYPE(
                PodIdRef::from_value(&MediaType::AUDIO).unwrap().as_pod(),
            ))
            .unwrap()
            .push_value(&ObjectFormatType::MEDIA_SUBTYPE(
                PodIdRef::from_value(&MediaSubType::DSP).unwrap().as_pod(),
            ))
            .unwrap()
            .into_pod_iter()
            .iter(),
    ))
    .unwrap();
    allocated.as_pod_mut().set_body_id(123);

    assert_eq!(allocated.as_pod().body_id(), 123);

    if let ObjectType::OBJECT_FORMAT(mut props) =
        allocated.as_pod().param_value(ParamType::FORMAT).unwrap()
    {
        if let ObjectFormatType::MEDIA_TYPE(v) = props.next().unwrap().value().unwrap() {
            assert_eq!(v.value().unwrap(), MediaType::AUDIO);
        } else {
            panic!()
        }
        if let ObjectFormatType::MEDIA_SUBTYPE(v) = props.next().unwrap().value().unwrap() {
            assert_eq!(v.value().unwrap(), MediaSubType::DSP);
        } else {
            panic!()
        }
        assert!(props.next().is_none())
    } else {
        panic!()
    }
}
