use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, TypeReference, TypeSlice};

/// the plbindgen macro is a procedural macro that will generate a C-compatible function
/// and signals to plbindgen that it may produce FFI::Platypus bindings for it.
#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // let _args = parse_macro_input!(attr as AttributeArgs);

    let vis = &input.vis;
    let sig = &input.sig;
    let unsafety = &sig.unsafety;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let block = &input.block;

    // error if vis is not pub
    if !matches!(vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(vis, "exported function must be public")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #[no_mangle]
        pub #unsafety extern "C" fn #fn_name(#inputs) #output {
            #block
        }
    };

    TokenStream::from(expanded)
}

/// the record macro is a procedural macro that will generate a C-compatible struct,
/// and signals to plbindgen that it may produce FFI::Platypus::Record bindings for it.
#[proc_macro_attribute]
pub fn record(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemStruct);

    let vis = &input.vis;
    let ident = &input.ident;
    let fields = &input.fields;
    let semi_token = &input.semi_token;

    if !matches!(vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(vis, "#[record] struct must be public")
            .to_compile_error()
            .into();
    }
    
    if !matches!(fields, syn::Fields::Named(_)) {
        return syn::Error::new_spanned(fields, "#[record] struct must have named fields")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        #[repr(C)]
        pub struct #ident #fields #semi_token
    };

    TokenStream::from(expanded)
}