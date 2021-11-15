use crate::bound_collector::BoundCollector;
use crate::ident_collector::IdentCollector;
use crate::iter;
use crate::struct_mapper::StructMapper;
use crate::syn_ext::{IntoGenericArgument, IntoType, SubsTypeParam, WithIdent, WithoutDefault};
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

    let params = &input.generics.params;

    let type_params: Vec<_> = params
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

    let fields = match data_struct.fields {
        Fields::Named(fields) => fields.named,
        Fields::Unnamed(fields) => fields.unnamed,
        Fields::Unit => abort!(
            data_struct.fields,
            "expected a struct with fields, found a unit struct"
        ),
    };

    let type_a = ident_collector.reserve_uppercase_letter('A');
    let type_b = ident_collector.reserve_uppercase_letter('B');
    let type_f = ident_collector.reserve_uppercase_letter('F');
    let var_f = Ident::new("f", Span::mixed_site());

    let ident = input.ident;

    let impls =
        type_params
            .into_iter()
            .enumerate()
            .map(|(type_param_idx, (param_idx, type_param))| {
                let mut struct_mapper = StructMapper::new(type_param, &type_a, &type_b, &var_f);

                let mappings: Vec<_> = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let member: Member = match &field.ident {
                            Some(ident) => ident.clone().into(),
                            None => idx.into(),
                        };

                        let mappable = quote_spanned!(Span::mixed_site() => self.#member);
                        let mapped = struct_mapper.map_struct(mappable, &field.ty);

                        quote_spanned!(Span::mixed_site() => #member: #mapped)
                    })
                    .collect();

                let type_a = type_a.clone();
                let type_b = type_b.clone();

                let type_param_a = type_param.clone().with_ident(type_a.clone()).without_default();
                let type_param_b = type_param.clone().with_ident(type_b.clone()).without_default();

                let impl_params = iter::replace_at(params.iter().map(|param| {
                    match param.clone().without_default() {
                        GenericParam::Type(impl_type_param) => {
                            let mut collector = BoundCollector::new();

                            for bound in impl_type_param.bounds {
                                match bound {
                                    TypeParamBound::Trait(trait_bound) => {
                                        collector.insert(TypeParamBound::Trait(trait_bound.clone().subs_type_param(type_param, &type_a)));
                                        collector.insert(TypeParamBound::Trait(trait_bound.subs_type_param(type_param, &type_b)));
                                    },
                                    bound => collector.insert(bound.clone()),
                                };
                            }

                            GenericParam::Type(TypeParam {
                                bounds: collector.into_bounds(),
                                ..impl_type_param
                            })
                        },
                        param => param,
                    }
                }), param_idx, [{
                    let mut collector = BoundCollector::new();
                    let param = type_param_a.clone();

                    for bound in type_param_a.bounds {
                        match bound {
                            TypeParamBound::Trait(trait_bound) => {
                                collector.insert(TypeParamBound::Trait(trait_bound.subs_type_param(type_param, &param.ident)));
                            },
                            bound => collector.insert(bound.clone()),
                        };
                    }

                    GenericParam::Type(TypeParam {
                        bounds: collector.into_bounds(),
                        ..param
                    })
                }, {
                    let mut collector = BoundCollector::new();
                    let param = type_param_b.clone();

                    for bound in type_param_b.bounds {
                        match bound {
                            TypeParamBound::Trait(trait_bound) => {
                                collector.insert(TypeParamBound::Trait(trait_bound.subs_type_param(type_param, &param.ident)));
                            },
                            bound => collector.insert(bound.clone()),
                        };
                    }

                    GenericParam::Type(TypeParam {
                        bounds: collector.into_bounds(),
                        ..param
                    })
                }]);

                let src_args = iter::replace_at(
                    params.iter().cloned().map(IntoGenericArgument::into_generic_argument),
                    param_idx,
                    [GenericArgument::Type(type_a.clone().into_type())],
                );
                let dst_args = iter::replace_at(
                    params.iter().cloned().map(IntoGenericArgument::into_generic_argument),
                    param_idx,
                    [GenericArgument::Type(type_b.clone().into_type())],
                );
                let where_clause = struct_mapper.where_clause();

                quote_spanned! { Span::mixed_site() =>
                    #[automatically_derived]
                    impl<#(#impl_params),*>
                        ::mapstruct::MapStruct<#type_a, #type_b, ::mapstruct::TypeParam<#type_param_idx>>
                        for #ident<#(#src_args),*>
                        #where_clause
                    {
                        type Output = #ident<#(#dst_args),*>;

                        fn map_struct<#type_f>(self, mut #var_f: #type_f) -> Self::Output
                        where
                            #type_f: FnMut(#type_a) -> #type_b
                        {
                            Self::Output {
                                #(#mappings,)*
                            }
                        }
                    }
                }
            });

    quote_spanned!(Span::mixed_site() => #(#impls)*)
}
