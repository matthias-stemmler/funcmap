use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::Visit;
use syn::{parse_quote, ConstParam, GenericArgument, GenericParam, LifetimeDef, Member, TypeParam};
use syn::{spanned::Spanned, Data, DeriveInput, Fields};

use crate::ident_collector::IdentCollector;
use crate::iter::replace_at;
use crate::macros::fail;
use crate::struct_mapper::StructMapper;

pub fn derive_map_struct(input: DeriveInput) -> TokenStream {
    let mut ident_collector = IdentCollector::new();
    ident_collector.visit_derive_input(&input);

    let input_span = input.span();
    let DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    let type_params: Vec<_> = generics
        .params
        .iter()
        .enumerate()
        .filter_map(|(param_idx, param)| match param {
            GenericParam::Type(type_param) => Some((param_idx, type_param)),
            _ => None,
        })
        .collect();

    if type_params.is_empty() {
        fail!(generics, "expected at least one type parameter, found none");
    }

    let data_struct = match data {
        Data::Struct(data_struct) => data_struct,
        Data::Enum(..) => fail!(input_span, "expected a struct, found an enum"),
        Data::Union(..) => fail!(input_span, "expected a struct, found a union"),
    };

    let fields = match data_struct.fields {
        Fields::Named(fields) => fields.named,
        Fields::Unnamed(fields) => fields.unnamed,
        Fields::Unit => fail!(
            data_struct.fields,
            "expected a struct with fields, found a unit struct"
        ),
    };

    let a = ident_collector.reserve_uppercase_letter('A');
    let b = ident_collector.reserve_uppercase_letter('B');
    let f: TypeParam = ident_collector.reserve_uppercase_letter('F');

    let impls =
        type_params
            .into_iter()
            .enumerate()
            .map(|(type_param_idx, (param_idx, type_param))| {
                let mut struct_mapper = StructMapper::new(&a, &b);

                let mappings: Vec<_> = fields
                    .iter()
                    .enumerate()
                    .map(|(idx, field)| {
                        let member: Member = match &field.ident {
                            Some(ident) => ident.clone().into(),
                            None => idx.into(),
                        };

                        let mappable = quote!(self.#member);
                        let mapped = struct_mapper.map_struct(mappable, &field.ty, type_param);

                        quote! {
                            #member: #mapped
                        }
                    })
                    .collect();

                let a: GenericParam = a.clone().into();
                let b: GenericParam = b.clone().into();
                let impl_params = replace_at(generics.params.iter(), param_idx, [&a, &b]);

                let src_params = replace_at(
                    generics.params.iter().map(param_to_argument),
                    param_idx,
                    Some(parse_quote!(#a)),
                );
                let dst_params = replace_at(
                    generics.params.iter().map(param_to_argument),
                    param_idx,
                    Some(parse_quote!(#b)),
                );
                let where_clause = struct_mapper.where_clause();

                quote! {
                    impl<#(#impl_params),*>
                        ::mapstruct::MapStruct<#a, #b, ::mapstruct::TypeParam<#type_param_idx>>
                        for #ident<#(#src_params),*>
                        #where_clause
                    {
                        type Output = #ident<#(#dst_params),*>;

                        fn map_struct<#f>(self, mut f: #f) -> Self::Output
                        where
                            #f: FnMut(#a) -> #b
                        {
                            Self::Output {
                                #(#mappings,)*
                            }
                        }
                    }
                }
            });

    quote! {
        #(#impls)*
    }
}

fn param_to_argument(param: &GenericParam) -> GenericArgument {
    match param {
        GenericParam::Type(TypeParam { ident, .. }) => parse_quote!(#ident),
        GenericParam::Lifetime(LifetimeDef { lifetime, .. }) => parse_quote!(#lifetime),
        GenericParam::Const(ConstParam { ident, .. }) => parse_quote!(#ident),
    }
}
