use crate::ident_collector::IdentCollector;
use crate::map_expr::map_expr;
use crate::predicates::{UniquePredicates, UniqueTypeBounds};
use crate::syn_ext::{IntoGenericArgument, IntoType, SubsType, WithIdent, WithoutDefault};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote_spanned;
use syn::visit::Visit;
use syn::{Data, DeriveInput, Fields};
use syn::{GenericArgument, GenericParam, Member, TypeParam, TypeParamBound};

pub fn derive_map_struct(input: DeriveInput) -> TokenStream {
    let mut ident_collector = IdentCollector::new_visiting();
    ident_collector.visit_derive_input(&input);
    let mut ident_collector = ident_collector.into_reserved();

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

    let data_struct = match input.data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(..) => abort!(input, "expected a struct, found an enum"),
        Data::Union(..) => abort!(input, "expected a struct, found a union"),
    };

    
    // TODO support Fields::Unit and use `match` syntax to make logic reusable for enums

    let fields = match data_struct.fields {
        Fields::Named(fields) => fields.named,
        Fields::Unnamed(fields) => fields.unnamed,
        Fields::Unit => abort!(
            data_struct.fields,
            "expected a struct with fields, found a unit struct"
        ),
    };

    let src_type_ident = ident_collector.reserve_uppercase_letter('A');
    let dst_type_ident = ident_collector.reserve_uppercase_letter('B');
    let fn_type_ident = ident_collector.reserve_uppercase_letter('F');
    let fn_var_ident = Ident::new("f", Span::mixed_site());

    let impls = type_params.into_iter().enumerate().map(
        |(mapped_type_param_idx, (mapped_param_idx, mapped_type_param))| {
            let impl_params = all_params
                .iter()
                .enumerate()
                .flat_map(|(param_idx, param)| {
                    if param_idx == mapped_param_idx {
                        vec![
                            GenericParam::Type(subs_type_in_bounds(
                                mapped_type_param
                                    .clone()
                                    .with_ident(src_type_ident.clone())
                                    .without_default(),
                                &mapped_type_param.ident,
                                &[&src_type_ident],
                            )),
                            GenericParam::Type(subs_type_in_bounds(
                                mapped_type_param
                                    .clone()
                                    .with_ident(dst_type_ident.clone())
                                    .without_default(),
                                &mapped_type_param.ident,
                                &[&dst_type_ident],
                            )),
                        ]
                    } else {
                        vec![match param {
                            GenericParam::Type(type_param) => {
                                GenericParam::Type(subs_type_in_bounds(
                                    type_param.clone().without_default(),
                                    &mapped_type_param.ident,
                                    &[&src_type_ident, &dst_type_ident],
                                ))
                            }
                            GenericParam::Const(const_param) => {
                                GenericParam::Const(const_param.clone().without_default())
                            }
                            param => param.clone(),
                        }]
                    }
                });

            let src_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_param_idx {
                    GenericArgument::Type(src_type_ident.clone().into_type())
                } else {
                    param.clone().into_generic_argument()
                }
            });

            let dst_args = all_params.iter().enumerate().map(|(param_idx, param)| {
                if param_idx == mapped_param_idx {
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
                unique_predicates.add(
                    predicate
                        .clone()
                        .subs_type(&mapped_type_param.ident, &src_type_ident),
                );

                unique_predicates.add(
                    predicate
                        .clone()
                        .subs_type(&mapped_type_param.ident, &dst_type_ident),
                );
            }

            let mut mappings = Vec::new();

            for (field_idx, field) in fields.iter().enumerate() {
                let member: Member = match &field.ident {
                    Some(field_ident) => field_ident.clone().into(),
                    None => field_idx.into(),
                };
                let mappable = quote_spanned!(Span::mixed_site() => self.#member);

                let (mapped, predicates) = map_expr(
                    mappable,
                    &field.ty,
                    mapped_type_param,
                    &src_type_ident,
                    &dst_type_ident,
                    &fn_var_ident,
                );

                unique_predicates.extend(predicates.into_iter());
                mappings.push(quote_spanned!(Span::mixed_site() => #member: #mapped));
            }

            let ident = &input.ident;
            let where_clause = unique_predicates.into_where_clause();

            quote_spanned! { Span::mixed_site() =>
                #[automatically_derived]
                impl<#(#impl_params),*>
                    ::mapstruct::MapStruct<
                        #src_type_ident,
                        #dst_type_ident,
                        ::mapstruct::TypeParam<#mapped_type_param_idx>
                    >
                    for #ident<#(#src_args),*>
                    #where_clause
                {
                    type Output = #ident<#(#dst_args),*>;

                    fn map_struct<#fn_type_ident>(
                        self,
                        mut #fn_var_ident: #fn_type_ident
                    ) -> Self::Output
                    where
                        #fn_type_ident: FnMut(#src_type_ident) -> #dst_type_ident
                    {
                        Self::Output {
                            #(#mappings,)*
                        }
                    }
                }
            }
        },
    );

    quote_spanned!(Span::mixed_site() => #(#impls)*)
}

fn subs_type_in_bounds(type_param: TypeParam, ident: &Ident, new_idents: &[&Ident]) -> TypeParam {
    let mut unique_type_bounds = UniqueTypeBounds::new();

    for bound in type_param.bounds {
        match bound {
            TypeParamBound::Trait(trait_bound) => {
                for new_ident in new_idents {
                    unique_type_bounds.add(TypeParamBound::Trait(
                        trait_bound.clone().subs_type(ident, new_ident),
                    ));
                }
            }
            bound => unique_type_bounds.add(bound.clone()),
        };
    }

    TypeParam {
        bounds: unique_type_bounds.into_bounds(),
        ..type_param
    }
}
