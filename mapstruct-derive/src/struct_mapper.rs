use std::collections::HashSet;

use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, Index, Type, TypeParam, TypeTuple, WherePredicate};

use crate::depends_on::depends_on;
use crate::macros::fail;
use crate::subs_type_param::subs_type_param;
use crate::type_nesting::{type_nesting, TypeNesting};

pub struct StructMapper<'a> {
    src_type_param: &'a TypeParam,
    dst_type_param: &'a TypeParam,
    where_predicates: HashSet<WherePredicate>,
}

impl<'a> StructMapper<'a> {
    pub fn new(src_type_param: &'a TypeParam, dst_type_param: &'a TypeParam) -> Self {
        Self {
            src_type_param,
            dst_type_param,
            where_predicates: HashSet::new(),
        }
    }

    pub fn where_clause(&self) -> impl ToTokens {
        if self.where_predicates.is_empty() {
            quote!()
        } else {
            let where_predicates = self.where_predicates.iter();
            quote!(where #(#where_predicates,)*)
        }
    }

    pub fn map_struct(
        &mut self,
        mappable: TokenStream,
        ty: &Type,
        type_param: &TypeParam,
    ) -> TokenStream {
        if !depends_on(ty, type_param) {
            return mappable;
        }

        if let Type::Tuple(type_tuple) = ty {
            return self.map_tuple(mappable, type_tuple, type_param);
        }

        match type_nesting(ty) {
            TypeNesting::Basic => quote!(f(#mappable)),
            TypeNesting::Nested(inner_ty) => {
                let src_type = subs_type_param(ty, type_param, self.src_type_param);
                let dst_type = subs_type_param(ty, type_param, self.dst_type_param);
                let inner_src_type = subs_type_param(inner_ty, type_param, self.src_type_param);
                let inner_dst_type = subs_type_param(inner_ty, type_param, self.dst_type_param);

                self.where_predicates.insert(parse_quote! {
                    #src_type: ::mapstruct::MapStruct<#inner_src_type, #inner_dst_type, Output = #dst_type>
                });

                let inner_ident = quote! { value };
                let mapped = self.map_struct(inner_ident.clone(), inner_ty, type_param);

                quote! {
                    #mappable.map_struct(|#inner_ident: #inner_src_type| #mapped)
                }
            }
            TypeNesting::NotNested => fail!(ty, "type not supported"),
        }
    }

    fn map_tuple(
        &mut self,
        mappable: TokenStream,
        type_tuple: &TypeTuple,
        type_param: &TypeParam,
    ) -> TokenStream {
        let mapped = type_tuple.elems.iter().enumerate().map(|(i, ty)| {
            let idx = Index::from(i);
            let mappable = quote!(#mappable.#idx);
            self.map_struct(mappable, ty, type_param)
        });

        quote! {
            (#(#mapped),*)
        }
    }
}
