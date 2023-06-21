use proc_macro::TokenStream;

#[proc_macro_derive(RawWrapper, attributes(raw))]
pub fn raw_wrapper(input: TokenStream) -> TokenStream {
    pipewire_macro_impl::derive_raw_wrapper(input.into()).into()
}

#[proc_macro_derive(Wrapper, attributes(raw_wrapper))]
pub fn wrapper(input: TokenStream) -> TokenStream {
    pipewire_macro_impl::derive_wrapper(input.into()).into()
}

#[proc_macro_attribute]
pub fn spa_interface(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_macro_impl::spa_interface(attr.into(), input.into()).into()
}

#[proc_macro_attribute]
pub fn proxied(attr: TokenStream, input: TokenStream) -> TokenStream {
    pipewire_macro_impl::proxied(attr.into(), input.into()).into()
}
