/*
 * SPDX-License-Identifier: MIT
 */
#![allow(unused)]

extern crate core;

use std::fmt::Debug;
use std::os::fd::RawFd;
use std::ptr::NonNull;

pub use pipewire_macro_impl::*;
pub use pipewire_proc_macro::*;

use crate::error::Error;
use crate::error::Error::ErrorCode;
use crate::wrapper::RawWrapper;

pub mod core_api;
pub mod error;
pub mod filter;
pub mod impl_api;
pub mod listeners;
pub mod macro_rules;
pub mod spa;
pub mod stream;
pub mod wrapper;

/// Invalid ID value
pub const SPA_ID_INVALID: u32 = 0xffffffff;

/// Result type used in the library
pub type Result<T> = std::result::Result<T, error::Error>;

/// Convert i32 result to the [Result<()>] value.
/// PipeWire uses i32 >= 0 for success and -err_core for errors.
/// # Arguments
///
/// * `result` - result received from PipeWire
fn i32_as_void_result(result: i32) -> crate::Result<()> {
    i32_as_result(result, ())
}

/// Convert i32 result to the [Result<T>] value.
/// PipeWire uses i32 >= 0 for success and -err_core for errors.
///
/// # Arguments
///
/// * `result` - result received from PipeWire
/// * `result_value` - value used as result on success
fn i32_as_result<T>(result: i32, result_value: T) -> crate::Result<T> {
    if result >= 0 {
        Ok(result_value)
    } else {
        Err(crate::error::Error::ErrorCode(-result as u32))
    }
}

/// NonNull from the external type pointer or [Error::CannotCreateInstance] if the ptr is null
fn new_instance_raw_wrapper<T: RawWrapper>(ptr: *mut T::CType) -> crate::Result<NonNull<T>> {
    NonNull::new(ptr as *mut T).ok_or_else(|| Error::CannotCreateInstance)
}

/// [RawWrapper] from the raw pointer to the external type or [Error::NullPointer]
fn raw_wrapper<'a, T: RawWrapper>(ptr: *mut T::CType) -> crate::Result<&'a T> {
    if let Some(raw_wrapper) = unsafe { (ptr as *mut T).as_ref() } {
        Ok(raw_wrapper)
    } else {
        Err(Error::NullPointer)
    }
}

#[test]
fn wrapper_tests() {
    let test_case = trybuild::TestCases::new();

    test_case.pass("tests/wrapper-on-struct-with-ptr-field.rs");
}

enum_wrapper!(TestEnum, u32, VAL1: 12u32, VAL2: 13u32);
enum_wrapper!(TestEnum2, u32, VAL11: 11u32, VAL22: 22u32,);
#[test]
fn test_enum_wrapper() {
    assert_eq!(*TestEnum::VAL1.as_raw(), 12u32);
    assert_eq!(format!("{:?}", TestEnum::VAL2), "VAL2");
    assert_eq!(format!("{:?}", TestEnum::from_raw(123u32)), "UNKNOWN(123)");
}
