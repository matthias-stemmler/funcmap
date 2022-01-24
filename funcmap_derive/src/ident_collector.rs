//! Provides types for collecting unique identifiers

use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::visit::Visit;

/// Collects unique identifiers for use in generated code
#[derive(Debug, Default)]
pub(crate) struct IdentCollector {
    idents: HashSet<String>,
}

/// Wrapper around [`IdentCollector`] implementing [`Visit`]
///
/// Note that only [`VisitingIdentCollector`] implements [`Visit`] while only
/// [`IdentCollector`] has a
/// [`reserve_uppercase_letter`](IdentCollector::reserve_uppercase_letter)
/// method. A [`VisitingIdentCollector`] can be unwrapped into an
/// [`IdentCollector`] using [`into_reserved`](Self::into_reserved).
///
/// This is to statically ensure the following order of invocations:
/// 1. Calls to `visit_*` methods from the [`Visit`] trait
/// 2. Calls to
/// [`reserve_uppercase_letter`](`IdentCollector::reserve_uppercase_letter`)
///
/// This way, we make sure that identifiers reserved through
/// [`reserve_uppercase_letter`](IdentCollector::reserve_uppercase_letter)
/// are actually unique within the visited syntax tree.
#[derive(Debug, Default)]
pub(crate) struct VisitingIdentCollector(IdentCollector);

impl IdentCollector {
    /// Creates a new [`VisitingIdentCollector`]
    pub(crate) fn new_visiting() -> VisitingIdentCollector {
        VisitingIdentCollector::default()
    }

    /// Reserves an uppercase-letter [`Ident`] with the given [`Span`]
    ///
    /// It is guaranteed that the returned identifier is different from any
    /// identifier previously reserved by this method or by visiting an AST.
    ///
    /// `desired_letter` must be an uppercase letter. It will be used as the
    /// returned identifier if it is still available. Otherwise, a different
    /// letter or combination of letters will be used.
    pub(crate) fn reserve_uppercase_letter(&mut self, desired_letter: char, span: Span) -> Ident {
        let letter = self.find_uppercase_letter(desired_letter);
        let ident = Ident::new(&letter, span);
        self.idents.insert(letter);
        ident
    }

    fn find_uppercase_letter(&self, desired_letter: char) -> String {
        debug_assert!(desired_letter.is_alphabetic() && desired_letter.is_uppercase());

        (0..=usize::MAX)
            .flat_map(|iteration| {
                (desired_letter..='Z').chain('A'..=desired_letter).map(
                    move |letter| match iteration {
                        0 => letter.to_string(),
                        1 => format!("__FUNCMAP_{}", letter),
                        i => format!("__FUNCMAP_{}{}", letter, i),
                    },
                )
            })
            .find(|letter| !self.idents.contains(letter))
            .unwrap()
    }
}

impl VisitingIdentCollector {
    /// Turns this [`VisitingIdentCollector`] into an [`IdentCollector`],
    /// effectively reserving all previously visited identifiers
    ///
    /// Afterwards, more identifiers can be reserved through
    /// [`reserve_uppercase_letter`](IdentCollector::reserve_uppercase_letter),
    /// but not more AST nodes can be visited.
    pub(crate) fn into_reserved(self) -> IdentCollector {
        self.0
    }
}

impl Visit<'_> for VisitingIdentCollector {
    fn visit_ident(&mut self, ident: &Ident) {
        self.0.idents.insert(ident.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use syn::parse_quote;

    #[test]
    fn when_desired_letter_is_available_it_gets_reserved() {
        let mut collector = IdentCollector::default();

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "T");
    }

    #[test]
    fn when_desired_letter_is_already_reserved_uses_next() {
        let mut collector = IdentCollector::default();
        collector.reserve_uppercase_letter('T', Span::call_site());

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "U");
    }

    #[test]
    fn when_rest_of_alphabet_is_already_reserved_wraps_around() {
        let mut collector = IdentCollector::default();
        collector.reserve_uppercase_letter('Z', Span::call_site());

        let ident = collector.reserve_uppercase_letter('Z', Span::call_site());

        assert_eq!(ident, "A");
    }

    #[test]
    fn when_entire_alphabet_is_already_reserved_uses_prefix() {
        let mut collector = IdentCollector::default();
        for c in 'A'..='Z' {
            collector.reserve_uppercase_letter(c, Span::call_site());
        }

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "__FUNCMAP_T");
    }

    #[test]
    fn when_desired_letter_with_prefix_is_already_reserved_uses_next() {
        let mut collector = IdentCollector::default();
        for c in 'A'..='Z' {
            collector.reserve_uppercase_letter(c, Span::call_site());
        }
        collector.reserve_uppercase_letter('T', Span::call_site());

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "__FUNCMAP_U");
    }

    #[test]
    fn when_entire_alphabet_with_prefix_is_already_reserved_uses_counter() {
        let mut collector = IdentCollector::default();
        for c in ('A'..='Z').chain('A'..='Z') {
            collector.reserve_uppercase_letter(c, Span::call_site());
        }

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "__FUNCMAP_T2");
    }

    #[test]
    fn when_desired_letter_with_prefix_and_counter_is_already_reserved_uses_next() {
        let mut collector = IdentCollector::default();
        for c in ('A'..='Z').chain('A'..='Z') {
            collector.reserve_uppercase_letter(c, Span::call_site());
        }
        collector.reserve_uppercase_letter('T', Span::call_site());

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "__FUNCMAP_U2");
    }

    #[test]
    fn visiting_reserves_visited_idents() {
        let mut collector = IdentCollector::new_visiting();
        collector.visit_derive_input(&parse_quote! {
           struct TestType<T, U, V>;
        });
        let mut collector = collector.into_reserved();

        let ident = collector.reserve_uppercase_letter('T', Span::call_site());

        assert_eq!(ident, "W");
    }
}
