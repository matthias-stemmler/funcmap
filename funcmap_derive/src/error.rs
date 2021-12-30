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

    pub fn combine<E>(&mut self, another: E)
    where
        E: Into<Self>,
    {
        if let Self(Some(another)) = another.into() {
            match &mut self.0 {
                Some(err) => err.combine(another),
                None => self.0 = Some(another),
            }
        }
    }

    pub fn to_compile_error(&self) -> TokenStream {
        self.0
            .as_ref()
            .map(syn::Error::to_compile_error)
            .unwrap_or_else(TokenStream::new)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Some(err) => err.fmt(f),
            None => Ok(()),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.0.as_ref().map(error::Error::source).flatten()
    }
}

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
        Ok(result.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use proc_macro2::Span;

    #[test]
    fn collect_combining_errors_success() {
        let results: Vec<Result<&str, Error>> = vec![Ok("Value 1"), Ok("Value 2")];
        let collected: Result<Vec<_>, _> = results.into_iter().collect_combining_errors();

        assert!(collected.is_ok());
        assert_eq!(collected.unwrap(), ["Value 1", "Value 2"]);
    }

    #[test]
    fn collect_combining_errors_failure() {
        let results = vec![
            Ok("Value 1"),
            Err(syn::Error::new(Span::call_site(), "Error 1")),
            Ok("Value 2"),
            Err(syn::Error::new(Span::call_site(), "Error 2")),
        ];
        let collected: Result<Vec<_>, _> = results.into_iter().collect_combining_errors();

        assert!(collected.is_err());
        assert_eq!(
            collected
                .unwrap_err()
                .0
                .unwrap()
                .into_iter()
                .map(|err| err.to_string())
                .collect::<Vec<_>>(),
            ["Error 1", "Error 2"]
        );
    }
}
