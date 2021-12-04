use crate::ident_collector::IdentCollector;
use crate::idents::*;
use crate::map_expr::map_expr;
use crate::predicates::{UniquePredicates, UniqueTypeBounds};
use crate::syn_ext::{
    IntoGenericArgument, IntoType, SubsType, WithSpan, WithoutAttrs, WithoutDefault,
    WithoutMaybeBounds,
};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::punctuated::Punctuated;
use syn::token::Add;
use syn::visit::Visit;
use syn::{
    Data, DataEnum, DataStruct, DeriveInput, Fields, GenericArgument, GenericParam, Member,
    TypeParam, TypeParamBound, Variant, WherePredicate,
};

pub fn derive_func_map(input: DeriveInput) -> TokenStream {
    let all_params = &input.generics.params;

    let type_params: Vec<_> = all_params
        .iter()
        .enumerate()
        .filter_map(|(param_idx, param)| match param {
            GenericParam::Type(type_param) => Some((param_idx, type_param)),
            _ => None,
        })
        .collect();

    if type_params.is_empty() {
        abort!(input, "expected at least one type parameter, found none");
    }

    let mut ident_collector = {
        let mut ident_collector = IdentCollector::new_visiting();
        ident_collector.visit_derive_input(&input);
        ident_collector.into_reserved()
    };

    let src_type_ident = ident_collector.reserve_uppercase_letter('A', Span::mixed_site());
    let dst_type_ident = ident_collector.reserve_uppercase_letter('B', Span::mixed_site());
    let fn_type_ident = ident_collector.reserve_uppercase_letter('F', Span::mixed_site());
    let fn_var_ident = Ident::new("f", Span::mixed_site());

    let impls = type_params.into_iter().enumerate().map(
        |(mapped_type_param_idx, (mapped_param_idx, mapped_type_param))| {
            let impl_params = all_params
                .iter()
                .enumerate()
                .flat_map(|(param_idx, param)| {
                    if param_idx == mapped_param_idx {
                        vec![
                            GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &mapped_type_param.bounds,
                                    &mapped_type_param.ident,
                                    &[&src_type_ident],
                                )
                                .without_maybe_bounds(),
                                ..src_type_ident.clone().with_span(Span::call_site()).into()
                            }),
                            GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &mapped_type_param.bounds,
                                    &mapped_type_param.ident,
                                    &[&dst_type_ident],
                                )
                                .without_maybe_bounds(),
                                ..dst_type_ident.clone().with_span(Span::call_site()).into()
                            }),
                        ]
                    } else {
                        vec![match param {
                            GenericParam::Type(type_param) => GenericParam::Type(TypeParam {
                                bounds: subs_type_in_bounds(
                                    &type_param.bounds,
                                    &mapped_type_param.ident,
                                    &[&src_type_ident, &dst_type_ident],
                                ),
                                ..type_param.ident.clone().with_span(Span::call_site()).into()
                            }),
                            GenericParam::Const(const_param) => GenericParam::Const(
                                const_param
                                    .clone()
                                    .with_span(Span::call_site())
                                    .without_attrs()
                                    .without_default(),
                            ),
                            GenericParam::Lifetime(lifetime_param) => GenericParam::Lifetime(
                                lifetime_param
                                    .clone()
                                    .with_span(Span::call_site())
                                    .without_attrs(),
                            ),
                        }]
                    }
                });

            let src_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_param_idx {
                    GenericArgument::Type(
                        src_type_ident
                            .clone()
                            .with_span(Span::call_site())
                            .into_type(),
                    )
                } else {
                    param
                        .clone()
                        .with_span(Span::call_site())
                        .into_generic_argument()
                }
            });

            let dst_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_param_idx {
                    GenericArgument::Type(
                        dst_type_ident
                            .clone()
                            .with_span(Span::call_site())
                            .into_type(),
                    )
                } else {
                    param
                        .clone()
                        .with_span(Span::call_site())
                        .into_generic_argument()
                }
            });

            let mut unique_predicates = UniquePredicates::new();

            for predicate in input
                .generics
                .where_clause
                .iter()
                .flat_map(|clause| clause.predicates.iter())
            {
                let predicate = match predicate
                    .clone()
                    .with_span(Span::call_site())
                    .without_attrs()
                {
                    WherePredicate::Type(predicate_type)
                        if predicate_type.bounded_ty
                            == mapped_type_param.ident.clone().into_type() =>
                    {
                        WherePredicate::Type(predicate_type.without_maybe_bounds())
                    }
                    predicate => predicate,
                };

                unique_predicates.add(
                    predicate
                        .clone()
                        .subs_type(&mapped_type_param.ident, &src_type_ident),
                );

                unique_predicates
                    .add(predicate.subs_type(&mapped_type_param.ident, &dst_type_ident));
            }

            let mut arms = Vec::new();

            for StructLike { ident, fields } in struct_likes(&input) {
                let mut mappings = Vec::new();
                let mut patterns = Vec::new();

                for (field_idx, field) in fields.into_iter().enumerate() {
                    let member: Member = match field.ident {
                        Some(field_ident) => field_ident.with_span(Span::call_site()).into(),
                        None => field_idx.into(),
                    };

                    let ident = format_ident!("field_{}", member, span = Span::mixed_site());
                    let pattern = quote!(#member: #ident);

                    let (mapped, predicates) = map_expr(
                        ident,
                        &field.ty,
                        mapped_type_param,
                        &src_type_ident,
                        &dst_type_ident,
                        &fn_var_ident,
                    );

                    unique_predicates.extend(predicates.into_iter());
                    patterns.push(pattern);
                    mappings.push(quote!(#member: #mapped));
                }

                let (pat_path, output_path) = match ident {
                    Some(ident) => {
                        let ident = ident.clone().with_span(Span::call_site());
                        (
                            quote!(Self::#ident),
                            quote!(Self::#OUTPUT_TYPE_IDENT::#ident),
                        )
                    }
                    None => (quote!(Self), quote!(Self::#OUTPUT_TYPE_IDENT)),
                };

                arms.push(quote! {
                    #pat_path { #(#patterns,)* } => #output_path { #(#mappings,)* }
                });
            }

            let ident = input.ident.clone().with_span(Span::call_site());
            let where_clause = unique_predicates.into_where_clause();

            quote! {
                #[automatically_derived]
                impl<#(#impl_params),*>
                    ::#CRATE_IDENT::#TRAIT_IDENT<
                        #src_type_ident,
                        #dst_type_ident,
                        ::#CRATE_IDENT::#MARKER_TYPE_IDENT<#mapped_type_param_idx>
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
                        #fn_type_ident: FnMut(#src_type_ident) -> #dst_type_ident
                    {
                        match self {
                            #(#arms,)*
                        }
                    }
                }
            }
        },
    );

    quote!(#(#impls)*)
}

fn subs_type_in_bounds<'ast>(
    bounds: impl IntoIterator<Item = &'ast TypeParamBound>,
    ident: &Ident,
    new_idents: &[&Ident],
) -> Punctuated<TypeParamBound, Add> {
    let mut unique_type_bounds = UniqueTypeBounds::new();

    for bound in bounds {
        match bound {
            TypeParamBound::Trait(trait_bound) => {
                for new_ident in new_idents {
                    unique_type_bounds.add(TypeParamBound::Trait(
                        trait_bound
                            .clone()
                            .with_span(Span::call_site())
                            .without_attrs()
                            .subs_type(ident, new_ident),
                    ));
                }
            }
            bound => unique_type_bounds.add(bound.clone().with_span(Span::call_site())),
        };
    }

    unique_type_bounds.into_bounds()
}

struct StructLike {
    ident: Option<Ident>,
    fields: Fields,
}

fn struct_likes(input: &DeriveInput) -> Vec<StructLike> {
    match &input.data {
        Data::Struct(DataStruct { fields, .. }) => vec![StructLike {
            ident: None,
            fields: fields.clone(),
        }],
        Data::Enum(DataEnum { variants, .. }) => variants
            .iter()
            .map(|Variant { ident, fields, .. }| StructLike {
                ident: Some(ident.clone()),
                fields: fields.clone(),
            })
            .collect(),
        Data::Union(..) => abort!(input, "expected a struct or an enum, found a union"),
    }
}
