use syn::{
    fold::{self, Fold},
    parse_quote, Type, TypeParam,
};

pub trait TypeParamExt {
    fn to_type(&self) -> Type;
}

impl TypeParamExt for TypeParam {
    fn to_type(&self) -> Type {
        parse_quote!(#self)
    }
}

pub trait TypeExt {
    fn is_type_param(&self, type_param: &TypeParam) -> bool;
    fn subs_type_param(&self, type_param: &TypeParam, subs_type: &Type) -> Type;
}

impl TypeExt for Type {
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

struct SubsTypeParam<'a> {
    type_param: &'a TypeParam,
    subs_type: &'a Type,
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
