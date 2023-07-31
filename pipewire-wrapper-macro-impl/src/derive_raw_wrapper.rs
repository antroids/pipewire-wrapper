use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Field, Generics};

use crate::ATTR_RAW;

pub(crate) struct WrappedRawStructInfo {
    pub struct_ident: Ident,
    pub struct_generics: Generics,
    pub raw_field: Field,
    pub other_fields: Vec<Field>,
}

impl Parse for WrappedRawStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match crate::parse_wrapped_struct_info(input, ATTR_RAW) {
            Ok((struct_ident, struct_generics, raw_field, other_fields)) => {
                Ok(WrappedRawStructInfo {
                    struct_ident,
                    struct_generics,
                    raw_field,
                    other_fields,
                })
            }
            Err(error) => Err(error),
        }
    }
}

pub fn derive_raw_wrapper(input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedRawStructInfo>(input) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_field_ident = &struct_info.raw_field.ident;
    let raw_field_type = &struct_info.raw_field.ty;
    let other_fields_idents: Vec<&Ident> = struct_info
        .other_fields
        .iter()
        .map(|f| {
            f.ident
                .as_ref()
                .expect("Anonymous fields are not supported")
        })
        .collect();

    let mut struct_generics_without_default = struct_info.struct_generics.clone();
    crate::strip_defaults_from_generics(&mut struct_generics_without_default);

    let mut struct_generics_without_bounds = struct_generics_without_default.clone();
    crate::strip_bounds_from_generics(&mut struct_generics_without_bounds);

    quote!(
        impl #struct_generics_without_default crate::wrapper::RawWrapper for #struct_ident #struct_generics_without_bounds {
            type CType = #raw_field_type;

            fn as_raw_ptr(&self) -> *mut Self::CType {
                std::ptr::addr_of!(self.#raw_field_ident) as *mut _
            }

            fn as_raw(&self) -> &Self::CType {
                &self.#raw_field_ident
            }

            fn from_raw(raw: Self::CType) -> Self {
                Self {
                    #raw_field_ident: raw,
                    #(#other_fields_idents: std::default::Default::default()),*
                }
            }

            unsafe fn mut_from_raw_ptr<'lft>(raw: *mut Self::CType) -> &'lft mut Self {
                (raw as *mut #struct_ident #struct_generics_without_bounds).as_mut().unwrap()
            }
        }

        impl #struct_generics_without_default From<#raw_field_type> for #struct_ident #struct_generics_without_bounds {
            fn from(value: #raw_field_type) -> Self {
                use crate::wrapper::RawWrapper;
                Self::from_raw(value)
            }
        }

        impl #struct_generics_without_default From<#struct_ident #struct_generics_without_bounds> for #raw_field_type {
            fn from(value: #struct_ident #struct_generics_without_bounds) -> Self {
                value.#raw_field_ident
            }
        }
    )
}
