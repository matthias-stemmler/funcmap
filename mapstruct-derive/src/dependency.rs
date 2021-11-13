use crate::type_ext::TypeExt;
use syn::visit::{self, Visit};
use syn::{PathSegment, Type, TypeParam};

pub trait DependencyOnExt {
    fn dependency_on<'ast>(&'ast self, type_param: &'ast TypeParam) -> Option<&'ast Type>;
}

impl DependencyOnExt for Type {
    fn dependency_on<'ast>(&'ast self, type_param: &'ast TypeParam) -> Option<&'ast Type> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

impl DependencyOnExt for PathSegment {
    fn dependency_on<'ast>(&'ast self, type_param: &'ast TypeParam) -> Option<&'ast Type> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_path_segment(self);
        visitor.into_dependency()
    }
}

#[derive(Debug)]
struct DependencyVisitor<'ast> {
    type_param: &'ast TypeParam,
    dependency: Option<&'ast Type>,
}

impl<'ast> DependencyVisitor<'ast> {
    fn new(type_param: &'ast TypeParam) -> Self {
        Self {
            type_param,
            dependency: None,
        }
    }

    fn into_dependency(self) -> Option<&'ast Type> {
        self.dependency
    }
}

impl<'ast> Visit<'ast> for DependencyVisitor<'ast> {
    fn visit_type(&mut self, ty: &'ast Type) {
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
