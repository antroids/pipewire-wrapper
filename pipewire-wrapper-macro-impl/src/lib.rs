use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parse2, AttrStyle, Attribute, Expr, Field, GenericArgument, Generics, ItemStruct, Lit, LitStr,
    Meta, MetaNameValue, PathArguments, Token, Type, TypePath,
};

pub mod macro_rules;

const ATTR_RAW: &'static str = "raw";
const ATTR_RAW_WRAPPER: &'static str = "raw_wrapper";

const ARG_METHODS: &'static str = "methods";
const ARG_INTERFACE: &'static str = "interface";

struct WrappedRawStructInfo {
    struct_ident: Ident,
    struct_generics: Generics,
    raw_field: Field,
}

impl Parse for WrappedRawStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match parse_wrapped_struct_info(input, ATTR_RAW) {
            Ok((struct_ident, struct_generics, raw_field)) => Ok(WrappedRawStructInfo {
                struct_ident,
                struct_generics,
                raw_field,
            }),
            Err(error) => Err(error),
        }
    }
}

struct WrappedStructInfo {
    struct_ident: Ident,
    struct_generics: Generics,
    raw_wrapper_field: Field,
}

impl Parse for WrappedStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match parse_wrapped_struct_info(input, ATTR_RAW_WRAPPER) {
            Ok((struct_ident, struct_generics, raw_wrapper_field)) => Ok(WrappedStructInfo {
                struct_ident,
                struct_generics,
                raw_wrapper_field,
            }),
            Err(error) => Err(error),
        }
    }
}

fn parse_wrapped_struct_info(
    input: ParseStream,
    attr_ident_name: &'static str,
) -> syn::Result<(Ident, Generics, Field)> {
    let item_struct: ItemStruct = input.parse()?;

    let field_with_attr = item_struct
        .fields
        .iter()
        .find(|field| {
            field
                .attrs
                .iter()
                .any(|attr| is_attribute_with_ident_name(attr, attr_ident_name))
        })
        .cloned();

    if let Some(field_with_attr) = field_with_attr {
        Ok((item_struct.ident, item_struct.generics, field_with_attr))
    } else {
        Err(input.error(format!(
            "Cannot find field with #[{}] attribute in wrapped struct",
            attr_ident_name
        )))
    }
}

fn is_attribute_with_ident_name(attr: &Attribute, ident_name: &'static str) -> bool {
    if let AttrStyle::Outer = attr.style {
        if let Meta::Path(path) = &attr.meta {
            if let Some(ident) = path.get_ident() {
                return ident == ident_name;
            }
        }
    }
    false
}

fn get_field_type(field: &Field) -> &Type {
    if let Type::Path(type_path) = &field.ty {
        let last_segment = type_path.path.segments.last().unwrap();
        if let PathArguments::AngleBracketed(generic_arg) = &last_segment.arguments {
            if let Some(GenericArgument::Type(generic_type)) = generic_arg.args.first() {
                return &generic_type;
            }
        }
    }
    &field.ty
}

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

struct ProxiedAttr {
    methods: Type,
    interface: LitStr,
}

impl Parse for ProxiedAttr {
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

pub fn derive_raw_wrapper(input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedRawStructInfo>(input) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let struct_generics = &struct_info.struct_generics;
    let raw_field_ident = &struct_info.raw_field.ident;
    let raw_field_type = &struct_info.raw_field.ty;

    quote!(
        impl #struct_generics crate::wrapper::RawWrapper for #struct_ident #struct_generics {
            type CType = #raw_field_type;

            fn as_raw_ptr(&self) -> *mut Self::CType {
                &self.#raw_field_ident as *const _ as *mut _
            }

            fn as_raw(&self) -> &Self::CType {
                &self.#raw_field_ident
            }

            fn from_raw(raw: Self::CType) -> Self {
                Self { #raw_field_ident: raw }
            }

            unsafe fn mut_from_raw_ptr<'a>(raw: *mut Self::CType) -> &'a mut Self {
                &mut *(raw as *mut #struct_ident #struct_generics)
            }
        }

        impl #struct_generics From<#raw_field_type> for #struct_ident #struct_generics {
            fn from(value: #raw_field_type) -> Self {
                use crate::wrapper::RawWrapper;
                Self::from_raw(value)
            }
        }
    )
}

pub fn derive_wrapper(input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedStructInfo>(input) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let struct_generics = &struct_info.struct_generics;
    let raw_wrapper_field_ident = &struct_info.raw_wrapper_field.ident;
    let raw_wrapper_field_type = get_field_type(&struct_info.raw_wrapper_field);

    quote!(
        impl #struct_generics crate::wrapper::Wrapper for #struct_ident #struct_generics {
            type RawWrapperType = #raw_wrapper_field_type;
        }

        impl #struct_generics AsRef<#raw_wrapper_field_type> for #struct_ident #struct_generics {
            fn as_ref(&self) -> &<Self as crate::wrapper::Wrapper>::RawWrapperType {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_ref() }
            }
        }

        impl #struct_generics AsMut<#raw_wrapper_field_type> for #struct_ident #struct_generics {
            fn as_mut(&mut self) -> &mut <Self as crate::wrapper::Wrapper>::RawWrapperType {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_mut() }
            }
        }

        impl #struct_generics std::ops::Deref for #struct_ident #struct_generics {
            type Target = <Self as crate::wrapper::Wrapper>::RawWrapperType;

            fn deref(&self) -> &Self::Target {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_ref() }
            }
        }

        impl #struct_generics std::ops::DerefMut for #struct_ident #struct_generics {
            fn deref_mut(&mut self) -> &mut Self::Target {
                use crate::wrapper::RawWrapper;
                unsafe { self.#raw_wrapper_field_ident.as_mut() }
            }
        }
    )
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
                        &self.#raw_field_ident.iface as *const spa_sys::spa_interface)
                }
            }
        }
    )
}

pub fn proxied(attr: TokenStream, input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedRawStructInfo>(input.clone()) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };
    let proxied_attr = match parse2::<ProxiedAttr>(attr) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_field_ident = &struct_info.raw_field.ident;
    let methods_type = proxied_attr.methods;
    let interface_name = proxied_attr.interface;
    let interface_name_ident = Ident::new(
        format!("{}_TYPE_INFO", interface_name.value().to_uppercase()).as_str(),
        Span::call_site(),
    );

    quote!(
        #input

        pub const #interface_name_ident: crate::core_api::type_info::TypeInfo = crate::interface_type!(#interface_name);

        impl crate::core_api::proxy::Proxied for #struct_ident {
            fn get_type_info() -> crate::core_api::type_info::TypeInfo<'static> {
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
                        &self.#raw_field_ident as *const _ as *const spa_sys::spa_interface)
                }
            }
        }
    )
}
