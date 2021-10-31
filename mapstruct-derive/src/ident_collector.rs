use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::parse::Parse;
use syn::parse_quote;
use syn::visit::Visit;

#[derive(Debug)]
pub struct IdentCollector {
    idents: HashSet<String>,
}

#[derive(Debug)]
pub struct VisitingIdentCollector(IdentCollector);

impl IdentCollector {
    pub fn new() -> Self {
        Self {
            idents: HashSet::new(),
        }
    }

    pub fn new_visiting() -> VisitingIdentCollector {
        VisitingIdentCollector(Self::new())
    }

    pub fn reserve_uppercase_letter<T>(&mut self, desired_letter: char) -> T
    where
        T: Parse,
    {
        let letter = self.find_uppercase_letter(desired_letter);
        let ident = Ident::new(&letter, Span::call_site());
        self.idents.insert(letter);
        parse_quote!(#ident)
    }

    fn find_uppercase_letter(&self, desired_letter: char) -> String {
        let desired_letter = if desired_letter.is_alphabetic() && desired_letter.is_uppercase() {
            desired_letter
        } else {
            'A'
        };

        (0..)
            .flat_map(|iteration| {
                (desired_letter..='Z')
                    .chain('A'..=desired_letter)
                    .map(move |c| match iteration {
                        0 => c.to_string(),
                        1 => format!("__MAPSTRUCT_{}", c),
                        _ => format!("__MAPSTRUCT_{}{}", c, iteration),
                    })
            })
            .find(|c| !self.idents.contains(c))
            .unwrap()
    }
}

impl Default for IdentCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl VisitingIdentCollector {
    pub fn into_reserved(self) -> IdentCollector {
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
        let mut collector = IdentCollector::new();

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "T");
    }

    #[test]
    fn test_free_lowercase() {
        let mut collector = IdentCollector::new();

        let ident: Ident = collector.reserve_uppercase_letter('t');

        assert_eq!(ident, "A");
    }

    #[test]
    fn test_free_nonalphabetic() {
        let mut collector = IdentCollector::new();

        let ident: Ident = collector.reserve_uppercase_letter('1');

        assert_eq!(ident, "A");
    }

    #[test]
    fn test_reserved() {
        let mut collector = IdentCollector::new();
        let _: Ident = collector.reserve_uppercase_letter('T');

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "U");
    }

    #[test]
    fn test_reserved_wraparound() {
        let mut collector = IdentCollector::new();
        let _: Ident = collector.reserve_uppercase_letter('Z');

        let ident: Ident = collector.reserve_uppercase_letter('Z');

        assert_eq!(ident, "A");
    }

    #[test]
    fn test_prefixed() {
        let mut collector = IdentCollector::new();
        for c in 'A'..='Z' {
            let _: Ident = collector.reserve_uppercase_letter(c);
        }

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "__MAPSTRUCT_T");
    }

    #[test]
    fn test_prefixed_reserved() {
        let mut collector = IdentCollector::new();
        for c in 'A'..='Z' {
            let _: Ident = collector.reserve_uppercase_letter(c);
        }
        let _: Ident = collector.reserve_uppercase_letter('T');

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "__MAPSTRUCT_U");
    }

    #[test]
    fn test_numbered() {
        let mut collector = IdentCollector::new();
        for c in ('A'..='Z').chain('A'..='Z') {
            let _: Ident = collector.reserve_uppercase_letter(c);
        }

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "__MAPSTRUCT_T2");
    }

    #[test]
    fn test_numbered_reserved() {
        let mut collector = IdentCollector::new();
        for c in ('A'..='Z').chain('A'..='Z') {
            let _: Ident = collector.reserve_uppercase_letter(c);
        }
        let _: Ident = collector.reserve_uppercase_letter('T');

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "__MAPSTRUCT_U2");
    }

    #[test]
    fn test_visiting() {
        let mut collector = IdentCollector::new_visiting();
        collector.visit_derive_input(&parse_quote! {
           struct TestType<T, U, V>;
        });
        let mut collector = collector.into_reserved();

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "W");
    }
}
