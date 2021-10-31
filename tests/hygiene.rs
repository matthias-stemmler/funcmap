use mapstruct::MapStruct;

#[test]
fn conflicting_idents_are_avoided() {
    #[allow(non_snake_case)]
    #[derive(MapStruct)]
    struct A<B, F, const C: usize> {
        B: B,
        F: F,
    }
}
