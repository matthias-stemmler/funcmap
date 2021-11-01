use crate::type_param::TypeExt;
use syn::{
    fold::{self, Fold},
    Type, TypeParam,
};

pub struct TypeTemplate<'a> {
    template_type: &'a Type,
    type_param: &'a TypeParam,
}

impl<'a> TypeTemplate<'a> {
    pub fn new(template_type: &'a Type, type_param: &'a TypeParam) -> Self {
        Self {
            template_type,
            type_param,
        }
    }

    pub fn apply(&self, ty: Type) -> Type {
        let mut folder = TypeTemplateFolder { template: self, ty };
        folder.fold_type(self.template_type.clone())
    }
}

struct TypeTemplateFolder<'a> {
    template: &'a TypeTemplate<'a>,
    ty: Type,
}

impl Fold for TypeTemplateFolder<'_> {
    fn fold_type(&mut self, ty: Type) -> Type {
        if ty.is_type_param(self.template.type_param) {
            self.ty.clone()
        } else {
            fold::fold_type(self, ty)
        }
    }
}
