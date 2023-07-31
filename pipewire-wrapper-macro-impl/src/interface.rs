use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse2, Expr, Lit, LitStr, MetaNameValue, Token, Type, TypePath};

use crate::derive_raw_wrapper::WrappedRawStructInfo;
use crate::{ARG_INTERFACE, ARG_METHODS};

struct InterfaceAttr {
    methods: Type,
    interface: LitStr,
}

impl Parse for InterfaceAttr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut methods_arg_value: Option<Type> = None;
        let mut interface_arg_value: Option<LitStr> = None;

        let key_value_list: Punctuated<MetaNameValue, Token![,]> =
            Punctuated::parse_terminated(input)?;
        for key_value in key_value_list {
            if key_value.path.is_ident(ARG_METHODS) {
                if let Expr::Path(path) = key_value.value {
                    methods_arg_value = Some(Type::Path(TypePath {
                        qself: path.qself,
                        path: path.path,
                    }));
                } else {
                    return Err(input.error("Methods struct type path expected"));
                }
            } else if key_value.path.is_ident(ARG_INTERFACE) {
                if let Expr::Lit(lit) = key_value.value {
                    if let Lit::Str(lit_str) = lit.lit {
                        interface_arg_value = Some(lit_str);
                    }
                }
            } else {
                return Err(input.error("Unexpected attribute name"));
            }
        }

        if methods_arg_value.is_none() {
            return Err(input.error("Methods meta attribute is missing"));
        }
        if interface_arg_value.is_none() {
            return Err(input.error("Interface meta attribute is missing"));
        }

        Ok(Self {
            methods: methods_arg_value.unwrap(),
            interface: interface_arg_value.unwrap(),
        })
    }
}

pub fn interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedRawStructInfo>(input.clone()) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };
    let interface_attr = match parse2::<InterfaceAttr>(attr) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_field_ident = &struct_info.raw_field.ident;
    let methods_type = interface_attr.methods;
    let interface_name = interface_attr.interface;
    let interface_name_ident = Ident::new(
        format!("{}_TYPE_INFO", interface_name.value().to_uppercase()).as_str(),
        Span::call_site(),
    );

    quote!(
        #input

        pub const #interface_name_ident: crate::core_api::type_info::TypeInfo = crate::interface_type!(#interface_name);

        impl crate::core_api::proxy::Proxied for #struct_ident {
            fn type_info() -> crate::core_api::type_info::TypeInfo<'static> {
                #interface_name_ident
            }
        }

        impl crate::wrapper::SpaInterface for #struct_ident {
            type Methods = #methods_type;

            fn spa_interface(&self) -> &crate::spa::interface::InterfaceRef {
                use crate::wrapper::RawWrapper;
                unsafe {
                    assert_eq!(0, std::mem::size_of::<#struct_ident>(),
                        "Objects with pw_interface should not have any data, they are just zero-size \
                        pointers. Probably #[spa_interface(...)] should be used here");
                    crate::spa::interface::InterfaceRef::from_raw_ptr(
                        std::ptr::addr_of!(self.#raw_field_ident).cast())
                }
            }
        }
    )
}
