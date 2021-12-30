use funcmap::FuncMap;

#[derive(FuncMap)]
#[funcmap(params('a, N, X))]
enum Test<'a, const N: usize> {
    #[funcmap]
    TestVariant1 {
        #[funcmap]
        value1: &'a (),

        #[funcmap]
        value2: &'a (),
    },

    #[funcmap]
    TestVariant2,
}

fn main() {}
