use std::error::Error;
use std::ffi::{c_char, CStr, CString};
use std::fmt::{Debug, Formatter};

#[derive(Copy, Clone)]
pub struct TypeInfo<'a> {
    type_: &'a CStr,
}

impl<'a> TypeInfo<'a> {
    pub const fn from_c_str(type_: &'a CStr) -> Self {
        Self { type_ }
    }

    pub const unsafe fn from_bytes_with_nul_unchecked(type_bytes: &'a [u8]) -> Self {
        let type_ = CStr::from_bytes_with_nul_unchecked(type_bytes);
        Self { type_ }
    }

    pub fn short_type_bytes(&self) -> &'a [u8] {
        let prefix_len = self.type_kind().map_or(0, |tk| tk.prefix().len());
        &self.type_.to_bytes_with_nul()[prefix_len..]
    }

    pub fn full_type_bytes(&self) -> &'a [u8] {
        self.type_.to_bytes_with_nul()
    }

    pub fn as_ptr(&self) -> *const c_char {
        self.type_.as_ptr()
    }

    pub fn type_kind(&self) -> crate::Result<TypeKind> {
        let type_bytes = self.type_.to_bytes_with_nul();
        let type_kind = if Self::starts_with(type_bytes, TypeKind::Interface.prefix()) {
            TypeKind::Interface
        } else if Self::starts_with(type_bytes, TypeKind::Object.prefix()) {
            TypeKind::Object
        } else {
            return Err(crate::Error::ErrorMessage("Unexpected type!"));
        };
        Ok(type_kind)
    }

    const fn starts_with(bytes: &'a [u8], prefix: &'a [u8]) -> bool {
        let prefix_len = prefix.len() - 1;
        let bytes_len = bytes.len() - 1;

        if bytes_len < prefix_len {
            return false;
        }

        let mut index = 0;
        while index < prefix_len && bytes[index] == prefix[index] {
            index += 1;
        }
        index == prefix_len
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TypeKind {
    Object,
    Interface,
}

impl TypeKind {
    const fn prefix(&self) -> &'static [u8] {
        match self {
            TypeKind::Object => pw_sys::PW_TYPE_INFO_OBJECT_BASE,
            TypeKind::Interface => pw_sys::PW_TYPE_INFO_INTERFACE_BASE,
        }
    }
}

pub trait WithTypeInfo {
    fn type_info(&self) -> &TypeInfo;
}

impl PartialEq for TypeInfo<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.type_ == other.type_
    }
}

impl Eq for TypeInfo<'_> {}

impl Debug for TypeInfo<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TypeInfo")
            .field("type", &self.type_)
            .finish()
    }
}

#[macro_export]
macro_rules! interface_type {
    ($interface_name:literal) => {{
        unsafe {
            crate::core_api::type_info::TypeInfo::from_bytes_with_nul_unchecked(
                concat!("PipeWire:Interface:", $interface_name, "\0").as_bytes(),
            )
        }
    }};
}

#[cfg(test)]
const TEST_INTERFACE: TypeInfo = interface_type!("test_int");
#[cfg(test)]
const TEST_INTERFACE_2: TypeInfo = interface_type!("test_int_2");

#[test]
fn test_create_interface() {
    let type_ =
        unsafe { TypeInfo::from_bytes_with_nul_unchecked(b"PipeWire:Interface:test_int\0") };
    assert_eq!(type_, TEST_INTERFACE);
    assert_eq!(
        type_.type_kind().unwrap(),
        TEST_INTERFACE.type_kind().unwrap()
    );
    assert_eq!(type_.short_type_bytes(), TEST_INTERFACE.short_type_bytes());
    assert_eq!(type_.full_type_bytes(), TEST_INTERFACE.full_type_bytes());

    assert_ne!(type_, TEST_INTERFACE_2);
    assert_eq!(
        type_.type_kind().unwrap(),
        TEST_INTERFACE_2.type_kind().unwrap()
    );
    assert_ne!(
        type_.short_type_bytes(),
        TEST_INTERFACE_2.short_type_bytes()
    );
    assert_ne!(type_.full_type_bytes(), TEST_INTERFACE_2.full_type_bytes());
}
