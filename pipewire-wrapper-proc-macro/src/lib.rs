/*
 * SPDX-License-Identifier: MIT
 */
use proc_macro::TokenStream;

/// Implement [RawWrapper](crate::wrapper::RawWrapper) trait.
/// `#[raw]` attribute must be added before field with wrapped raw value.
///
/// # Examples
///
/// ```no_run,ignore
/// #[derive(RawWrapper, Debug)]
/// #[repr(transparent)]
/// pub struct MainLoopRef {
///     #[raw]
///     raw: pw_sys::pw_main_loop,
/// }
/// ```
#[proc_macro_derive(RawWrapper, attributes(raw))]
pub fn raw_wrapper(input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::derive_raw_wrapper(input.into()).into()
}

/// Implement [Wrapper](crate::wrapper::Wrapper) trait.
/// `#[raw_wrapper]` attribute must be added before field with `NonNull<impl RawWrapper>` pointer to *Ref
/// struct, that implements [RawWrapper](crate::wrapper::RawWrapper)
///
/// # Examples
///
/// ```no_run,ignore
/// #[derive(RawWrapper, Debug)]
/// #[repr(transparent)]
/// pub struct MainLoopRef {
///     #[raw]
///     raw: pw_sys::pw_main_loop,
/// }
///
/// #[derive(Wrapper, Debug)]
/// pub struct MainLoop {
///     #[raw_wrapper]
///     ref_: NonNull<MainLoopRef>,
///
///     pipewire: std::sync::Arc<PipeWire>,
/// }
/// ```
#[proc_macro_derive(Wrapper, attributes(raw_wrapper))]
pub fn wrapper(input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::derive_wrapper(input.into()).into()
}

/// Implement [SpaInterface](crate::wrapper::SpaInterface)
///
/// # Arguments
///
/// * `methods` - struct with interface methods
///
/// ```no_run,ignore
/// #[derive(RawWrapper, Debug)]
/// #[spa_interface(methods=spa_sys::spa_loop_methods)]
/// #[repr(transparent)]
/// pub struct LoopRef {
///     #[raw]
///     raw: spa_sys::spa_loop,
/// }
/// ```
#[proc_macro_attribute]
pub fn spa_interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::spa_interface(attr.into(), input.into()).into()
}

/// Implement [SpaInterface](crate::wrapper::SpaInterface) and
/// [Proxied](crate::core_api::proxy::Proxied) traits.
///
/// # Arguments
///
/// * `methods` - struct with interface methods
/// * `interface` - interface name
///
/// # Examples
///
/// ```no_run,ignore
/// #[derive(RawWrapper, Debug)]
/// #[interface(methods=pw_sys::pw_node_methods, interface="Node")]
/// #[repr(transparent)]
/// pub struct NodeRef {
///     #[raw]
///     raw: pw_sys::pw_node,
/// }
/// ```
#[proc_macro_attribute]
pub fn interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::interface(attr.into(), input.into()).into()
}

/// Implement [Wrapper](crate::wrapper::Wrapper) trait for structure with the `ref_: Proxy<'c>` field.
/// Macros parameter will be used as target type to cast underlying proxy.
///
/// # Examples
///
/// ```no_run,ignore
/// #[derive(RawWrapper, Debug)]
/// #[interface(methods=pw_sys::pw_node_methods, interface="Node")]
/// #[repr(transparent)]
/// pub struct NodeRef {
///     #[raw]
///     raw: pw_sys::pw_node,
/// }
///
/// #[derive(Clone, Debug)]
/// #[proxy_wrapper(NodeRef)]
/// pub struct Node<'c> {
///     ref_: Proxy<'c>,
///
///     listeners: Listeners<Pin<Box<NodeEvents<'c>>>>,
/// }
/// ```
#[proc_macro_attribute]
pub fn proxy_wrapper(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::proxy_wrapper(attr.into(), input.into()).into()
}

/// Add an *Info structure after the enum definition.
/// For each enum variant will be added optional struct fields with value and flags.
///
/// # Examples
///
/// Derive:
/// ```no_run,ignore
/// #[allow(non_camel_case_types)]
/// #[derive(Debug)]
/// #[repr(u32)]
/// #[object_info(OBJECT_PROPS)]
/// pub enum ObjectPropType<'a> {
///     // Device
///     DEVICE(&'a PodStringRef) = Prop::DEVICE.raw,
///     DEVICE_NAME(&'a PodStringRef) = Prop::DEVICE_NAME.raw,
///
/// }
/// ```
///
/// Usage:
/// ```no_run,ignore
/// if let Ok(BasicType::OBJECT(object)) = param.downcast() {
///     match object.body_type() {
///         Type::OBJECT_PROP_INFO => {
///             let info = ObjectPropInfoInfo::try_from(object).unwrap();
///             println!("Prop info: {:?}", info);
///         }
///         _ => todo!(),
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn object_info(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_wrapper_macro_impl::derive_object_info(attr.into(), input.into()).into()
}
