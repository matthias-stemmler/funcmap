use crate::ident_collector::IdentCollector;
use crate::iter;
use crate::struct_mapper::StructMapper;
use crate::type_ext::{IdentExt, TypeParamExt, WithoutDefaultExt};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::quote_spanned;
use syn::visit::Visit;
use syn::{ConstParam, GenericArgument, GenericParam, LifetimeDef, Member};
use syn::{Data, DeriveInput, Fields};

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

                let type_param_a = type_param.clone().with_ident(type_a.clone()).without_default().into();
                let type_param_b = type_param.clone().with_ident(type_b.clone()).without_default().into();

                let impl_params = iter::replace_at(params.iter().map(|param| param.clone().without_default()), param_idx, [type_param_a, type_param_b]);

                let src_args = iter::replace_at(
                    params.iter().cloned().map(param_into_argument),
                    param_idx,
                    [GenericArgument::Type(type_a.clone().into_type())],
                );
                let dst_args = iter::replace_at(
                    params.iter().cloned().map(param_into_argument),
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

fn param_into_argument(param: GenericParam) -> GenericArgument {
    match param {
        GenericParam::Type(type_param) => GenericArgument::Type(type_param.into_type()),
        GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => GenericArgument::Lifetime(lifetime),
        GenericParam::Const(ConstParam { ident, .. }) => GenericArgument::Type(ident.into_type()),
    }
}
