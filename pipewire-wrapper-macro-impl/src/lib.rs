/*
 * SPDX-License-Identifier: MIT
 */
use proc_macro2::Ident;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{
    AttrStyle, Attribute, Field, GenericArgument, GenericParam, Generics, ItemStruct, Meta,
    PathArguments, Type,
};

pub mod derive_raw_wrapper;
pub mod derive_wrapper;
pub mod interface;
pub mod object_type_impl;
pub mod proxy_wrapper;
pub mod spa_interface;

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
