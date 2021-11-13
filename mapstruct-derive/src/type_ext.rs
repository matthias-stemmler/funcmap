use proc_macro2::Ident;
use syn::fold::{self, Fold};
use syn::{Type, TypeParam, TypePath};

pub trait TypeParamExt {
    fn to_type(&self) -> Type;
}

impl TypeParamExt for TypeParam {
    fn to_type(&self) -> Type {
        Type::from_ident(self.ident.clone())
    }
}

pub trait TypeExt {
    fn from_ident(ident: Ident) -> Self;

    fn is_type_param(&self, type_param: &TypeParam) -> bool;

    fn subs_type_param(&self, type_param: &TypeParam, subs_type: &Type) -> Type;
}

impl TypeExt for Type {
    fn from_ident(ident: Ident) -> Self {
        Self::Path(TypePath {
            qself: None,
            path: ident.into(),
        })
    }

    fn is_type_param(&self, type_param: &TypeParam) -> bool {
        self == &type_param.to_type()
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
