use syn::{parse_quote, Type, TypeParam};

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
}

impl TypeExt for Type {
    fn is_type_param(&self, type_param: &TypeParam) -> bool {
        self == &type_param.to_type()
    }
}
