/*
 * SPDX-License-Identifier: MIT
 */

//! Wrappers for the external types
//!
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};

use crate::spa::interface::InterfaceRef;

/// Owned wrapper for the external type.
/// Should be used when external value can be created or dropped from the Rust side,
/// or when Wrapper should have additional fields.
/// Methods wrappers can be implemented in the [RawWrapper] and used from the both variants.
/// Usually Wrappers have only constructor and drop implementations.
pub trait Wrapper
where
    Self: Deref<Target = Self::RawWrapperType>,
    Self: DerefMut<Target = Self::RawWrapperType>,
    Self: AsRef<Self::RawWrapperType>,
    Self: AsMut<Self::RawWrapperType>,
    Self: Sized,
{
    /// [RawWrapper] type
    type RawWrapperType: RawWrapper;

    /// Converts the Wrapper into raw pointer, that must be dropped manually.
    /// Used when the external method takes the ownership.
    #[must_use]
    fn into_raw(self) -> *mut <Self::RawWrapperType as RawWrapper>::CType {
        ManuallyDrop::new(self).as_raw()
    }

    /// Raw mutable pointer to the external value.
    fn as_raw(&self) -> *mut <Self::RawWrapperType as RawWrapper>::CType {
        self.as_ref().as_raw_ptr()
    }
}

/// Wrapper for the external type, must not be null.
/// By convention, all wrapped structures should with the Ref prefix.
/// #\[repr(transparent)\] should be used where possible.
pub trait RawWrapper
where
    Self: Sized,
{
    /// External type
    type CType;

    /// Raw ptr to the external type
    fn as_raw_ptr(&self) -> *mut Self::CType;

    /// Wrapped external value
    fn as_raw(&self) -> &Self::CType;

    /// Creates wrapper from the external value, can be use when external type has no raw pointers.
    fn from_raw(raw: Self::CType) -> Self;

    /// Cast external pointer to the borrowed wrapper.
    /// Panic when pointer is null.
    /// Lifetime is not reliable and should be guaranteed explicitly.
    ///
    /// # Arguments
    ///
    /// * `raw` - raw pointer
    ///
    /// # Safety
    ///
    /// `raw` must be valid pointer to the structure
    unsafe fn from_raw_ptr<'a>(raw: *const Self::CType) -> &'a Self {
        Self::mut_from_raw_ptr(raw as *mut Self::CType)
    }

    /// Cast external pointer to the borrowed mutable wrapper.
    /// Panic when pointer is null.
    /// Lifetime is not reliable and should be guaranteed explicitly.
    ///
    /// # Arguments
    ///
    /// * `raw` - raw pointer
    /// # Safety
    ///
    /// `raw` must be valid pointer to the structure
    unsafe fn mut_from_raw_ptr<'a>(raw: *mut Self::CType) -> &'a mut Self;

    /// Raw mutable pointer to Self
    fn as_ptr(&self) -> *mut Self {
        self.as_raw_ptr() as *mut Self
    }
}

/// Provides support for [spa_sys::spa_interface] methods.
/// [spa_interface_call](crate::spa_interface_call) can used to call the methods
pub trait SpaInterface: RawWrapper {
    /// [spa_sys::spa_interface] methods structure
    type Methods;

    /// Interface wrapper
    fn spa_interface(&self) -> &InterfaceRef;

    /// Interface version
    fn version(&self) -> u32 {
        self.spa_interface().version()
    }
}
