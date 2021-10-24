use std::collections::HashSet;

use proc_macro2::{Ident, Span};
use syn::parse::Parse;
use syn::parse_quote;
use syn::visit::Visit;

pub struct IdentCollector {
    idents: HashSet<String>,
}

impl IdentCollector {
    pub fn new() -> Self {
        Self {
            idents: HashSet::new(),
        }
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

        let mut iteration = 0;

        loop {
            let letter = (desired_letter..='Z')
                .chain('A'..=desired_letter)
                .map(|c| match iteration {
                    0 => c.to_string(),
                    1 => format!("__MAPSTRUCT_{}", c),
                    _ => format!("__MAPSTRUCT_{}{}", c, iteration),
                })
                .find(|c| !self.idents.contains(c));

            if let Some(letter) = letter {
                return letter;
            }

            iteration += 1;
        }
    }
}

impl Visit<'_> for IdentCollector {
    fn visit_ident(&mut self, ident: &Ident) {
        self.idents.insert(ident.to_string());
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
    fn test_visited() {
        let mut collector = IdentCollector::new();
        collector.visit_derive_input(&parse_quote! {
           struct TestType<T, U, V>;
        });

        let ident: Ident = collector.reserve_uppercase_letter('T');

        assert_eq!(ident, "W");
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
}
