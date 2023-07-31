use std::ops::Deref;

use proc_macro2::{Ident, TokenStream};
use quote::__private::ext::RepToTokensExt;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{parse2, Fields, GenericParam, ItemEnum, LifetimeParam, Type, Variant};

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
    fn struct_ident_with_suffix(&self, suffix: &str) -> Ident {
        let enum_ident_string = self.ident_.to_string();
        let mut ident_string = enum_ident_string.trim_end_matches("Type").to_owned();
        ident_string.push_str(suffix);
        Ident::new(ident_string.as_str(), self.ident_.span())
    }

    fn info_struct_ident(&self) -> Ident {
        self.struct_ident_with_suffix("Info")
    }

    fn builder_struct_ident(&self) -> Ident {
        self.struct_ident_with_suffix("Builder")
    }

    fn struct_prop_value_fields_idents(&self) -> Vec<Ident> {
        self.variants
            .iter()
            .map(|variant| {
                Ident::new(
                    crate::escape_ident(variant.ident.to_string().to_lowercase().as_str()),
                    variant.span(),
                )
            })
            .collect()
    }

    fn struct_prop_flags_fields_idents(&self) -> Vec<Ident> {
        self.variants
            .iter()
            .map(|variant| {
                let mut ident = variant.ident.to_string().to_lowercase();
                ident.push_str("_flags");
                Ident::new(crate::escape_ident(ident.as_str()), variant.span())
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

pub fn object_type_impl(attr: TokenStream, input: TokenStream) -> TokenStream {
    let enum_info: ObjectTypeEnumInfo = parse2(input.clone()).unwrap();
    let object_type: Ident = parse2(attr).unwrap();

    let enum_ident = &enum_info.ident_;
    let enum_variants_idents: Vec<Ident> = enum_info
        .variants
        .iter()
        .map(|variant| variant.ident.clone())
        .collect();
    let info_struct_ident = enum_info.info_struct_ident();
    let builder_struct_ident = enum_info.builder_struct_ident();
    let lifetime = &enum_info.lifetime;
    let struct_value_fields_idents = enum_info.struct_prop_value_fields_idents();
    let struct_flags_fields_idents = enum_info.struct_prop_flags_fields_idents();
    let struct_fields_types = enum_info.struct_fields_types();
    let builder_fields_types: Vec<Type> = enum_info
        .struct_fields_types()
        .into_iter()
        .map(|type_| {
            if let Type::Reference(ref_) = type_ {
                ref_.elem.deref().clone()
            } else {
                type_
            }
        })
        .collect();

    quote!(
        #input

        /// Structure that contains all possible properties for object type.
        /// Can be converted from [PodObjectRef](crate::spa::pod::object::PodObjectRef)
        /// by [TryFrom] trait.
        #[cfg(feature = "spa-pod-object-info")]
        #[derive(Default, Debug)]
        pub struct #info_struct_ident<#lifetime> {
            pub body_id: u32,
            #(pub #struct_value_fields_idents: Option<#struct_fields_types>),*,
            #(pub #struct_flags_fields_idents: crate::spa::pod::object::PodPropFlags),*
        }

        #[cfg(feature = "spa-pod-object-info")]
        impl<#lifetime> TryFrom<&#lifetime crate::spa::pod::object::PodObjectRef> for #info_struct_ident<#lifetime> {
            type Error = PodError;

            fn try_from(value: &#lifetime crate::spa::pod::object::PodObjectRef) -> Result<Self, Self::Error> {
                use crate::spa::pod::PodValue;
                use crate::spa::pod::restricted::PodHeader;
                if let crate::spa::pod::object::ObjectType::#object_type(iter) = value.value()? {
                    let mut info = Self::default();
                    info.body_id = value.body_id();

                    for prop in iter {
                        match prop.value()? {
                            #(#enum_ident::#enum_variants_idents (val) => {
                                info.#struct_value_fields_idents = Some(val);
                                info.#struct_flags_fields_idents = prop.flags();
                            }),*,
                            _ => panic!("Unsupported type"),
                        };
                    }

                    Ok(info)
                } else {
                    Err(PodError::UnexpectedObjectType(value.body_type().into()))
                }
            }
        }

        /// The builder for given object type.
        /// Has setter methods for each property of the object.
        #[cfg(feature = "spa-pod-object-builders")]
        #[derive(Default)]
        pub struct #builder_struct_ident<'a> {
            body_id: u32,

            #(#struct_value_fields_idents: Option<<&'a #builder_fields_types as crate::spa::pod::PodValue>::Value>),*,
            #(#struct_flags_fields_idents: crate::spa::pod::object::PodPropFlags),*,
        }

        #[cfg(feature = "spa-pod-object-builders")]
        impl<'a> #builder_struct_ident<'a> {

            /// Body id, usually [ParamType](pipewire_wrapper::spa::param::ParamType)
            pub fn body_id(mut self, body_id: u32) -> Self {
                self.body_id = body_id;
                self
            }

            #(pub fn #struct_value_fields_idents(mut self, value: <&'a #builder_fields_types as crate::spa::pod::PodValue>::Value) -> Self {
                self.#struct_value_fields_idents = Some(value);
                self
            }
            )*

            #(pub fn #struct_flags_fields_idents(mut self, flags: crate::spa::pod::object::PodPropFlags) -> Self {
                self.#struct_flags_fields_idents = flags;
                self
            }
            )*

            /// Build the allocated [PodObjectRef](crate::spa::pod::object::PodObjectRef).
            pub fn build(self) -> PodResult<crate::spa::pod::pod_buf::AllocPod<crate::spa::pod::object::PodObjectRef>> {
                use crate::spa::pod::FromValue;
                use crate::spa::pod::object::{ObjectPropsIterator, PodPropRef, PodObjectRef, ObjectType};

                let mut props_iter = <ObjectPropsIterator<#enum_ident>>::build();

                #(let #struct_value_fields_idents = self
                    .#struct_value_fields_idents
                    .map(|ref v| <#builder_fields_types as FromValue>::from_value(v))
                    .transpose()?;

                if let Some(#struct_value_fields_idents) = &#struct_value_fields_idents {
                    let prop_value = #enum_ident::#enum_variants_idents(#struct_value_fields_idents.as_pod());
                    let mut prop =
                        <PodPropRef<#enum_ident> as FromValue>::from_value(&prop_value)?;
                    prop.as_pod_mut().set_flags(self.#struct_flags_fields_idents);
                    props_iter = props_iter.push_alloc_pod(prop)?;
                }
                )*

                PodObjectRef::from_id_and_value(
                    self.body_id,
                    &ObjectType::#object_type(props_iter.into_pod_iter().iter()),
                )
            }
        }
    )
}
