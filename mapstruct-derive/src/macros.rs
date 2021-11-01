macro_rules! debug_assert_parse {
    ($token_stream:ident as $ty:ty) => {
        debug_assert!(::syn::parse2::<$ty>($token_stream.clone()).is_ok());
    };
}

macro_rules! fail {
    ($spanned:expr, $message:expr $(,$args:expr)*) => {
        {
            use ::syn::spanned::Spanned;

            return ::syn::Error::new(
                $spanned.span(),
                format!(concat!("failed to derive MapStruct: ", $message) $(,$args)*)
            )
            .to_compile_error();
        }
    };
}

pub(crate) use fail;
pub(crate) use debug_assert_parse;