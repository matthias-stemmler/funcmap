#[test]
fn crate_path_uses_leading_colon_by_default() {
    use ::funcmap::FuncMap;

    #[derive(FuncMap)]
    struct Test<T>(T);

    // would be referred to if derive macro produced `funcmap::..` instead of `::funcmap::..`
    mod funcmap {}
}

#[test]
fn crate_path_can_be_configured() {
    use fake_funcmap::FuncMap;

    #[derive(FuncMap)]
    #[funcmap(crate = "fake_funcmap")]
    struct Test<T>(T);

    // would be conflicting if `Test<T1>` implemented `funcmap::FuncMap<T1, T2>`
    impl AssertNotOriginalFuncMap for Test<T1> {}

    fake_funcmap::assert::<Test<T1>, T1, T2, fake_funcmap::TypeParam<0>>();
}

mod fake_funcmap {
    pub use funcmap::*;

    pub struct TypeParam<const N: usize>;

    pub trait FuncMap<A, B, P = TypeParam<0>> {
        type Output;

        fn try_func_map<E, F>(self, _: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>;
    }

    pub fn assert<T, A, B, P>()
    where
        T: FuncMap<A, B, P>,
    {
    }
}

trait AssertNotOriginalFuncMap {}

impl<T> AssertNotOriginalFuncMap for T where T: funcmap::FuncMap<T1, T2> {}

#[derive(Debug)]
struct T1;

#[derive(Debug)]
struct T2;
