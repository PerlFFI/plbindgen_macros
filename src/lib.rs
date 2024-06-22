use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, Item, ItemFn, ItemStruct, ItemType, PatType, TypePath, TypeReference, TypeSlice};


struct CheckArg {
    ident: syn::Ident,
    fn_arg: syn::FnArg,
}

/// the plbindgen macro is a procedural macro that will generate a C-compatible function
/// and signals to plbindgen that it may produce FFI::Platypus bindings for it.
#[proc_macro_attribute]
pub fn export(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let vis = &input.vis;
    let sig = &input.sig;
    let unsafety = &sig.unsafety;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let block = &input.block;

    if !matches!(vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(vis, "exported function must be public")
            .to_compile_error()
            .into();
    }
    let mut check_arg: Option<CheckArg> = None;
    for input in inputs {
        let syn::FnArg::Typed(PatType { ty, pat, .. }) = input else {
            return syn::Error::new_spanned(input, "expected typed argument")
                .to_compile_error()
                .into();
        };
        let syn::Pat::Ident(syn::PatIdent { ident: name, .. }) = pat.as_ref() else {
            return syn::Error::new_spanned(pat, "expected identifier")
                .to_compile_error()
                .into();
        };

        match check_arg.take()  {
            Some(CheckArg { ref ident, fn_arg }) if ident != name || !is_usize(ty) => {
                return syn::Error::new_spanned(fn_arg, format!("must be followed by {}: usize", ident))
                    .to_compile_error()
                    .into();
            }
            None if is_array(ty) => {
                check_arg.replace(CheckArg {
                    ident: format_ident!("{name}_len"),
                    fn_arg: input.clone(),
                });
            }
            _ => {},
        }
    }
    if let Some(CheckArg { ident, fn_arg }) = check_arg {
        return syn::Error::new_spanned(fn_arg, format!("must be followed by {}: usize", ident))
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

// returns true if is array<T>, where array is our type alias
fn is_array(ty: &syn::Type) -> bool {
    if let syn::Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.first() {
            if segment.ident == "array" {
                return true;
            }
        }
    }
    false
}

fn is_usize(ty: &syn::Type) -> bool {
    if let syn::Type::Path(TypePath { path, .. }) = ty {
        return path.is_ident("usize");
    }
    false
}

/// #[opaque] informs plbindgen that the struct or type alias is an opaque pointer.
/// the name of the struct will be used for the platypus type alias.

#[proc_macro_attribute]
pub fn opaque(attr: TokenStream, input: TokenStream) -> TokenStream {
    let item = parse_macro_input!(input as Item);

    match item {
        Item::Struct(item) => opaque_struct(attr, item),
        Item::Type(item) => opaque_type(attr, item),
        _ => syn::Error::new_spanned(item, "#[opaque] can only be used on structs or type aliases")
            .to_compile_error()
            .into(),
    }
}


fn opaque_struct(_attr: TokenStream, item: ItemStruct) -> TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let fields = &item.fields;
    let semi_token = &item.semi_token;

    if !matches!(vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(vis, "#[opaque] struct must be public")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        pub struct #ident #fields #semi_token
    };

    TokenStream::from(expanded)
}

fn opaque_type(_attr: TokenStream, item: ItemType) -> TokenStream {
    let vis = &item.vis;
    let ident = &item.ident;
    let ty = &item.ty;
    let semi_token = &item.semi_token;

    if !matches!(vis, syn::Visibility::Public(_)) {
        return syn::Error::new_spanned(vis, "#[opaque] type must be public")
            .to_compile_error()
            .into();
    }

    let expanded = quote! {
        pub type #ident = #ty #semi_token
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
