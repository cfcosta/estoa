use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr,
    FnArg,
    ItemFn,
    Type,
    parse::Nothing,
    parse_macro_input,
    spanned::Spanned,
};

#[proc_macro_attribute]
/// Duplicate `#[strategy]` annotations on the same argument trigger a compile error.
///
/// ```compile_fail
/// use estoa_proptest_macros::proptest;
///
/// #[proptest]
/// fn duplicate_strategies(
///     #[strategy(|_gen| todo!())]
///     #[strategy(|_gen| todo!())]
///     value: u8,
/// ) {}
/// ```
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

    struct Argument {
        ty: Type,
        strategy: Option<Expr>,
    }

    let mut arguments = Vec::<Argument>::new();

    for input in function.sig.inputs.iter_mut() {
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
                let mut strategy_expr: Option<Expr> = None;
                let mut retained_attrs = Vec::new();

                for attr in pat_type.attrs.drain(..) {
                    if attr.path().is_ident("strategy") {
                        if strategy_expr.is_some() {
                            return syn::Error::new(
                                attr.span(),
                                "#[strategy] cannot be specified more than once per argument",
                            )
                            .to_compile_error()
                            .into();
                        }

                        match attr.parse_args::<Expr>() {
                            Ok(expr) => strategy_expr = Some(expr),
                            Err(err) => return err.to_compile_error().into(),
                        }
                    } else {
                        retained_attrs.push(attr);
                    }
                }

                pat_type.attrs = retained_attrs;

                arguments.push(Argument {
                    ty: (*pat_type.ty).clone(),
                    strategy: strategy_expr,
                });
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

    let uses_strategies = arguments.iter().any(|arg| arg.strategy.is_some());
    let mut bindings = Vec::new();
    let mut binding_idents = Vec::new();

    for (index, argument) in arguments.iter().enumerate() {
        let binding_ident = format_ident!("__estoa_proptest_binding_{index}");
        binding_idents.push(binding_ident.clone());
        let ty = &argument.ty;

        let binding_stmt = match &argument.strategy {
            Some(expr) => {
                quote! {
                    let #binding_ident: #ty = {
                        const __ESTOA_MAX_ATTEMPTS: usize = ::estoa_proptest::strategies::MAX_STRATEGY_ATTEMPTS;
                        let mut __estoa_attempts = 0usize;
                        loop {
                            match (#expr)(&mut generator) {
                                ::estoa_proptest::strategies::Generation::Accepted { value, .. } => break value,
                                ::estoa_proptest::strategies::Generation::Rejected { iteration, depth, .. } => {
                                    __estoa_attempts += 1;
                                    if __estoa_attempts >= __ESTOA_MAX_ATTEMPTS {
                                        panic!(
                                            "strategy rejected value after {} attempts (iteration {}, depth {})",
                                            __estoa_attempts,
                                            iteration,
                                            depth,
                                        );
                                    }
                                    continue;
                                }
                            }
                        }
                    };
                }
            }
            None if uses_strategies => {
                quote! {
                    let #binding_ident: #ty = ::estoa_proptest::arbitrary(&mut generator.rng);
                }
            }
            None => {
                quote! {
                    let #binding_ident: #ty = ::estoa_proptest::arbitrary(&mut rng);
                }
            }
        };

        bindings.push(binding_stmt);
    }

    let outer_rng_setup = if bindings.is_empty() {
        quote! {}
    } else if uses_strategies {
        quote! {
            let mut generator = ::estoa_proptest::strategies::Generator::build(::estoa_proptest::rng());
        }
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
