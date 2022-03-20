//! Infrastructure for dealing with fallible operations while deriving

use std::error;
use std::fmt::{self, Display, Formatter};

use proc_macro2::TokenStream;

/// An error that occurred while deriving
#[derive(Debug)]
pub(crate) struct Error(syn::Error);

impl Error {
    /// Turns this error into a [`TokenStream`] containing a
    /// `compile_error!(...)` macro invocation with the appropriate
    /// [`Span`](proc_macro2::Span)
    pub(crate) fn into_compile_error(self) -> TokenStream {
        self.0.into_compile_error()
    }

    /// Combines this error with `other` by adding all messages of `other` to
    /// this error
    fn combine<E>(&mut self, other: E)
    where
        E: Into<Self>,
    {
        let Self(other) = other.into();
        self.0.combine(other);
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl error::Error for Error {}

impl From<syn::Error> for Error {
    fn from(err: syn::Error) -> Self {
        Self(err)
    }
}

/// A builder for values of type [`Result<T, Error>`]
#[derive(Debug, Default)]
pub(crate) struct Builder(Option<Error>);

impl Builder {
    /// Creates a new [`Builder`] containing no error yet
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Adds the given error to this builder
    ///
    /// If this builder already contains an error, it is combined with the given
    /// one
    pub(crate) fn add_err<E>(&mut self, err: E) -> &mut Self
    where
        E: Into<Error>,
    {
        self.0 = Some(self.err_combined_with(err));
        self
    }

    /// Builds a result by returning the error contained in this builder, or the
    /// given value if this builder contains no error
    pub(crate) fn err_or<T>(self, value: T) -> Result<T, Error> {
        match self.0 {
            Some(err) => Err(err),
            None => Ok(value),
        }
    }

    fn err_combined_with<E>(&mut self, err: E) -> Error
    where
        E: Into<Error>,
    {
        match self.0.take() {
            Some(mut error) => {
                error.combine(err);
                error
            }
            None => err.into(),
        }
    }
}

impl<E> Extend<E> for Builder
where
    E: Into<Error>,
{
    fn extend<I>(&mut self, iter: I)
    where
        I: IntoIterator<Item = E>,
    {
        for err in iter {
            self.add_err(err);
        }
    }
}

impl<E> FromIterator<E> for Builder
where
    E: Into<Error>,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = E>,
    {
        let mut builder = Self::new();
        builder.extend(iter);
        builder
    }
}

/// An extension trait for types implementing [`Iterator`]
pub(crate) trait IteratorExt<T> {
    /// Collects an iterator of results into a single result
    ///
    /// If all results are [`Ok`], then all values are collected into a
    /// collection of type `C`. If any result is [`Err`], then all errors are
    /// collected into a single [`Error`].
    fn collect_with_errors<C>(self) -> Result<C, Error>
    where
        C: FromIterator<T>;
}

impl<E, I, T> IteratorExt<T> for I
where
    E: Into<Error>,
    I: Iterator<Item = Result<T, E>>,
{
    fn collect_with_errors<C>(self) -> Result<C, Error>
    where
        C: FromIterator<T>,
    {
        let mut builder = Builder::new();

        let values = self
            .filter_map(|result| result.add_err_to(&mut builder))
            .collect();

        builder.err_or(values)
    }
}

/// An extension trait for [`Result<T, E>`]
pub(crate) trait ResultExt<T> {
    /// If this result is [`Err`], adds the error to the given [`Builder`]
    ///
    /// Returns this result's value if it is [`Ok`], or [`None`] otherwise.
    fn add_err_to(self, builder: &mut Builder) -> Option<T>;

    /// Combines the given [`Builder`] with this result
    ///
    /// Returns [`Ok`] if this result is [`Ok`] and the given builder contains
    /// no error. Otherwise returns [`Err`] containing an error combined from
    /// the given builder's error and this result's error.
    fn with_error_from(self, builder: Builder) -> Result<T, Error>;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn add_err_to(self, builder: &mut Builder) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(err) => {
                builder.add_err(err);
                None
            }
        }
    }

    fn with_error_from(self, mut builder: Builder) -> Result<T, Error> {
        match self {
            Ok(value) => builder.err_or(value),
            Err(err) => Err(builder.err_combined_with(err)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::iter;

    use proc_macro2::Span;
    use syn::{parse_quote, Block, Stmt};

    #[test]
    fn new_builder_is_ok_and_yields_value() {
        let builder = Builder::new();

        assert_eq!(builder.err_or(42).ok(), Some(42));
    }

    #[test]
    fn builder_after_add_err_yields_err() {
        let mut builder = Builder::new();
        builder.add_err(syn_error("Error"));

        assert!(builder.err_or(()).is_err());
    }

    #[test]
    fn builder_after_add_err_contains_all_errors() {
        let mut builder = Builder::new();
        builder
            .add_err(syn_error("Error 1"))
            .add_err(syn_error("Error 2"));

        assert_eq!(builder_messages(builder), ["Error 1", "Error 2"]);
    }

    #[test]
    fn builder_after_add_err_produces_combined_compile_errors() {
        let mut builder = Builder::new();
        builder
            .add_err(syn_error("Error 1"))
            .add_err(syn_error("Error 2"));

        let compile_errors: Vec<Stmt> = {
            let result = builder.err_or(());
            assert!(result.is_err());
            let compile_error = result.unwrap_err().into_compile_error();
            let block: Block = parse_quote!({ #compile_error });
            block.stmts
        };

        assert_eq!(compile_errors.len(), 2);
    }

    #[test]
    fn empty_iterator_collects_into_ok_builder() {
        let builder: Builder = iter::empty::<syn::Error>().collect();

        assert!(builder.err_or(()).is_ok());
    }

    #[test]
    fn non_empty_iterator_collects_into_builder_containing_all_errors() {
        let builder: Builder = [syn_error("Error 1"), syn_error("Error 2")]
            .into_iter()
            .collect();

        assert_eq!(builder_messages(builder), ["Error 1", "Error 2"]);
    }

    #[test]
    fn all_ok_results_collect_into_all_values() {
        let results: [Result<&str, Error>; 2] = [Ok("Value 1"), Ok("Value 2")];

        let collected: Result<Vec<_>, _> = results.into_iter().collect_with_errors();

        assert_eq!(collected.ok(), Some(vec!["Value 1", "Value 2"]));
    }

    #[test]
    fn not_all_ok_results_collect_into_all_errors() {
        let results = [
            Ok("Value 1"),
            Err(syn_error("Error 1")),
            Ok("Value 2"),
            Err(syn_error("Error 2")),
        ];

        let collected: Result<Vec<_>, _> = results.into_iter().collect_with_errors();

        assert_eq!(
            collected.err().map(err_messages),
            Some(vec!["Error 1".into(), "Error 2".into()])
        );
    }

    #[test]
    fn ok_result_add_err_to_builder_adds_no_error_and_yields_value() {
        let mut builder = Builder::new();
        let result: Result<_, syn::Error> = Ok("Value");

        let value = result.add_err_to(&mut builder);

        assert!(builder.err_or(()).is_ok());
        assert_eq!(value, Some("Value"));
    }

    #[test]
    fn err_result_add_err_to_builder_adds_error_and_yields_none() {
        let mut builder = Builder::new();
        let result: Result<(), _> = Err(syn_error("Error"));

        let value = result.add_err_to(&mut builder);

        assert_eq!(builder_messages(builder), ["Error"]);
        assert!(value.is_none());
    }

    #[test]
    fn ok_result_with_errors_from_empty_builder_yields_value() {
        let builder = Builder::new();
        let result: Result<_, syn::Error> = Ok("Value");

        let combined = result.with_error_from(builder);

        assert_eq!(combined.ok(), Some("Value"));
    }

    #[test]
    fn ok_result_with_errors_from_non_empty_builder_yields_error() {
        let mut builder = Builder::new();
        builder.add_err(syn_error("Error"));
        let result: Result<_, syn::Error> = Ok("Value");

        let combined = result.with_error_from(builder);

        assert_eq!(combined.err().map(err_messages), Some(vec!["Error".into()]));
    }

    #[test]
    fn err_result_with_errors_from_empty_builder_yields_error() {
        let builder = Builder::new();
        let result: Result<(), _> = Err(syn_error("Error"));

        let combined = result.with_error_from(builder);

        assert_eq!(combined.err().map(err_messages), Some(vec!["Error".into()]));
    }

    #[test]
    fn err_result_with_errors_from_non_empty_builder_yields_all_errors() {
        let mut builder = Builder::new();
        builder.add_err(syn_error("Error 1"));
        let result: Result<(), _> = Err(syn_error("Error 2"));

        let combined = result.with_error_from(builder);

        assert_eq!(
            combined.err().map(err_messages),
            Some(vec!["Error 1".into(), "Error 2".into()])
        );
    }

    fn syn_error(message: impl Display) -> syn::Error {
        syn::Error::new(Span::call_site(), message)
    }

    fn builder_messages(builder: Builder) -> Vec<String> {
        match builder.err_or(()) {
            Ok(..) => Vec::new(),
            Err(err) => err_messages(err),
        }
    }

    fn err_messages(err: Error) -> Vec<String> {
        err.0.into_iter().map(|item| item.to_string()).collect()
    }
}
