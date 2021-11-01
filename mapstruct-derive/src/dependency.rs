use crate::type_param::TypeExt;
use syn::visit::{self, Visit};
use syn::{PathSegment, Type, TypeParam};

pub trait DependencyOnExt {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Type>;
}

impl DependencyOnExt for Type {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Type> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

impl DependencyOnExt for PathSegment {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Type> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_path_segment(self);
        visitor.into_dependency()
    }
}

struct DependencyVisitor<'a> {
    type_param: &'a TypeParam,
    dependency: Option<&'a Type>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(type_param: &'a TypeParam) -> Self {
        Self {
            type_param,
            dependency: None,
        }
    }

    fn into_dependency(self) -> Option<&'a Type> {
        self.dependency
    }
}

impl<'a> Visit<'a> for DependencyVisitor<'a> {
    fn visit_type(&mut self, ty: &'a Type) {
        match self.dependency {
            None if ty.is_type_param(self.type_param) => self.dependency = Some(ty),
            None => visit::visit_type(self, ty),
            _ => (),
        };
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use syn::parse_quote;

    use super::*;

    #[rstest]
    #[case(parse_quote ! (Foo), false)]
    #[case(parse_quote ! (T), true)]
    #[case(parse_quote ! ((T)), true)]
    #[case(parse_quote ! ((Foo, T)), true)]
    #[case(parse_quote ! ([T]), true)]
    #[case(parse_quote ! ([T; 1]), true)]
    #[case(parse_quote ! (fn (T) -> Foo), true)]
    #[case(parse_quote ! (fn (Foo) -> T), true)]
    #[case(parse_quote ! (* const T), true)]
    #[case(parse_quote ! (* mut T), true)]
    #[case(parse_quote ! (& T), true)]
    #[case(parse_quote ! (& mut T), true)]
    #[case(parse_quote ! (Foo::T), false)]
    #[case(parse_quote ! (Foo < Bar >), false)]
    #[case(parse_quote ! (Foo < T >), true)]
    #[case(parse_quote ! (Foo < Bar < T >>), true)]
    #[case(parse_quote ! (T < Foo >), false)]
    #[case(parse_quote ! (T::Foo < Bar >), false)]
    #[case(parse_quote ! (< T as Foo >::Bar < Baz >), true)]
    #[case(parse_quote ! (Foo < Bar, T >), true)]
    #[case(parse_quote ! (Foo < 'T >), false)]
    fn test_depends_on(#[case] ty: Type, #[case] expected_result: bool) {
        let type_param: TypeParam = parse_quote!(T);

        assert_eq!(ty.dependency_on(&type_param).is_some(), expected_result);
    }
}
