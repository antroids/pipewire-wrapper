use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemStruct, TypePath};

pub fn proxy_wrapper(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_info: ItemStruct = parse2(input.clone()).unwrap();
    let interface_attr: TypePath = parse2(attr).unwrap();

    let struct_ident = &struct_info.ident;
    let ref_type_ident = interface_attr.path.get_ident().unwrap();

    quote!(
        #input

        impl<'c> #struct_ident<'c> {
            pub fn proxy(&self) -> &crate::core_api::proxy::Proxy {
                &self.ref_
            }

            pub fn is_bound(&self) -> bool {
                self.ref_.is_bound()
            }
        }

        impl<'c> crate::wrapper::Wrapper for #struct_ident <'c> {
            type RawWrapperType = #ref_type_ident;
        }

        impl <'c> std::ops::Deref for #struct_ident <'c> {
            type Target = #ref_type_ident;

            fn deref(&self) -> &'c Self::Target {
                unsafe { #ref_type_ident::mut_from_raw_ptr(self.ref_.as_raw_ptr().cast()) }
            }
        }

        impl<'c> std::ops::DerefMut for #struct_ident <'c> {
            fn deref_mut(&mut self) -> &'c mut Self::Target {
                unsafe { #ref_type_ident::mut_from_raw_ptr(self.ref_.as_raw_ptr().cast()) }
            }
        }

        impl<'c> AsRef<#ref_type_ident> for #struct_ident <'c> {
            fn as_ref(&self) -> &'c #ref_type_ident {
                unsafe { #ref_type_ident::mut_from_raw_ptr(self.ref_.as_raw_ptr().cast()) }
            }
        }

        impl<'c> AsMut<#ref_type_ident> for #struct_ident <'c> {
            fn as_mut(&mut self) -> &'c mut #ref_type_ident {
                unsafe { #ref_type_ident::mut_from_raw_ptr(self.ref_.as_raw_ptr().cast()) }
            }
        }
    )
}
