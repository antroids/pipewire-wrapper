/*
 * SPDX-License-Identifier: MIT
 */
use proc_macro2::{Ident, Span, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    parse2, AttrStyle, Attribute, Expr, Field, Fields, GenericArgument, GenericParam, Generics,
    ItemEnum, ItemStruct, LifetimeParam, Lit, LitStr, Meta, MetaNameValue, PathArguments, Token,
    Type, TypePath, Variant,
};

const ATTR_RAW: &str = "raw";
const ATTR_RAW_WRAPPER: &str = "raw_wrapper";

const ARG_METHODS: &str = "methods";
const ARG_INTERFACE: &str = "interface";

fn escape_ident(ident_name: &str) -> &str {
    match ident_name {
        "type" => "type_",
        _ => ident_name,
    }
}

struct WrappedRawStructInfo {
    struct_ident: Ident,
    struct_generics: Generics,
    raw_field: Field,
    other_fields: Vec<Field>,
}

impl Parse for WrappedRawStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match parse_wrapped_struct_info(input, ATTR_RAW) {
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

struct WrappedStructInfo {
    struct_ident: Ident,
    struct_generics: Generics,
    raw_wrapper_field: Field,
}

impl Parse for WrappedStructInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        match parse_wrapped_struct_info(input, ATTR_RAW_WRAPPER) {
            Ok((struct_ident, struct_generics, raw_wrapper_field, _)) => Ok(WrappedStructInfo {
                struct_ident,
                struct_generics,
                raw_wrapper_field,
            }),
            Err(error) => Err(error),
        }
    }
}

struct ObjectTypeEnumInfo {
    ident_: Ident,
    lifetime: LifetimeParam,
    variants: Vec<Variant>,
}

impl Parse for ObjectTypeEnumInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let enum_: ItemEnum = input.parse()?;
        let ident_ = enum_.ident;
        let variants: Vec<Variant> = enum_.variants.into_iter().collect();

        if let Some(GenericParam::Lifetime(lifetime)) = enum_.generics.params.first() {
            Ok(Self {
                ident_,
                lifetime: lifetime.clone(),
                variants,
            })
        } else {
            Err(input.error("Only enums with single lifetime generic parameter are supported"))
        }
    }
}

impl ObjectTypeEnumInfo {
    fn struct_ident(&self) -> Ident {
        let enum_ident_string = self.ident_.to_string();
        let mut ident_string = enum_ident_string.trim_end_matches("Type").to_owned();
        ident_string.push_str("Info");
        Ident::new(ident_string.as_str(), self.ident_.span())
    }

    fn struct_fields_idents(&self) -> Vec<Ident> {
        self.variants
            .iter()
            .map(|variant| {
                Ident::new(
                    escape_ident(variant.ident.to_string().to_lowercase().as_str()),
                    variant.span(),
                )
            })
            .collect()
    }

    fn struct_fields_types(&self) -> Vec<Type> {
        self.variants
            .iter()
            .map(|variant| match variant.fields.next().unwrap() {
                Fields::Named(field) => field.named.first().unwrap().ty.clone(),
                Fields::Unnamed(field) => field.unnamed.first().unwrap().ty.clone(),
                Fields::Unit => panic!("Unit enum fields are not supported"),
            })
            .collect()
    }
}

fn parse_wrapped_struct_info(
    input: ParseStream,
    attr_ident_name: &'static str,
) -> syn::Result<(Ident, Generics, Field, Vec<Field>)> {
    let item_struct: ItemStruct = input.parse()?;

    let mut field_with_attr: Option<Field> = None;
    let mut other_fields: Vec<Field> = Vec::new();
    for field in item_struct.fields {
        if field
            .attrs
            .iter()
            .any(|attr| is_attribute_with_ident_name(attr, attr_ident_name))
        {
            field_with_attr = Some(field);
        } else {
            other_fields.push(field);
        }
    }

    if let Some(field_with_attr) = field_with_attr {
        Ok((
            item_struct.ident,
            item_struct.generics,
            field_with_attr,
            other_fields,
        ))
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
                return generic_type;
            }
        }
    }
    &field.ty
}

fn strip_defaults_from_generics(generics: &mut Generics) {
    generics.params.iter_mut().for_each(|p| {
        if let GenericParam::Type(ty) = p {
            ty.eq_token = None;
            ty.default = None;
        }
    });
}

fn strip_bounds_from_generics(generics: &mut Generics) {
    generics.params.iter_mut().for_each(|p| {
        if let GenericParam::Type(ty) = p {
            ty.colon_token = None;
            ty.bounds = Punctuated::default();
        }
    });
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
    strip_defaults_from_generics(&mut struct_generics_without_default);

    let mut struct_generics_without_bounds = struct_generics_without_default.clone();
    strip_bounds_from_generics(&mut struct_generics_without_bounds);

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

pub fn derive_wrapper(input: TokenStream) -> TokenStream {
    let struct_info = match parse2::<WrappedStructInfo>(input) {
        Ok(parsed) => parsed,
        Err(error) => return error.to_compile_error(),
    };

    let struct_ident = &struct_info.struct_ident;
    let raw_wrapper_field_ident = &struct_info.raw_wrapper_field.ident;
    let raw_wrapper_field_type = get_field_type(&struct_info.raw_wrapper_field);

    let mut struct_generics_without_default = struct_info.struct_generics.clone();
    strip_defaults_from_generics(&mut struct_generics_without_default);

    let mut struct_generics_without_bounds = struct_generics_without_default.clone();
    strip_bounds_from_generics(&mut struct_generics_without_bounds);

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

pub fn derive_object_info(input: TokenStream) -> TokenStream {
    let enum_info: ObjectTypeEnumInfo = parse2(input.clone()).unwrap();

    let enum_ident = &enum_info.ident_;
    let enum_variants_idents: Vec<Ident> = enum_info
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();
    let struct_ident = enum_info.struct_ident();
    let lifetime = &enum_info.lifetime;
    let struct_fields_idents = enum_info.struct_fields_idents();
    let struct_fields_types = enum_info.struct_fields_types();

    quote!(
        #[derive(Default, Debug)]
        pub struct #struct_ident< #lifetime > {
            #(pub #struct_fields_idents : Option<#struct_fields_types>),*
        }

        impl<#lifetime> TryFrom<crate::spa::pod::object::ObjectPropsIterator<#lifetime, #enum_ident<#lifetime>>>
            for #struct_ident<#lifetime>
        {
            type Error = crate::spa::pod::PodError;

            fn try_from(value: crate::spa::pod::object::ObjectPropsIterator<#lifetime, #enum_ident<#lifetime>>) -> Result<Self, Self::Error> {
                use crate::spa::pod::PodValue;
                let mut info = Self::default();
                for prop in value {
                    match prop.value()? {
                        #(#enum_ident::#enum_variants_idents (val) => info.#struct_fields_idents = Some(val),)*
                        _ => panic!("Unsupported type"),
                    };
                }
                Ok(info)
            }
        }
    )
}
