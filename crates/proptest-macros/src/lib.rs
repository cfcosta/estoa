use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    Expr,
    FnArg,
    ItemFn,
    Lit,
    MetaNameValue,
    Token,
    Type,
    parse_macro_input,
    punctuated::Punctuated,
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
    let attr_args = parse_macro_input!(attr with Punctuated::<MetaNameValue, Token![,]>::parse_terminated);
    let mut config = MacroConfig::default();
    let mut errors: Option<syn::Error> = None;

    for name_value in attr_args {
        if let Err(err) = config.apply(name_value) {
            match &mut errors {
                Some(existing) => existing.combine(err),
                None => errors = Some(err),
            }
        }
    }

    if let Some(err) = errors {
        return err.to_compile_error().into();
    }

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

    let outer_attrs = other_attrs.clone();
    function.attrs = other_attrs;

    let vis = function.vis.clone();
    let original_ident = function.sig.ident.clone();
    let inner_ident = format_ident!("__{}_proptest_impl", original_ident);
    function.sig.ident = inner_ident.clone();
    function.vis = syn::Visibility::Inherited;

    let mut bindings = Vec::new();
    let mut binding_idents = Vec::new();

    for (index, argument) in arguments.iter().enumerate() {
        let binding_ident = format_ident!("__estoa_proptest_binding_{index}");
        binding_idents.push(binding_ident.clone());
        let ty = &argument.ty;

        let binding_stmt = match &argument.strategy {
            Some(expr) => {
                let strategy_ident = format_ident!("__estoa_strategy_{index}");
                quote! {
                    let mut #strategy_ident = ::estoa_proptest::strategy::runtime::adapt(#expr);
                    let #binding_ident: #ty = {
                        let mut __estoa_attempts = 0usize;
                        loop {
                            match ::estoa_proptest::strategy::runtime::execute(
                                &mut #strategy_ident,
                                &mut generator,
                            ) {
                                ::estoa_proptest::strategies::Generation::Accepted { value, .. } => {
                                    generator.advance_iteration();
                                    break value;
                                }
                                ::estoa_proptest::strategies::Generation::Rejected { iteration, depth, .. } => {
                                    generator.advance_iteration();
                                    __estoa_attempts += 1;
                                    if __estoa_attempts >= __ESTOA_REJECTION_LIMIT {
                                        panic!(
                                            "#[proptest] strategy rejected value after {} attempts (iteration {}, depth {}; limit {})",
                                            __estoa_attempts,
                                            iteration,
                                            depth,
                                            __ESTOA_REJECTION_LIMIT,
                                        );
                                    }
                                    continue;
                                }
                            }
                        }
                    };
                }
            }
            None => {
                quote! {
                    let #binding_ident: #ty = {
                        let mut __estoa_attempts = 0usize;
                        loop {
                            match ::estoa_proptest::strategy::runtime::from_arbitrary(&mut generator) {
                                ::estoa_proptest::strategies::Generation::Accepted { value, .. } => {
                                    generator.advance_iteration();
                                    break value;
                                }
                                ::estoa_proptest::strategies::Generation::Rejected { iteration, depth, .. } => {
                                    generator.advance_iteration();
                                    __estoa_attempts += 1;
                                    if __estoa_attempts >= __ESTOA_REJECTION_LIMIT {
                                        panic!(
                                            "#[proptest] strategy rejected value after {} attempts (iteration {}, depth {}; limit {})",
                                            __estoa_attempts,
                                            iteration,
                                            depth,
                                            __ESTOA_REJECTION_LIMIT,
                                        );
                                    }
                                    continue;
                                }
                            }
                        }
                    };
                }
            }
        };

        bindings.push(binding_stmt);
    }

    let outer_rng_setup = if bindings.is_empty() {
        quote! {}
    } else {
        quote! {
            let mut generator = ::estoa_proptest::strategies::Generator::build_with_limit(
                ::estoa_proptest::rng(),
                __ESTOA_RECURSION_LIMIT,
            );
        }
    };

    let cases_tokens = config.cases_tokens();
    let recursion_limit_tokens = config.recursion_limit_tokens();
    let rejection_limit_tokens = config.rejection_limit_tokens();

    let output = quote! {
        #( #doc_attrs )*
        #( #outer_attrs )*
        #[test]
        #vis fn #original_ident() {
            const __ESTOA_CASES: usize = #cases_tokens;
            const __ESTOA_RECURSION_LIMIT: usize = #recursion_limit_tokens;
            const __ESTOA_REJECTION_LIMIT: usize = #rejection_limit_tokens;
            for __estoa_case in 0..__ESTOA_CASES {
                let _ = __estoa_case;
                #outer_rng_setup
                #( #bindings )*
                #inner_ident( #( #binding_idents ),* );
            }
        }

        #function
    };

    output.into()
}

#[derive(Default)]
struct MacroConfig {
    cases: Option<usize>,
    recursion_limit: Option<usize>,
    rejection_limit: Option<usize>,
}

impl MacroConfig {
    fn apply(&mut self, name_value: MetaNameValue) -> syn::Result<()> {
        let ident = name_value.path.get_ident().cloned().ok_or_else(|| {
            syn::Error::new(name_value.path.span(), "expected identifier")
        })?;
        let key = ident.to_string();
        let value = parse_usize(&name_value.value, &key)?;
        if value == 0 {
            return Err(syn::Error::new(
                name_value.value.span(),
                format!("`{}` must be at least 1", key),
            ));
        }

        match key.as_str() {
            "cases" => {
                if self.cases.replace(value).is_some() {
                    Err(syn::Error::new(
                        ident.span(),
                        "`cases` specified more than once",
                    ))
                } else {
                    Ok(())
                }
            }
            "recursion_limit" => {
                if self.recursion_limit.replace(value).is_some() {
                    Err(syn::Error::new(
                        ident.span(),
                        "`recursion_limit` specified more than once",
                    ))
                } else {
                    Ok(())
                }
            }
            "rejection_limit" => {
                if self.rejection_limit.replace(value).is_some() {
                    Err(syn::Error::new(
                        ident.span(),
                        "`rejection_limit` specified more than once",
                    ))
                } else {
                    Ok(())
                }
            }
            _ => Err(syn::Error::new(
                ident.span(),
                format!("unknown #[proptest] option `{}`", key),
            )),
        }
    }

    fn cases_tokens(&self) -> proc_macro2::TokenStream {
        let value = self.cases.unwrap_or(10_000);
        quote! { #value }
    }

    fn recursion_limit_tokens(&self) -> proc_macro2::TokenStream {
        match self.recursion_limit {
            Some(value) => quote! { #value },
            None => quote! { ::core::usize::MAX },
        }
    }

    fn rejection_limit_tokens(&self) -> proc_macro2::TokenStream {
        match self.rejection_limit {
            Some(value) => quote! { #value },
            None => {
                quote! { ::estoa_proptest::strategies::MAX_STRATEGY_ATTEMPTS }
            }
        }
    }
}

fn parse_usize(expr: &Expr, key: &str) -> syn::Result<usize> {
    match expr {
        Expr::Lit(lit) => match &lit.lit {
            Lit::Int(int) => {
                if !int.suffix().is_empty() {
                    return Err(syn::Error::new(
                        int.span(),
                        format!(
                            "`{}` must be an unsuffixed integer literal",
                            key
                        ),
                    ));
                }
                int.base10_parse::<usize>().map_err(|_| {
                    syn::Error::new(
                        int.span(),
                        format!("`{}` is too large to fit in usize", key),
                    )
                })
            }
            _ => Err(syn::Error::new(
                lit.span(),
                format!("`{}` must be an integer literal", key),
            )),
        },
        other => Err(syn::Error::new(
            other.span(),
            format!("`{}` must be an integer literal", key),
        )),
    }
}
