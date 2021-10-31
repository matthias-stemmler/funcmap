use proc_macro2::Ident;
use syn::visit::{self, Visit};
use syn::{Path, PathSegment, Type, TypeParam};

use crate::path::is_ident;

pub trait DependencyOnExt {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Path>;
}

impl DependencyOnExt for Type {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Path> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_type(self);
        visitor.into_dependency()
    }
}

impl DependencyOnExt for PathSegment {
    fn dependency_on<'a>(&'a self, type_param: &'a TypeParam) -> Option<&'a Path> {
        let mut visitor = DependencyVisitor::new(type_param);
        visitor.visit_path_segment(self);
        visitor.into_dependency()
    }
}

struct DependencyVisitor<'a> {
    ident: &'a Ident,
    dependency: Option<&'a Path>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(type_param: &'a TypeParam) -> Self {
        Self {
            ident: &type_param.ident,
            dependency: None,
        }
    }

    fn into_dependency(self) -> Option<&'a Path> {
        self.dependency
    }
}

impl<'a> Visit<'a> for DependencyVisitor<'a> {
    fn visit_path(&mut self, path: &'a Path) {
        match self.dependency {
            None if is_ident(path, self.ident) => self.dependency = Some(path),
            None => visit::visit_path(self, path),
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
