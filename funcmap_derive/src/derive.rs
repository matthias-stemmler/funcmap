use crate::derivable::Derivable;
use crate::ident::{
    FALLIBLE_FN_IDENT, FALLIBLE_TRAIT_IDENT, FN_IDENT, MARKER_TYPE_IDENT, OUTPUT_TYPE_IDENT,
    TRAIT_IDENT,
};
use crate::input::{FuncMapInput, Structish};
use crate::map::Mapping;
use crate::predicates::{UniquePredicates, UniqueTypeBounds};
use crate::result::{self, Error, ResultExt};
use crate::syn_ext::{
    IntoGenericArgument, IntoType, SubsType, WithoutAttrs, WithoutDefault, WithoutMaybeBounds,
};

use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::{
    DeriveInput, GenericArgument, GenericParam, Member, Token, TypeParam, TypeParamBound,
    WherePredicate,
};

pub(crate) fn derive(item: TokenStream, derivable: Derivable) -> TokenStream {
    match try_derive(item, derivable) {
        Ok(output) => output,
        Err(err) => err.into_compile_error(),
    }
}

pub(crate) fn try_derive(item: TokenStream, derivable: Derivable) -> Result<TokenStream, Error> {
    let input: DeriveInput = syn::parse2(item)?;
    let input: FuncMapInput = input.try_into()?;
    let mut ident_collector = input.meta.ident_collector;

    let src_type_ident = ident_collector.reserve_uppercase_letter('A', Span::mixed_site());
    let dst_type_ident = ident_collector.reserve_uppercase_letter('B', Span::mixed_site());
    let fn_type_ident = ident_collector.reserve_uppercase_letter('F', Span::mixed_site());
    let err_type_ident = ident_collector.reserve_uppercase_letter('E', Span::mixed_site());
    let fn_var_ident = Ident::new("f", Span::mixed_site());

    let all_params = &input.generics.params;

    let mut result_builder = result::Builder::new();

    let impls = input
        .mapped_type_params
        .into_iter()
        .map(|mapped_type_param| {
            let impl_params = all_params
                .iter()
                .enumerate()
                .flat_map(|(param_idx, param)| {
                    if param_idx == mapped_type_param.param_idx {
                        vec![
                            GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &mapped_type_param.type_param.bounds,
                                    &mapped_type_param.type_param.ident,
                                    &[&src_type_ident],
                                )
                                .without_maybe_bounds(),
                                ..src_type_ident.clone().into()
                            }),
                            GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &mapped_type_param.type_param.bounds,
                                    &mapped_type_param.type_param.ident,
                                    &[&dst_type_ident],
                                )
                                .without_maybe_bounds(),
                                ..dst_type_ident.clone().into()
                            }),
                        ]
                    } else {
                        vec![match param {
                            GenericParam::Type(type_param) => GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &type_param.bounds,
                                    &mapped_type_param.type_param.ident,
                                    &[&src_type_ident, &dst_type_ident],
                                ),
                                ..type_param.ident.clone().into()
                            }),
                            GenericParam::Const(const_param) => GenericParam::Const(
                                const_param.clone().without_attrs().without_default(),
                            ),
                            GenericParam::Lifetime(lifetime_param) => {
                                GenericParam::Lifetime(lifetime_param.clone().without_attrs())
                            }
                        }]
                    }
                });

            let src_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_type_param.param_idx {
                    GenericArgument::Type(src_type_ident.clone().into_type())
                } else {
                    param.clone().into_generic_argument()
                }
            });

            let dst_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_type_param.param_idx {
                    GenericArgument::Type(dst_type_ident.clone().into_type())
                } else {
                    param.clone().into_generic_argument()
                }
            });

            let mut unique_predicates = UniquePredicates::new();

            for predicate in input
                .generics
                .where_clause
                .iter()
                .flat_map(|clause| clause.predicates.iter())
            {
                let predicate = match predicate.clone().without_attrs() {
                    WherePredicate::Type(predicate_type)
                        if predicate_type.bounded_ty
                            == mapped_type_param.type_param.ident.clone().into_type() =>
                    {
                        WherePredicate::Type(predicate_type.without_maybe_bounds())
                    }
                    predicate => predicate,
                };

                unique_predicates
                    .add(
                        predicate
                            .clone()
                            .subs_type(&mapped_type_param.type_param.ident, &src_type_ident),
                    )
                    .add_err_to(&mut result_builder);

                unique_predicates
                    .add(predicate.subs_type(&mapped_type_param.type_param.ident, &dst_type_ident))
                    .add_err_to(&mut result_builder);
            }

            let mut arms = Vec::new();

            for Structish {
                variant_ident,
                fields,
            } in &input.variants
            {
                let mut mappings = Vec::new();
                let mut patterns = Vec::new();

                for (field_idx, field) in fields.iter().enumerate() {
                    let (member, ident) = match &field.ident {
                        Some(field_ident) => {
                            let member: Member = field_ident.clone().into();
                            let ident = format_ident!(
                                "field_{}",
                                field_ident.clone(),
                                span = Span::mixed_site()
                            );
                            (member, ident)
                        }
                        None => {
                            let member: Member = field_idx.into();
                            let ident =
                                format_ident!("field_{}", field_idx, span = Span::mixed_site());
                            (member, ident)
                        }
                    };

                    let pattern = quote!(#member: #ident);

                    let mapping = Mapping {
                        type_param: &mapped_type_param.type_param,
                        src_type_ident: &src_type_ident,
                        dst_type_ident: &dst_type_ident,
                        fn_ident: &fn_var_ident,
                        crate_path: &input.meta.crate_path,
                        derivable,
                    };

                    if let Some((mapped, predicates)) = mapping
                        .map(ident, &field.ty)
                        .add_err_to(&mut result_builder)
                    {
                        for predicate in predicates.into_iter() {
                            unique_predicates
                                .add(predicate)
                                .add_err_to(&mut result_builder);
                        }

                        patterns.push(pattern);
                        mappings.push(quote!(#member: #mapped));
                    }
                }

                let (pat_path, output_path) = match variant_ident {
                    Some(ident) => (
                        quote!(Self::#ident),
                        quote!(Self::#OUTPUT_TYPE_IDENT::#ident),
                    ),
                    None => (quote!(Self), quote!(Self::#OUTPUT_TYPE_IDENT)),
                };

                arms.push(quote! {
                    #pat_path { #(#patterns,)* } => #output_path { #(#mappings,)* }
                });
            }

            let crate_path = &input.meta.crate_path;
            let ident = &input.ident;
            let where_clause = unique_predicates.into_where_clause();
            let type_param_idx = mapped_type_param.type_param_idx;

            let attrs = quote! {
                #[allow(absolute_paths_not_starting_with_crate)]
                #[allow(bare_trait_objects)]
                #[allow(deprecated)]
                #[allow(drop_bounds)]
                #[allow(dyn_drop)]
                #[allow(keyword_idents)]
                #[allow(non_camel_case_types)]
                #[allow(trivial_bounds)]
                #[allow(unused_qualifications)]
                #[allow(clippy::disallowed_method)]
                #[allow(clippy::disallowed_type)]
                #[automatically_derived]
            };

            match derivable {
                Derivable::Standard => quote! {
                    #attrs
                    impl<#(#impl_params),*>
                        #crate_path::#TRAIT_IDENT<
                            #src_type_ident,
                            #dst_type_ident,
                            #crate_path::#MARKER_TYPE_IDENT<#type_param_idx>
                        >
                        for #ident<#(#src_args),*>
                        #where_clause
                    {
                        type #OUTPUT_TYPE_IDENT = #ident<#(#dst_args),*>;

                        fn #FN_IDENT<#fn_type_ident>(
                            self,
                            mut #fn_var_ident: #fn_type_ident
                        ) -> Self::#OUTPUT_TYPE_IDENT
                        where
                            #fn_type_ident: ::core::ops::FnMut(#src_type_ident) -> #dst_type_ident
                        {
                            match self {
                                #(#arms,)*
                            }
                        }
                    }
                },
                Derivable::Fallible => quote! {
                    #attrs
                    impl<#(#impl_params),*>
                        #crate_path::#FALLIBLE_TRAIT_IDENT<
                            #src_type_ident,
                            #dst_type_ident,
                            #crate_path::#MARKER_TYPE_IDENT<#type_param_idx>
                        >
                        for #ident<#(#src_args),*>
                        #where_clause
                    {
                        fn #FALLIBLE_FN_IDENT<#err_type_ident, #fn_type_ident>(
                            self,
                            mut #fn_var_ident: #fn_type_ident
                        ) -> ::core::result::Result<Self::#OUTPUT_TYPE_IDENT, #err_type_ident>
                        where
                            #fn_type_ident:
                                ::core::ops::FnMut(
                                    #src_type_ident
                                ) -> ::core::result::Result<#dst_type_ident, #err_type_ident>
                        {
                            ::core::result::Result::Ok(match self {
                                #(#arms,)*
                            })
                        }
                    }
                },
            }
        })
        .collect::<Vec<_>>();

    result_builder.err_or(quote! {
        #(#impls)*
    })
}

fn subs_type_in_bounds<'ast>(
    bounds: impl IntoIterator<Item = &'ast TypeParamBound>,
    ident: &Ident,
    new_idents: &[&Ident],
) -> Punctuated<TypeParamBound, Token![+]> {
    let mut unique_type_bounds = UniqueTypeBounds::new();

    for bound in bounds {
        match bound {
            TypeParamBound::Trait(trait_bound) => {
                for new_ident in new_idents {
                    unique_type_bounds.add(TypeParamBound::Trait(
                        trait_bound
                            .clone()
                            .without_attrs()
                            .subs_type(ident, new_ident),
                    ));
                }
            }
            bound @ TypeParamBound::Lifetime(..) => unique_type_bounds.add(bound.clone()),
        };
    }

    unique_type_bounds.into_bounds()
}
