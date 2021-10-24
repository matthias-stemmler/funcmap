use syn::{
    AngleBracketedGenericArguments, GenericArgument, PathArguments, Type, TypeArray, TypePath,
};

pub enum TypeNesting<'a> {
    Basic,
    Nested(&'a Type),
    NotNested,
}

pub fn type_nesting(ty: &Type) -> TypeNesting {
    match ty {
        Type::Array(TypeArray { elem, .. }) => TypeNesting::Nested(elem),
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

            match args.first() {
                Some(GenericArgument::Type(ty)) => TypeNesting::Nested(ty),
                _ => TypeNesting::NotNested,
            }
        }
        _ => TypeNesting::NotNested,
    }
}
