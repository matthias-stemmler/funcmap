use proc_macro2::Ident;
use syn::fold::{self, Fold};
use syn::{Path, PathArguments, PathSegment, Type, TypeParam, TypePath};

pub trait TypeParamExt {
    fn into_type(self) -> Type;

    fn with_ident(self, ident: Ident) -> Self;

    fn without_default(self) -> Self;
}

impl TypeParamExt for TypeParam {
    fn into_type(self) -> Type {
        self.ident.into_type()
    }

    fn with_ident(self, ident: Ident) -> Self {
        Self { ident, ..self }
    }

    fn without_default(self) -> Self {
        Self {
            eq_token: None,
            default: None,
            ..self
        }
    }
}

pub trait TypeExt {
    fn is_type_param(&self, type_param: &TypeParam) -> bool;

    fn subs_type_param(&self, type_param: &TypeParam, subs_type: &Type) -> Type;
}

impl TypeExt for Type {
    fn is_type_param(&self, type_param: &TypeParam) -> bool {
        match self {
            Type::Path(TypePath {
                qself: None,
                path:
                    Path {
                        leading_colon: None,
                        segments,
                    },
            }) => {
                let mut segments = segments.iter();

                match segments.next() {
                    Some(PathSegment {
                        ident,
                        arguments: PathArguments::None,
                    }) => ident == &type_param.ident && segments.next().is_none(),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn subs_type_param(&self, type_param: &TypeParam, subs_type: &Type) -> Type {
        let mut subs_type_param = SubsTypeParam {
            type_param,
            subs_type,
        };

        subs_type_param.fold_type(self.clone())
    }
}

struct SubsTypeParam<'ast> {
    type_param: &'ast TypeParam,
    subs_type: &'ast Type,
}

impl Fold for SubsTypeParam<'_> {
    fn fold_type(&mut self, ty: Type) -> Type {
        if ty.is_type_param(self.type_param) {
            self.subs_type.clone()
        } else {
            fold::fold_type(self, ty)
        }
    }
}

pub trait IdentExt {
    fn into_type(self) -> Type;
}

impl IdentExt for Ident {
    fn into_type(self) -> Type {
        Type::Path(TypePath {
            qself: None,
            path: self.into(),
        })
    }
}
