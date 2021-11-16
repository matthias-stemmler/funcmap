use crate::predicates::UniquePredicates;
use crate::syn_ext::{DependencyOn, SubsType};
use proc_macro2::{Ident, Span, TokenStream};
use proc_macro_error::abort;
use quote::{quote_spanned, ToTokens};
use syn::{
    parse_quote, punctuated::Pair, AngleBracketedGenericArguments, GenericArgument, Index, Path,
    PathArguments, PathSegment, QSelf, Type, TypeArray, TypeParam, TypePath,
};

pub fn map_expr(
    mappable: TokenStream,
    ty: &Type,
    type_param: &TypeParam,
    src_type_ident: &Ident,
    dst_type_ident: &Ident,
    mapping_fn_ident: &Ident,
) -> (TokenStream, UniquePredicates) {
    let mut mapper = Mapper {
        type_param,
        src_type_ident,
        dst_type_ident,
        mapping_fn_ident,
        unique_predicates: UniquePredicates::new(),
    };

    let mapped = mapper.map(mappable, ty);
    (mapped, mapper.unique_predicates)
}

#[derive(Debug)]
struct Mapper<'ast> {
    type_param: &'ast TypeParam,
    src_type_ident: &'ast Ident,
    dst_type_ident: &'ast Ident,
    mapping_fn_ident: &'ast Ident,
    unique_predicates: UniquePredicates,
}

impl<'ast> Mapper<'ast> {
    fn map(&mut self, mappable: TokenStream, ty: &Type) -> TokenStream {
        if ty.dependency_on(self.type_param).is_none() {
            return mappable;
        }

        match ty {
            Type::Array(TypeArray { elem: inner_ty, .. }) => {
                let (src_type, dst_type) = self.subs_types(ty.clone());
                let (inner_src_type, inner_dst_type) = self.subs_types(Type::clone(inner_ty));

                self.unique_predicates.add(parse_quote! {
                    #src_type: ::mapstruct::MapStruct<
                        #inner_src_type,
                        #inner_dst_type,
                        Output = #dst_type
                    >
                });

                let closure = self.map_closure(inner_ty);

                quote_spanned!(Span::mixed_site() => #mappable.map_struct(#closure))
            }
            Type::Path(type_path) => {
                let TypePath {
                    qself,
                    path:
                        Path {
                            leading_colon,
                            segments,
                        },
                } = type_path;

                if let Some(QSelf { ty, .. }) = &qself {
                    if let Some(dep_ty) = ty.dependency_on(self.type_param) {
                        abort!(dep_ty, "mapping over self type is not supported");
                    }
                }

                let (prefix, ident, arguments) = {
                    let mut prefix = segments.clone();
                    match prefix.pop() {
                        Some(Pair::End(PathSegment { ident, arguments })) => {
                            (prefix, ident, arguments)
                        }
                        Some(_) => abort!(
                            segments,
                            "mapping over type with trailing :: is not supported"
                        ),
                        None => abort!(segments, "mapping over empty type is not supported"),
                    }
                };

                if let Some(prefix_dep) = prefix
                    .iter()
                    .find_map(|segment| segment.dependency_on(self.type_param))
                {
                    abort!(
                        prefix_dep,
                        "mapping over types with associated items is not supported"
                    );
                }

                let angle_bracketed = match arguments {
                    PathArguments::None => {
                        let mapping_fn_ident = self.mapping_fn_ident;
                        return quote_spanned!(Span::mixed_site() => #mapping_fn_ident(#mappable));
                    }
                    PathArguments::AngleBracketed(angle_bracketed) => angle_bracketed,
                    PathArguments::Parenthesized(..) => {
                        abort!(arguments, "mapping over function types is not supported")
                    }
                };

                let args = angle_bracketed.args;

                let arg_types = args.iter().filter_map(|arg| match arg {
                    GenericArgument::Type(ty) => Some(ty),
                    _ => None,
                });

                let mut mappable = mappable;

                for (type_idx, arg_type) in arg_types.enumerate() {
                    if arg_type.dependency_on(self.type_param).is_none() {
                        continue;
                    }

                    let (inner_src_type, inner_dst_type) = self.subs_types(arg_type.clone());

                    let make_type = |mapped_until_idx: usize| {
                        let mapped_args = map_type_args(args.iter(), |type_arg_idx, ty: &Type| {
                            if type_arg_idx >= mapped_until_idx {
                                self.subs_src_type(ty.clone())
                            } else {
                                self.subs_dst_type(ty.clone())
                            }
                        });

                        Type::Path(TypePath {
                            qself: qself.clone(),
                            path: Path {
                                leading_colon: *leading_colon,
                                segments: prefix
                                    .iter()
                                    .cloned()
                                    .chain([PathSegment {
                                        ident: ident.clone(),
                                        arguments: PathArguments::AngleBracketed(
                                            AngleBracketedGenericArguments {
                                                args: mapped_args.collect(),
                                                ..angle_bracketed
                                            },
                                        ),
                                    }])
                                    .collect(),
                            },
                        })
                    };

                    let src_type = make_type(type_idx);
                    let dst_type = make_type(type_idx + 1);

                    self.unique_predicates.add(parse_quote! {
                        #src_type: ::mapstruct::MapStruct<
                            #inner_src_type,
                            #inner_dst_type,
                            ::mapstruct::TypeParam<#type_idx>, Output = #dst_type
                        >
                    });

                    let closure = self.map_closure(arg_type);

                    mappable = quote_spanned! { Span::mixed_site() =>
                        #mappable.map_struct_over(::mapstruct::TypeParam::<#type_idx>, #closure)
                    }
                }

                mappable
            }
            Type::Tuple(type_tuple) => {
                let mapped = type_tuple.elems.iter().enumerate().map(|(i, ty)| {
                    let idx = Index::from(i);
                    let mappable = quote_spanned!(Span::mixed_site() => #mappable.#idx);
                    self.map(mappable, ty)
                });

                quote_spanned!(Span::mixed_site() => (#(#mapped),*))
            }
            _ => abort!(ty, "type not supported"),
        }
    }

    fn map_closure(&mut self, ty: &Type) -> TokenStream {
        let closure_arg = Ident::new("value", Span::mixed_site());
        let mapped = self.map(closure_arg.clone().into_token_stream(), ty);
        quote_spanned!(Span::mixed_site() => |#closure_arg| #mapped)
    }

    fn subs_src_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.type_param.ident, self.src_type_ident)
    }

    fn subs_dst_type(&self, ty: Type) -> Type {
        ty.subs_type(&self.type_param.ident, self.dst_type_ident)
    }

    fn subs_types(&self, ty: Type) -> (Type, Type) {
        (self.subs_src_type(ty.clone()), self.subs_dst_type(ty))
    }
}

fn map_type_args<I, F>(iter: I, f: F) -> MapTypeArgs<I, F> {
    MapTypeArgs::new(iter, f)
}

#[derive(Debug)]
struct MapTypeArgs<I, F> {
    iter: I,
    f: F,
    type_idx: usize,
}

impl<I, F> MapTypeArgs<I, F> {
    fn new(iter: I, f: F) -> Self {
        Self {
            iter,
            f,
            type_idx: 0,
        }
    }
}

impl<'ast, I, F> Iterator for MapTypeArgs<I, F>
where
    I: Iterator<Item = &'ast GenericArgument>,
    F: FnMut(usize, &'ast Type) -> Type,
{
    type Item = GenericArgument;

    fn next(&mut self) -> Option<Self::Item> {
        Some(match self.iter.next()? {
            GenericArgument::Type(ty) => {
                let arg = GenericArgument::Type((self.f)(self.type_idx, ty));
                self.type_idx += 1;
                arg
            }
            arg => arg.clone(),
        })
    }
}
