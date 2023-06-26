use std::fmt::{Debug, Formatter};
use std::marker::PhantomData;
use std::mem::size_of;
use std::ptr::addr_of;

use pipewire_proc_macro::RawWrapper;

use crate::spa::type_::pod::bitmap::PodBitmapRef;
use crate::spa::type_::pod::bytes::PodBytesRef;
use crate::spa::type_::pod::choice::PodChoiceRef;
use crate::spa::type_::pod::id::PodIdRef;
use crate::spa::type_::pod::object::PodObjectRef;
use crate::spa::type_::pod::sequence::PodSequenceRef;
use crate::spa::type_::pod::string::PodStringRef;
use crate::spa::type_::pod::struct_::PodStructRef;
use crate::spa::type_::pod::{
    BasicType, BasicTypePod, BasicTypeValue, PodBoolRef, PodDoubleRef, PodError, PodFdRef,
    PodFloatRef, PodFractionRef, PodIntRef, PodLongRef, PodPointerRef, PodRectangleRef, PodRef,
    PodResult, PodValueParser, ReadablePod, SizedPod,
};
use crate::spa::type_::Type;
use crate::wrapper::RawWrapper;

#[repr(u32)]
#[derive(Debug)]
#[allow(non_camel_case_types)]
pub enum ArrayIteratorType<'a> {
    NONE(ArrayIterator<'a, &'a PodRef>) = Type::NONE.raw,
    BOOL(ArrayIterator<'a, PodBoolRef>) = Type::BOOL.raw,
    ID(ArrayIterator<'a, PodIdRef<u32>>) = Type::ID.raw,
    INT(ArrayIterator<'a, PodIntRef>) = Type::INT.raw,
    LONG(ArrayIterator<'a, PodLongRef>) = Type::LONG.raw,
    FLOAT(ArrayIterator<'a, PodFloatRef>) = Type::FLOAT.raw,
    DOUBLE(ArrayIterator<'a, PodDoubleRef>) = Type::DOUBLE.raw,
    STRING(ArrayIterator<'a, &'a PodStringRef>) = Type::STRING.raw,
    BYTES(ArrayIterator<'a, &'a PodBytesRef>) = Type::BYTES.raw,
    RECTANGLE(ArrayIterator<'a, PodRectangleRef>) = Type::RECTANGLE.raw,
    FRACTION(ArrayIterator<'a, PodFractionRef>) = Type::FRACTION.raw,
    BITMAP(ArrayIterator<'a, &'a PodBitmapRef>) = Type::BITMAP.raw,
    ARRAY(ArrayIterator<'a, &'a PodArrayRef>) = Type::ARRAY.raw,
    // STRUCT(ArrayIterator<'a, PodStructRef>) = Type::STRUCT.raw,
    // OBJECT(ArrayIterator<'a, PodObjectRef>) = Type::OBJECT.raw,
    // SEQUENCE(ArrayIterator<'a, PodSequenceRef>) = Type::SEQUENCE.raw,
    // POINTER(ArrayIterator<'a, PodPointerRef>) = Type::POINTER.raw,
    // FD(ArrayIterator<'a, PodFdRef>) = Type::FD.raw,
    // CHOICE(ArrayIterator<'a, PodChoiceRef>) = Type::CHOICE.raw,
    POD(ArrayIterator<'a, &'a PodRef>) = Type::POD.raw,
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodArrayBodyRef {
    #[raw]
    raw: spa_sys::spa_pod_array_body,
}

impl PodArrayBodyRef {
    unsafe fn content_ptr(&self) -> *const u8 {
        (self.as_raw_ptr() as *const u8).offset(size_of::<PodArrayBodyRef>() as isize)
    }

    fn child(&self) -> &PodRef {
        unsafe { PodRef::from_raw_ptr(addr_of!(self.raw.child)) }
    }
}

#[derive(RawWrapper)]
#[repr(transparent)]
pub struct PodArrayRef {
    #[raw]
    raw: spa_sys::spa_pod_array,
}

impl SizedPod for PodArrayRef {
    fn size_bytes(&self) -> usize {
        self.upcast().size_bytes()
    }
}

impl BasicTypePod for PodArrayRef {}

impl<'a> ReadablePod for &'a PodArrayRef {
    type Value = ArrayIteratorType<'a>;

    fn value(&self) -> PodResult<Self::Value> {
        let content_size = self.size_bytes() - size_of::<PodArrayRef>();
        Self::parse(content_size as u32, self.body())
    }
}

impl Debug for PodArrayRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_list = f.debug_list();
        //let mut index = 0;
        // while index < self.elements() {
        //     debug_list.entry(self.element(index));
        //     index += 1;
        // }
        debug_list.finish()
    }
}

impl PodArrayRef {
    fn body(&self) -> &PodArrayBodyRef {
        unsafe { PodArrayBodyRef::from_raw_ptr(addr_of!(self.raw.body)) }
    }

    fn elements(&self) -> u32 {
        self.raw.pod.size / self.raw.body.child.size
    }

    pub fn element(&self, index: u32) -> PodResult<BasicTypeValue> {
        if self.elements() >= index {
            Err(PodError::IndexIsOutOfRange)
        } else {
            let first_element_ptr: *const u8 = unsafe { self.body().content_ptr() };
            let ptr = unsafe {
                first_element_ptr.offset(index as isize * self.body().child().size() as isize)
            };
            let size = self.raw.pod.size;
            match self.body().child().type_() {
                Type::NONE => <&PodRef>::parse(size, ptr).map(|v| BasicTypeValue::NONE(v)),
                Type::BOOL => <PodBoolRef>::parse(size, ptr).map(|v| BasicTypeValue::BOOL(v)),
                Type::ID => <PodIdRef<u32>>::parse(size, ptr).map(|v| BasicTypeValue::ID(v)),
                Type::INT => <PodIntRef>::parse(size, ptr).map(|v| BasicTypeValue::INT(v)),
                Type::LONG => <PodLongRef>::parse(size, ptr).map(|v| BasicTypeValue::LONG(v)),
                Type::FLOAT => <PodFloatRef>::parse(size, ptr).map(|v| BasicTypeValue::FLOAT(v)),
                Type::DOUBLE => <PodDoubleRef>::parse(size, ptr).map(|v| BasicTypeValue::DOUBLE(v)),
                Type::STRING => {
                    <&PodStringRef>::parse(size, ptr).map(|v| BasicTypeValue::STRING(v))
                }
                Type::BYTES => <&PodBytesRef>::parse(size, ptr).map(|v| BasicTypeValue::BYTES(v)),
                Type::RECTANGLE => {
                    <PodRectangleRef>::parse(size, ptr).map(|v| BasicTypeValue::RECTANGLE(v))
                }
                Type::FRACTION => {
                    <PodFractionRef>::parse(size, ptr).map(|v| BasicTypeValue::FRACTION(v))
                }
                Type::BITMAP => {
                    <&PodBitmapRef>::parse(size, ptr).map(|v| BasicTypeValue::BITMAP(v))
                }
                Type::ARRAY => <&PodArrayRef>::parse(size, ptr).map(|v| BasicTypeValue::ARRAY(v)),
                // Type::STRUCT => <&PodStructRef>::parse(size, element_ptr).map(|v| BasicTypeValue::STRUCT(v)),
                // Type::OBJECT => <&PodObjectRef>::parse(size, element_ptr).map(|v| BasicTypeValue::OBJECT(v)),
                // Type::SEQUENCE => <&PodSequenceRef>::parse(size, element_ptr).map(|v| BasicTypeValue::SEQUENCE(v)),
                // Type::POINTER => <&PodPointerRef>::parse(size, element_ptr).map(|v| BasicTypeValue::POINTER(v)),
                // Type::FD => <PodFdRef>::parse(size, element_ptr).map(|v| BasicTypeValue::FD(v)),
                // Type::CHOICE => <&PodChoiceRef>::parse(size, element_ptr).map(|v| BasicTypeValue::CHOICE(v)),
                Type::POD => <&PodRef>::parse(size, ptr).map(|v| BasicTypeValue::POD(v)),
                _ => Err(PodError::UnknownPodTypeToDowncast),
            }
        }
    }
}

impl<'a> PodValueParser<&'a PodArrayBodyRef> for &'a PodArrayRef {
    type To = ArrayIteratorType<'a>;

    fn parse(s: u32, b: &'a PodArrayBodyRef) -> PodResult<Self::To> {
        match b.child().type_() {
            Type::NONE => Ok(ArrayIteratorType::NONE(ArrayIterator::new(b, s))),
            Type::BOOL => Ok(ArrayIteratorType::BOOL(ArrayIterator::new(b, s))),
            Type::ID => Ok(ArrayIteratorType::ID(ArrayIterator::new(b, s))),
            Type::INT => Ok(ArrayIteratorType::INT(ArrayIterator::new(b, s))),
            Type::LONG => Ok(ArrayIteratorType::LONG(ArrayIterator::new(b, s))),
            Type::FLOAT => Ok(ArrayIteratorType::FLOAT(ArrayIterator::new(b, s))),
            Type::DOUBLE => Ok(ArrayIteratorType::DOUBLE(ArrayIterator::new(b, s))),
            Type::STRING => Ok(ArrayIteratorType::STRING(ArrayIterator::new(b, s))),
            Type::BYTES => Ok(ArrayIteratorType::BYTES(ArrayIterator::new(b, s))),
            Type::RECTANGLE => Ok(ArrayIteratorType::RECTANGLE(ArrayIterator::new(b, s))),
            Type::FRACTION => Ok(ArrayIteratorType::FRACTION(ArrayIterator::new(b, s))),
            Type::BITMAP => Ok(ArrayIteratorType::BITMAP(ArrayIterator::new(b, s))),
            Type::ARRAY => Ok(ArrayIteratorType::ARRAY(ArrayIterator::new(b, s))),
            // Type::STRUCT => Ok(ArrayIteratorType::STRUCT(ArrayIterator::new( value, &size))),
            // Type::OBJECT => Ok(ArrayIteratorType::OBJECT(ArrayIterator::new( value, &size))),
            // Type::SEQUENCE => Ok(ArrayIteratorType::SEQUENCE(ArrayIterator::new( value, &size))),
            // Type::POINTER => Ok(ArrayIteratorType::POINTER(ArrayIterator::new( value, &size))),
            // Type::FD => Ok(ArrayIteratorType::FD(ArrayIterator::new( value, &size))),
            // Type::CHOICE => Ok(ArrayIteratorType::CHOICE(ArrayIterator::new( value, &size))),
            Type::POD => Ok(ArrayIteratorType::POD(ArrayIterator::new(b, s))),
            _ => Err(PodError::UnknownPodTypeToDowncast),
        }
    }
}

impl<'a> PodValueParser<*const u8> for &'a PodArrayRef {
    type To = ArrayIteratorType<'a>;

    fn parse(size: u32, value: *const u8) -> PodResult<Self::To> {
        unsafe { Self::parse(size, &*(value as *const PodArrayBodyRef)) }
    }
}

pub struct ArrayIterator<'a, T: PodValueParser<*const u8>> {
    body: &'a PodArrayBodyRef,
    size: u32,
    first_element_ptr: *const u8,
    current_element_ptr: *const u8,
    phantom: PhantomData<T>,
}

impl<'a, T: PodValueParser<*const u8>> ArrayIterator<'a, T> {
    pub fn new(body: &'a PodArrayBodyRef, size: u32) -> Self {
        let first_element_ptr = unsafe { body.content_ptr() };
        Self {
            body,
            size,
            first_element_ptr,
            current_element_ptr: first_element_ptr,
            phantom: PhantomData::default(),
        }
    }

    unsafe fn inside(&self, ptr: *const u8) -> bool {
        let max_offset = self.size as isize;
        let offset = ptr.offset_from(self.first_element_ptr);
        offset < max_offset && (offset + self.body.raw.child.size as isize) <= max_offset
    }

    unsafe fn next_ptr(&self, ptr: *const u8) -> *const u8 {
        ptr.offset(self.body.raw.child.size as isize)
    }
}

impl<T: PodValueParser<*const u8>> Debug for ArrayIterator<'_, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArrayIterator").finish()
    }
}

impl<'a, T: PodValueParser<*const u8>> Iterator for ArrayIterator<'a, T> {
    type Item = T::To;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let current_element_ptr = self.current_element_ptr;
            if self.inside(current_element_ptr) {
                self.current_element_ptr = self.next_ptr(current_element_ptr);
                T::parse(self.body.raw.child.size, current_element_ptr).ok()
            } else {
                None
            }
        }
    }
}
