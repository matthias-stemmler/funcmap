use std::{
    error,
    fmt::{self, Display, Formatter},
};

use proc_macro2::TokenStream;

#[derive(Debug, Default)]
pub struct Error(Option<syn::Error>);

impl Error {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn ok(self) -> Result<(), Self> {
        match &self.0 {
            Some(..) => Err(self),
            None => Ok(()),
        }
    }

    pub fn combine<E>(&mut self, other: E)
    where
        E: Into<Self>,
    {
        if let Self(Some(other)) = other.into() {
            match &mut self.0 {
                Some(err) => err.combine(other),
                None => self.0 = Some(other),
            }
        }
    }

    pub fn into_compile_error(self) -> TokenStream {
        self.0
            .map(syn::Error::into_compile_error)
            .unwrap_or_else(TokenStream::new)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(err) => err.fmt(f),
            None => write!(f, "no error"),
        }
    }
}

impl error::Error for Error {}

impl<E> Extend<E> for Error
where
    E: Into<Self>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = E>,
    {
        for err in iter {
            self.combine(err)
        }
    }
}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Self(Some(err))
    }
}

impl<E> FromIterator<E> for Error
where
    E: Into<Error>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = E>,
    {
        let mut err = Self::new();
        err.extend(iter);
        err
    }
}

pub struct IntoIter(Option<<syn::Error as IntoIterator>::IntoIter>);

impl Iterator for IntoIter {
    type Item = <syn::Error as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.iter_mut().flat_map(Iterator::next).next()
    }
}

impl IntoIterator for Error {
    type Item = <syn::Error as IntoIterator>::Item;
    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.map(IntoIterator::into_iter))
    }
}

pub trait IteratorExt<C> {
    fn collect_combining_errors(self) -> Result<C, Error>;
}

impl<C, E, I, T> IteratorExt<C> for I
where
    C: FromIterator<T>,
    E: Into<Error>,
    I: Iterator<Item = Result<T, E>>,
{
    fn collect_combining_errors(self) -> Result<C, Error> {
        let mut error = Error::new();

        let values = self
            .filter_map(|result| result.combine_err_with(&mut error))
            .collect();

        error.ok()?;

        Ok(values)
    }
}

pub trait ResultExt<T> {
    fn combine_err_with(self, error: &mut Error) -> Option<T>;

    fn err_combined_with(self, error: Error) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn combine_err_with(self, error: &mut Error) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                error.combine(err);
                None
            }
        }
    }

    fn err_combined_with(self, mut error: Error) -> Result<T, Error> {
        let result = self.combine_err_with(&mut error);
        error.ok()?;
        Ok(result.unwrap()) // `result` must be `Ok(..)` or else `error.ok()?` would have returned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    use proc_macro2::Span;
    use syn::{parse_quote, Block};

    #[test]
    fn new_error_is_ok() {
        let error = Error::new();

        assert!(error.ok().is_ok());
    }

    #[test]
    fn error_from_syn_error_is_not_ok() {
        let error: Error = syn_error("Error").into();

        assert!(error.ok().is_err());
    }

    #[test]
    fn new_error_displays_no_error() {
        let error = Error::new();

        assert_eq!(error.to_string(), "no error");
    }

    #[test]
    fn error_from_syn_error_displays_its_message() {
        let error: Error = syn_error("Error").into();

        assert_eq!(error.to_string(), "Error");
    }

    #[test]
    fn new_error_is_empty() {
        let error = Error::new();

        assert!(messages(error).is_empty());
    }

    #[test]
    fn combined_error_contains_all_errors() {
        let mut error: Error = syn_error("Error 1").into();
        error.combine(syn_error("Error 2"));

        assert_eq!(messages(error), ["Error 1", "Error 2"]);
    }

    #[test]
    fn new_error_produces_no_compile_error() {
        let error = Error::new();

        assert!(error.into_compile_error().is_empty());
    }

    #[test]
    fn combined_error_produces_combined_compile_errors() {
        let mut error: Error = syn_error("Error 1").into();
        error.combine(syn_error("Error 2"));

        let compile_error = error.into_compile_error();
        let compile_errors: Block = parse_quote!({ #compile_error });

        assert_eq!(compile_errors.stmts.len(), 2);
    }

    #[test]
    fn empty_iterator_collects_into_no_error() {
        let error: Error = iter::empty::<syn::Error>().collect();

        assert!(error.ok().is_ok());
    }

    #[test]
    fn non_empty_iterator_collects_into_all_errors() {
        let error: Error = [syn_error("Error 1"), syn_error("Error 2")]
            .into_iter()
            .collect();

        assert_eq!(messages(error), ["Error 1", "Error 2"]);
    }

    #[test]
    fn all_ok_results_collect_into_all_values() {
        let results: [Result<&str, Error>; 2] = [Ok("Value 1"), Ok("Value 2")];

        let collected: Result<Vec<_>, _> = results.into_iter().collect_combining_errors();

        assert!(collected.is_ok());
        assert_eq!(collected.unwrap(), ["Value 1", "Value 2"]);
    }

    #[test]
    fn not_all_ok_results_collect_into_all_errors() {
        let results = [
            Ok("Value 1"),
            Err(syn_error("Error 1")),
            Ok("Value 2"),
            Err(syn_error("Error 2")),
        ];

        let collected: Result<Vec<_>, _> = results.into_iter().collect_combining_errors();

        assert!(collected.is_err());
        assert_eq!(messages(collected.unwrap_err()), ["Error 1", "Error 2"]);
    }

    #[test]
    fn combining_ok_result_adds_no_error_and_yields_value() {
        let mut error = Error::new();
        let result: Result<_, syn::Error> = Ok("Value");

        let value = result.combine_err_with(&mut error);

        assert!(error.ok().is_ok());
        assert_eq!(value, Some("Value"));
    }

    #[test]
    fn combining_err_result_adds_error_and_yields_none() {
        let mut error = Error::new();
        let result: Result<(), _> = Err(syn_error("Error"));

        let value = result.combine_err_with(&mut error);

        assert_eq!(messages(error), ["Error"]);
        assert!(value.is_none());
    }

    #[test]
    fn ok_result_combined_with_no_error_is_ok() {
        let error = Error::new();
        let result: Result<_, syn::Error> = Ok("Value");

        let combined = result.err_combined_with(error);

        assert!(combined.is_ok());
        assert_eq!(combined.unwrap(), "Value");
    }

    #[test]
    fn ok_result_combined_with_some_error_is_err() {
        let error: Error = syn_error("Error").into();
        let result: Result<_, syn::Error> = Ok("Value");

        let combined = result.err_combined_with(error);

        assert!(combined.is_err());
        assert_eq!(messages(combined.unwrap_err()), ["Error"]);
    }

    #[test]
    fn err_result_combined_with_no_error_is_err() {
        let error = Error::new();
        let result: Result<(), _> = Err(syn_error("Error"));

        let combined = result.err_combined_with(error);

        assert!(combined.is_err());
        assert_eq!(messages(combined.unwrap_err()), ["Error"]);
    }

    fn syn_error(message: impl Display) -> syn::Error {
        syn::Error::new(Span::call_site(), message)
    }

    fn messages(into_iter: impl IntoIterator<Item = impl Display>) -> Vec<String> {
        into_iter.into_iter().map(|item| item.to_string()).collect()
    }
}
