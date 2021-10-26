use syn::{
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypeArray, TypePath,
};

pub enum TypeNesting<'a> {
    Basic,
    Nested(Vec<&'a Type>),
    NotNested,
}

pub fn type_nesting(ty: &Type) -> TypeNesting {
    match ty {
        Type::Array(TypeArray { elem, .. }) => TypeNesting::Nested(vec![elem]),
        Type::Path(TypePath { path, .. }) => {
            let segment = match path.segments.first() {
                Some(segment) => segment,
                None => return TypeNesting::NotNested,
            };

            let args = match &segment.arguments {
                PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) => args,
                PathArguments::None => return TypeNesting::Basic,
                PathArguments::Parenthesized(..) => return TypeNesting::NotNested,
            };

            let type_args: Vec<_> = args.iter().filter_map(|arg| match arg {
                GenericArgument::Type(ty) => Some(ty),
                _ => None
            }).collect();

            if type_args.is_empty() {
                TypeNesting::Basic
            } else {
                TypeNesting::Nested(type_args)
            }
        }
        _ => TypeNesting::NotNested,
    }
}
