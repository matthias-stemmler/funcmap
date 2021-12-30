use funcmap::FuncMap;

#[test]
fn generics_to_be_mapped_can_be_configured() {
    fn noop() {}

    #[derive(FuncMap, Debug, PartialEq)]
    #[funcmap(params(T))]
    struct Test<S, T> {
        value: T,
        not_mappable: fn() -> S,
    }

    let src = Test {
        value: T1,
        not_mappable: noop,
    };
    let dst = src.func_map(|_| T2);

    assert_eq!(
        dst,
        Test {
            value: T2,
            not_mappable: noop
        }
    );
}

#[derive(Debug, PartialEq)]
struct T1;

#[derive(Debug, PartialEq)]
struct T2;
