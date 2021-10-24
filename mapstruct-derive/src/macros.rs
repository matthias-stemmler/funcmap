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
