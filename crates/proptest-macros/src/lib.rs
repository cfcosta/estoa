use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    FnArg,
    ItemFn,
    Type,
    parse::Nothing,
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_attribute]
pub fn proptest(attr: TokenStream, item: TokenStream) -> TokenStream {
    parse_macro_input!(attr as Nothing);
    let mut function = parse_macro_input!(item as ItemFn);

    if let Some(async_token) = &function.sig.asyncness {
        return syn::Error::new(
            async_token.span(),
            "#[proptest] does not support async functions",
        )
        .to_compile_error()
        .into();
    }

    if let Some(const_token) = &function.sig.constness {
        return syn::Error::new(
            const_token.span(),
            "#[proptest] does not support const functions",
        )
        .to_compile_error()
        .into();
    }

    if let Some(abi) = &function.sig.abi {
        return syn::Error::new(
            abi.extern_token.span(),
            "#[proptest] does not support extern functions",
        )
        .to_compile_error()
        .into();
    }

    if !function.sig.generics.params.is_empty() {
        return syn::Error::new(
            function.sig.generics.span(),
            "#[proptest] does not support generic functions",
        )
        .to_compile_error()
        .into();
    }

    let inputs = function.sig.inputs.clone();

    let mut argument_types = Vec::<Type>::new();

    for input in inputs.iter() {
        match input {
            FnArg::Receiver(receiver) => {
                return syn::Error::new(
                    receiver.span(),
                    "#[proptest] cannot be applied to methods",
                )
                .to_compile_error()
                .into();
            }
            FnArg::Typed(pat_type) => {
                argument_types.push((*pat_type.ty).clone());
            }
        }
    }

    let mut doc_attrs = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in function.attrs.drain(..) {
        if attr.path().is_ident("doc") {
            doc_attrs.push(attr);
        } else {
            other_attrs.push(attr);
        }
    }

    function.attrs = other_attrs;

    let vis = function.vis.clone();
    let original_ident = function.sig.ident.clone();
    let inner_ident = format_ident!("__{}_proptest_impl", original_ident);
    function.sig.ident = inner_ident.clone();
    function.vis = syn::Visibility::Inherited;

    let mut bindings = Vec::new();
    let mut binding_idents = Vec::new();

    for (index, ty) in argument_types.iter().enumerate() {
        let binding_ident = format_ident!("__estoa_proptest_binding_{index}");
        binding_idents.push(binding_ident.clone());
        bindings.push(quote! {
            let #binding_ident: #ty = ::estoa_proptest::arbitrary(&mut rng);
        });
    }

    let outer_rng_setup = if bindings.is_empty() {
        quote! {}
    } else {
        quote! {
            let mut rng = ::estoa_proptest::rng();
        }
    };

    let output = quote! {
        #( #doc_attrs )*
        #[test]
        #vis fn #original_ident() {
            #outer_rng_setup
            #( #bindings )*
            #inner_ident( #( #binding_idents ),* );
        }

        #function
    };

    output.into()
}
