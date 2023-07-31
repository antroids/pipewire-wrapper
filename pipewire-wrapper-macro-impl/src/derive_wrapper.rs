use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Field, Generics};

use crate::ATTR_RAW_WRAPPER;

struct WrappedStructInfo {
    struct_ident: Ident,
    struct_generics: Generics,
    raw_wrapper_field: Field,
}

impl Parse for WrappedStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match crate::parse_wrapped_struct_info(input, ATTR_RAW_WRAPPER) {
            Ok((struct_ident, struct_generics, raw_wrapper_field, _)) => Ok(WrappedStructInfo {
                struct_ident,
                struct_generics,
                raw_wrapper_field,
            }),
            Err(error) => Err(error),
        }
    }
}

pub fn derive_wrapper(input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedStructInfo>(input) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_wrapper_field_ident = &struct_info.raw_wrapper_field.ident;
    let raw_wrapper_field_type = crate::get_field_type(&struct_info.raw_wrapper_field);

    let mut struct_generics_without_default = struct_info.struct_generics.clone();
    crate::strip_defaults_from_generics(&mut struct_generics_without_default);

    let mut struct_generics_without_bounds = struct_generics_without_default.clone();
    crate::strip_bounds_from_generics(&mut struct_generics_without_bounds);

    quote!(
        impl #struct_generics_without_default crate::wrapper::Wrapper for #struct_ident #struct_generics_without_bounds {
            type RawWrapperType = #raw_wrapper_field_type;
        }

        impl #struct_generics_without_default AsRef<#raw_wrapper_field_type> for #struct_ident #struct_generics_without_bounds {
            fn as_ref(&self) -> &<Self as crate::wrapper::Wrapper>::RawWrapperType {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_ref() }
            }
        }

        impl #struct_generics_without_default AsMut<#raw_wrapper_field_type> for #struct_ident #struct_generics_without_bounds {
            fn as_mut(&mut self) -> &mut <Self as crate::wrapper::Wrapper>::RawWrapperType {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_mut() }
            }
        }

        impl #struct_generics_without_default std::ops::Deref for #struct_ident #struct_generics_without_bounds {
            type Target = <Self as crate::wrapper::Wrapper>::RawWrapperType;

            fn deref(&self) -> &Self::Target {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_ref() }
            }
        }

        impl #struct_generics_without_default std::ops::DerefMut for #struct_ident #struct_generics_without_bounds {
            fn deref_mut(&mut self) -> &mut Self::Target {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_mut() }
            }
        }
    )
}
