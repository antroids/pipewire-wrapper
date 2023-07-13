use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use crate::spa::interface::InterfaceRef;

pub trait Wrapper
where
    Self: Deref<Target = Self::RawWrapperType>,
    Self: DerefMut<Target = Self::RawWrapperType>,
    Self: AsRef<Self::RawWrapperType>,
    Self: AsMut<Self::RawWrapperType>,
    Self: Sized,
{
    type RawWrapperType: RawWrapper;

    #[must_use]
    fn into_raw(self) -> *mut <Self::RawWrapperType as RawWrapper>::CType {
        ManuallyDrop::new(self).as_raw()
    }

    fn as_raw(&self) -> *mut <Self::RawWrapperType as RawWrapper>::CType {
        self.as_ref().as_raw_ptr()
    }
}

pub trait RawWrapper
where
    Self: Sized,
{
    type CType;

    fn as_raw_ptr(&self) -> *mut Self::CType;
    fn as_raw(&self) -> &Self::CType;
    fn from_raw(raw: Self::CType) -> Self;
    unsafe fn from_raw_ptr<'a>(raw: *const Self::CType) -> &'a Self {
        Self::mut_from_raw_ptr(raw as *mut Self::CType)
    }
    unsafe fn mut_from_raw_ptr<'a>(raw: *mut Self::CType) -> &'a mut Self;
    fn as_ptr(&self) -> *mut Self {
        self.as_raw_ptr() as *mut Self
    }
}

pub trait SpaInterface: RawWrapper {
    type Methods;

    fn spa_interface(&self) -> &InterfaceRef;

    fn version(&self) -> u32 {
        self.spa_interface().version()
    }
}
