use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, Token, Type};

use crate::derive_raw_wrapper::WrappedRawStructInfo;
use crate::ARG_METHODS;

struct SpaInterfaceAttr {
    methods: Type,
}

impl Parse for SpaInterfaceAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let methods_arg_ident: Ident = input.parse()?;
        let _name_value_separator: Token![=] = input.parse()?;
        let methods_arg_value: Type = input.parse()?;

        if methods_arg_ident == ARG_METHODS {
            Ok(Self {
                methods: methods_arg_value,
            })
        } else {
            Err(input.error(format!(
                "Expected single methods = MethodsStructType attribute argument, found {}",
                input
            )))
        }
    }
}

pub fn spa_interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedRawStructInfo>(input.clone()) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };
    let spa_interface_attr = match parse2::<SpaInterfaceAttr>(attr) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_field_ident = &struct_info.raw_field.ident;
    let methods_type = spa_interface_attr.methods;

    quote!(
        #input

        impl crate::wrapper::SpaInterface for #struct_ident {
            type Methods = #methods_type;

            fn spa_interface(&self) -> &crate::spa::interface::InterfaceRef {
                use crate::wrapper::RawWrapper;
                unsafe {
                    assert_ne!(0, std::mem::size_of::<#struct_ident>(),
                        "Objects with spa_interface should contain the iface pointer, they cannot \
                        be zero-size pointers. Probably #[pw_interface(...)] should be used here");
                    crate::spa::interface::InterfaceRef::from_raw_ptr(
                        std::ptr::addr_of!(self.#raw_field_ident).cast())
                }
            }
        }
    )
}
