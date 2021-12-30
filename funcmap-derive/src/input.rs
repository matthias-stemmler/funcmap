use crate::{
    error::{Error, IteratorExt, ResultExt},
    ident_collector::IdentCollector,
    idents::*,
    opts::{assert_no_opts, FuncMapOpts, Param},
    syn_ext::ToNonEmptyTokens,
};

use std::{collections::HashSet, iter};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    visit::Visit, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, GenericParam,
    Generics, Path, Type, TypeParam, Variant,
};

#[derive(Debug)]
pub struct FuncMapInput {
    pub meta: FuncMapMeta,
    pub ident: Ident,
    pub generics: Generics,
    pub mapped_type_params: Vec<MappedTypeParam>,
    pub variants: Vec<Structish>,
}

#[derive(Debug)]
pub struct FuncMapMeta {
    pub crate_path: Path,
    pub ident_collector: IdentCollector,
}

#[derive(Debug)]
pub struct MappedTypeParam {
    pub param_idx: usize,
    pub type_param_idx: usize,
    pub type_param: TypeParam,
}

#[derive(Debug)]
pub struct Structish {
    pub variant_ident: Option<Ident>,
    pub fields: Vec<Fieldish>,
}

#[derive(Debug)]
pub struct Fieldish {
    pub ident: Option<Ident>,
    pub ty: Type,
}

impl TryFrom<DeriveInput> for FuncMapInput {
    type Error = Error;

    fn try_from(derive_input: DeriveInput) -> Result<Self, Self::Error> {
        let ident_collector = {
            let mut ident_collector = IdentCollector::new_visiting();
            ident_collector.visit_derive_input(&derive_input);
            ident_collector.into_reserved()
        };

        let opts: FuncMapOpts = derive_input.attrs.try_into()?;

        let meta = FuncMapMeta {
            crate_path: opts.crate_path.unwrap_or_else(|| {
                let path = CRATE_IDENT.to_ident().into();

                Path {
                    leading_colon: Some(Default::default()),
                    ..path
                }
            }),

            ident_collector,
        };

        let mut mapped_type_param_idents = HashSet::new();
        let mut error = Error::new();

        for param in opts.params {
            match (
                derive_input.generics.params.iter().find(|p| &&param == p),
                param,
            ) {
                (Some(GenericParam::Type(..)), Param::TypeOrConst(ident)) => {
                    mapped_type_param_idents.insert(ident);
                }
                (Some(GenericParam::Lifetime(..)), param) => {
                    error.combine(syn::Error::new_spanned(
                        param,
                        format!("cannot implement {} over lifetime parameter", TRAIT_IDENT),
                    ));
                }
                (Some(GenericParam::Const(..)), param) => {
                    error.combine(syn::Error::new_spanned(
                        param,
                        format!("cannot implement {} over const generic", TRAIT_IDENT),
                    ));
                }
                (_, param) => {
                    error.combine(syn::Error::new_spanned(param, "unknown generic parameter"));
                }
            }
        }

        let mapped_type_params: Vec<_> = derive_input
            .generics
            .params
            .iter()
            .enumerate()
            .filter_map(|(param_idx, param)| match param {
                GenericParam::Type(type_param)
                    if mapped_type_param_idents.is_empty()
                        || mapped_type_param_idents.contains(&type_param.ident) =>
                {
                    Some((param_idx, type_param.clone()))
                }
                _ => None,
            })
            .enumerate()
            .map(
                |(type_param_idx, (param_idx, type_param))| MappedTypeParam {
                    param_idx,
                    type_param_idx,
                    type_param,
                },
            )
            .collect();

        if mapped_type_params.is_empty() {
            error.combine(syn::Error::new_spanned(
                derive_input
                    .generics
                    .to_non_empty_token_stream()
                    .unwrap_or_else(|| derive_input.ident.to_token_stream()),
                "expected at least one type parameter, found none",
            ));
        }

        let variants = match derive_input.data {
            Data::Struct(data_struct) => {
                iter::once(data_struct.try_into()).collect_combining_errors()
            }

            Data::Enum(DataEnum { variants, .. }) => variants
                .into_iter()
                .map(TryInto::try_into)
                .collect_combining_errors(),

            Data::Union(DataUnion { union_token, .. }) => iter::once(Err(syn::Error::new_spanned(
                union_token,
                "expected a struct or an enum, found a union",
            )))
            .collect_combining_errors(),
        }
        .err_combined_with(error)?;

        Ok(Self {
            meta,
            ident: derive_input.ident,
            generics: derive_input.generics,
            mapped_type_params,
            variants,
        })
    }
}

impl TryFrom<DataStruct> for Structish {
    type Error = Error;

    fn try_from(data_struct: DataStruct) -> Result<Self, Self::Error> {
        Ok(Self {
            variant_ident: None,
            fields: data_struct
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect_combining_errors()?,
        })
    }
}

impl TryFrom<Variant> for Structish {
    type Error = Error;

    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        let mut error = Error::new();

        assert_no_opts(&variant.attrs, "variants").combine_err_with(&mut error);

        Ok(Self {
            variant_ident: Some(variant.ident),
            fields: variant
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect_combining_errors()
                .err_combined_with(error)?,
        })
    }
}

impl TryFrom<Field> for Fieldish {
    type Error = Error;

    fn try_from(field: Field) -> Result<Self, Self::Error> {
        assert_no_opts(&field.attrs, "fields")?;

        Ok(Self {
            ident: field.ident,
            ty: field.ty,
        })
    }
}
