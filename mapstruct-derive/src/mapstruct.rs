use proc_macro2::TokenStream;
use quote::quote;
use syn::visit::Visit;
use syn::Member;
use syn::{spanned::Spanned, Data, DeriveInput, Fields};

use crate::ident_collector::IdentCollector;
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

    let type_param = {
        let type_params: Vec<_> = generics.type_params().collect();

        match type_params.len() {
            1 => type_params[0],
            n => fail!(generics, "expected exactly one type parameter, found {}", n),
        }
    };

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

    let mut struct_mapper = StructMapper::new(&a, &b);

    let mappings: Vec<_> = fields
        .into_iter()
        .enumerate()
        .map(|(idx, field)| {
            let member: Member = match field.ident {
                Some(ident) => ident.into(),
                None => idx.into(),
            };

            let mappable = quote!(self.#member);
            let mapped = struct_mapper.map_struct(mappable, &field.ty, type_param);

            quote! {
                #member: #mapped
            }
        })
        .collect();

    let where_clause = struct_mapper.where_clause();

    quote! {
        impl<#a, #b> ::mapstruct::MapStruct<#a, #b> for #ident<#a> #where_clause {
            type Output = #ident<#b>;

            fn map_struct<F>(self, mut f: F) -> Self::Output
            where
                F: FnMut(#a) -> #b
            {
                Self::Output {
                    #(#mappings,)*
                }
            }
        }
    }
}
