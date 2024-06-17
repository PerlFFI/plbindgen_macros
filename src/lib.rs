use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, FnArg, ItemFn, TypeReference, TypeSlice};

/// the platypus macro is a procedural macro that will generate a C-compatible function,
/// and support semi-automatic conversion of &[T] to *const T and usize.
/// 
/// it is probably most useful in combination with plbindgen.
#[proc_macro_attribute]
pub fn platypus(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // let _args = parse_macro_input!(attr as AttributeArgs);

    let sig = &input.sig;
    let fn_name = &sig.ident;
    let inputs = &sig.inputs;
    let output = &sig.output;
    let block = &input.block;
    let mut defs = vec![];
    let mut new_inputs = Vec::new();

    // Because rust-analyzer does not like adding new secret arguments,
    // we we detect an [T] or &[T] argument, we will require the next argument is a usize
    // with a particular name, and then we will construct the slice ourselves.
    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            let pat = &pat_type.pat;
            let mut ty = &pat_type.ty;
            // C FFI can't do slices, but platypus can do arrays.
            // If we see a name: [T], then let's do name: *const T, name_len: usize
            // and then construct our own slice with the same name
            let mut is_slice = false;
            if let syn::Type::Reference(TypeReference { elem, .. }) = ty.as_ref() {
                if let syn::Type::Slice(TypeSlice { elem, .. }) = elem.as_ref() {
                    ty = elem;
                    is_slice = true;
                }
            }

            if is_slice {
                let ident = match pat.as_ref() {
                    syn::Pat::Ident(ident) => ident.ident.to_string(),
                    _ => panic!("Expected identifier"),
                };
                let pat_len = format_ident!("{}_len", ident);
                new_inputs.push(quote! {
                    #pat: *const #ty
                });
                defs.push(quote! ( 
                    let #pat = unsafe { std::slice::from_raw_parts(#pat, #pat_len) };
                 ));
            } else {
                new_inputs.push(quote! {
                    #pat: #ty
                });
            }
        }
    }

    let expanded = quote! {
        #[no_mangle]
        pub unsafe extern "C" fn #fn_name(#(#new_inputs),*) #output {
            #(#defs)*
            #block
        }
    };

    TokenStream::from(expanded)
}
