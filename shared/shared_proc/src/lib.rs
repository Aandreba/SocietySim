use proc_macro2::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn cfg_cpu(
    attrs: proc_macro::TokenStream,
    items: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attrs = TokenStream::from(attrs);
    let items = TokenStream::from(items);

    if attrs.is_empty() {
        return quote! {
            #[cfg(not(target_arch = "spirv"))]
            #items
        }
        .into();
    } else {
        return quote! {
            #[cfg_attr(not(target_arch = "spirv"), #attrs)]
            #items
        }
        .into();
    }
}