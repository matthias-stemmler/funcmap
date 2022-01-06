use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::visit::Visit;

#[derive(Debug, Default)]
pub(crate) struct IdentCollector {
    idents: HashSet<String>,
}

#[derive(Debug, Default)]
pub(crate) struct VisitingIdentCollector(IdentCollector);

impl IdentCollector {
    pub(crate) fn new_visiting() -> VisitingIdentCollector {
        VisitingIdentCollector::default()
    }

    pub(crate) fn reserve_uppercase_letter(&mut self, desired_letter: char, span: Span) -> Ident {
        let letter = self.find_uppercase_letter(desired_letter);
        let ident = Ident::new(&letter, span);
        self.idents.insert(letter);
        ident
    }

    fn find_uppercase_letter(&self, desired_letter: char) -> String {
        if !desired_letter.is_alphabetic() || !desired_letter.is_uppercase() {
            panic!("{} is not an uppercase letter", desired_letter);
        }

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
    use syn::parse_quote;

    use super::*;

    #[test]
    fn test_free_valid() {
        let mut collector = IdentCollector::default();

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "T");
    }

    #[test]
    fn test_reserved() {
        let mut collector = IdentCollector::default();
        let _: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "U");
    }

    #[test]
    fn test_reserved_wraparound() {
        let mut collector = IdentCollector::default();
        let _: Ident = collector.reserve_uppercase_letter('Z', Span::mixed_site());

        let ident: Ident = collector.reserve_uppercase_letter('Z', Span::mixed_site());

        assert_eq!(ident, "A");
    }

    #[test]
    fn test_prefixed() {
        let mut collector = IdentCollector::default();
        for c in 'A'..='Z' {
            let _: Ident = collector.reserve_uppercase_letter(c, Span::mixed_site());
        }

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "__FUNCMAP_T");
    }

    #[test]
    fn test_prefixed_reserved() {
        let mut collector = IdentCollector::default();
        for c in 'A'..='Z' {
            let _: Ident = collector.reserve_uppercase_letter(c, Span::mixed_site());
        }
        let _: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "__FUNCMAP_U");
    }

    #[test]
    fn test_numbered() {
        let mut collector = IdentCollector::default();
        for c in ('A'..='Z').chain('A'..='Z') {
            let _: Ident = collector.reserve_uppercase_letter(c, Span::mixed_site());
        }

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "__FUNCMAP_T2");
    }

    #[test]
    fn test_numbered_reserved() {
        let mut collector = IdentCollector::default();
        for c in ('A'..='Z').chain('A'..='Z') {
            let _: Ident = collector.reserve_uppercase_letter(c, Span::mixed_site());
        }
        let _: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "__FUNCMAP_U2");
    }

    #[test]
    fn test_visiting() {
        let mut collector = IdentCollector::new_visiting();
        collector.visit_derive_input(&parse_quote! {
           struct TestType<T, U, V>;
        });
        let mut collector = collector.into_reserved();

        let ident: Ident = collector.reserve_uppercase_letter('T', Span::mixed_site());

        assert_eq!(ident, "W");
    }

    #[test]
    #[should_panic]
    fn test_lowercase() {
        let mut collector = IdentCollector::default();
        collector.reserve_uppercase_letter('t', Span::mixed_site());
    }

    #[test]
    #[should_panic]
    fn test_nonalphabetic() {
        let mut collector = IdentCollector::default();
        collector.reserve_uppercase_letter('1', Span::mixed_site());
    }
}
