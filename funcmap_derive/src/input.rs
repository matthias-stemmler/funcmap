//! Functionality for preparing the input to `funcmap` derive macros

use crate::{
    ident::{CRATE_IDENT, TRAIT_IDENT},
    ident_collector::IdentCollector,
    opts::{self, FuncMapOpts, Param},
    result::{self, Error, IteratorExt, ResultExt},
    syn_ext::ToNonEmptyTokens,
};

use std::{collections::HashSet, iter};

use proc_macro2::Ident;
use quote::ToTokens;
use syn::{
    visit::Visit, Data, DataEnum, DataStruct, DataUnion, DeriveInput, Field, GenericParam,
    Generics, Path, Token, Type, TypeParam, Variant,
};

/// Input to a `funcmap` derive macro
#[derive(Debug)]
pub(crate) struct FuncMapInput {
    /// Meta information for deriving mappings
    pub(crate) meta: FuncMapMeta,

    /// Identifier of type for which to derive mappings
    pub(crate) ident: Ident,

    /// Generics of type for which to derive mappings
    pub(crate) generics: Generics,

    /// Type parameters for which to derive mappings
    pub(crate) mapped_type_params: Vec<MappedTypeParam>,

    /// Variants of type for which to derive mappings
    ///
    /// For structs, this is a one-element vector
    pub(crate) variants: Vec<Structish>,
}

/// Meta information for deriving mappings
#[derive(Debug)]
pub(crate) struct FuncMapMeta {
    /// Path to the `funcmap` crate
    pub(crate) crate_path: Path,

    /// [`IdentCollector`] where all identifiers that occur within the
    /// definition of the type are already reserved
    pub(crate) ident_collector: IdentCollector,
}

/// Type parameter for which to derive a mapping
#[derive(Debug)]
pub(crate) struct MappedTypeParam {
    /// 0-based index of the type parameter within *all* parameters of the type,
    /// including lifetimes and const generics
    pub(crate) param_idx: usize,

    /// 0-based index of the type parameter within all *type* parameters of the
    /// type
    pub(crate) type_param_idx: usize,

    /// The type parameter itself
    pub(crate) type_param: TypeParam,
}

/// Either a struct or a variant of an enum
#[derive(Debug)]
pub(crate) struct Structish {
    /// Name of the struct/variant
    ///
    /// For structs, this is [`None`]
    pub(crate) variant_ident: Option<Ident>,

    /// Fields of the struct/variant
    pub(crate) fields: Vec<Fieldish>,
}

/// Field of a struct/variant
#[derive(Debug)]
pub(crate) struct Fieldish {
    /// Identifier of the field
    ///
    /// For tuple structs/variants, this is [`None`]
    pub(crate) ident: Option<Ident>,

    /// Type of the field
    pub(crate) ty: Type,
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
                let path = CRATE_IDENT.into();

                Path {
                    leading_colon: Some(<Token![::]>::default()),
                    ..path
                }
            }),

            ident_collector,
        };

        let mut mapped_type_param_idents = HashSet::new();
        let mut result_builder = result::Builder::new();

        for param in opts.params {
            match (
                derive_input.generics.params.iter().find(|p| &&param == p),
                param,
            ) {
                (Some(GenericParam::Type(..)), Param::TypeOrConst(ident)) => {
                    mapped_type_param_idents.insert(ident);
                }
                (Some(GenericParam::Lifetime(..)), param) => {
                    result_builder.add_err(syn::Error::new_spanned(
                        param,
                        format!("cannot implement {} over lifetime parameter", TRAIT_IDENT),
                    ));
                }
                (Some(GenericParam::Const(..)), param) => {
                    result_builder.add_err(syn::Error::new_spanned(
                        param,
                        format!("cannot implement {} over const generic", TRAIT_IDENT),
                    ));
                }
                (_, param) => {
                    result_builder
                        .add_err(syn::Error::new_spanned(param, "unknown generic parameter"));
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
            result_builder.add_err(syn::Error::new_spanned(
                derive_input
                    .generics
                    .to_non_empty_token_stream()
                    .unwrap_or_else(|| derive_input.ident.to_token_stream()),
                "expected at least one type parameter, found none",
            ));
        }

        let variants = match derive_input.data {
            Data::Struct(data_struct) => iter::once(data_struct.try_into()).collect_with_errors(),

            Data::Enum(DataEnum { variants, .. }) => variants
                .into_iter()
                .map(TryInto::try_into)
                .collect_with_errors(),

            Data::Union(DataUnion { union_token, .. }) => iter::once(Err(syn::Error::new_spanned(
                union_token,
                "expected a struct or an enum, found a union",
            )))
            .collect_with_errors(),
        }
        .with_error_from(result_builder)?;

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
                .collect_with_errors()?,
        })
    }
}

impl TryFrom<Variant> for Structish {
    type Error = Error;

    fn try_from(variant: Variant) -> Result<Self, Self::Error> {
        let mut result_builder = result::Builder::new();

        opts::assert_absent(&variant.attrs, "variants").add_err_to(&mut result_builder);

        Ok(Self {
            variant_ident: Some(variant.ident),
            fields: variant
                .fields
                .into_iter()
                .map(TryInto::try_into)
                .collect_with_errors()
                .with_error_from(result_builder)?,
        })
    }
}

impl TryFrom<Field> for Fieldish {
    type Error = Error;

    fn try_from(field: Field) -> Result<Self, Self::Error> {
        opts::assert_absent(&field.attrs, "fields")?;

        Ok(Self {
            ident: field.ident,
            ty: field.ty,
        })
    }
}
