use funcmap::FuncMap;

#[test]
fn non_camel_case_types() {
    #![deny(non_camel_case_types)]

    #[allow(non_camel_case_types)]
    #[derive(FuncMap)]
    struct Test<t>(t);
}

#[test]
fn unused_qualifications() {
    #![deny(unused_qualifications)]

    #[allow(unused_qualifications)]
    #[derive(FuncMap)]
    struct Test<T>(core::option::Option<T>);
}

#[test]
fn deprecated() {
    #![deny(deprecated)]

    #[deprecated]
    #[derive(FuncMap)]
    struct Deprecated<T>(T);

    #[allow(deprecated)]
    #[derive(FuncMap)]
    struct Test<T>(Deprecated<T>);
}

#[test]
fn drop_bounds() {
    #![deny(drop_bounds)]

    #[allow(drop_bounds)]
    #[derive(FuncMap)]
    struct Test<T>(T)
    where
        T: Drop;
}

#[test]
fn dyn_drop() {
    #![deny(dyn_drop)]

    #[allow(dyn_drop)]
    #[allow(trivial_bounds)]
    #[derive(FuncMap)]
    struct Test<T>(T)
    where
        for<'a> &'a dyn Drop: Copy;
}
