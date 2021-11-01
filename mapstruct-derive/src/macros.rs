macro_rules! debug_assert_parse {
    ($token_stream:ident as $ty:ty) => {
        debug_assert!(::syn::parse2::<$ty>($token_stream.clone()).is_ok());
    };
}

pub(crate) use debug_assert_parse;
