use std::collections::HashSet;
use syn::{punctuated::Punctuated, token::Add, TypeParamBound};

#[derive(Debug, Default)]
pub struct BoundCollector {
    bounds: HashSet<TypeParamBound>,
}

impl BoundCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, bound: TypeParamBound) {
        self.bounds.insert(bound);
    }

    pub fn into_bounds(self) -> Punctuated<TypeParamBound, Add> {
        self.bounds.into_iter().collect()
    }
}
