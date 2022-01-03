use funcmap::{FuncMap, TypeParam};
use units::*;

/// Example data structure illustrating the use of [FuncMap]
/// `L` and `M` are meant to be units of length resp. mass from the [units] module below
#[derive(FuncMap, Debug)]
struct Measurements<L, M> {
    lengths: Vec<L>,
    masses: Vec<M>,
}

/// Markers for the two type parameters of [Measurements]
/// These are abstractions over the concrete indices `0` and `1`
const LENGTH_PARAM: TypeParam<0> = TypeParam;
const MASS_PARAM: TypeParam<1> = TypeParam;

impl<L, M> Measurements<L, M> {
    fn into_base(self) -> Measurements<Meter, Kilogram>
    where
        L: Into<Meter>,
        M: Into<Kilogram>,
    {
        // use `func_map_over` to specify the type parameter of `Measurements`
        // over which the mapping is to be performed
        self.func_map_over(LENGTH_PARAM, Into::into)
            .func_map_over(MASS_PARAM, Into::into)
    }
}

fn main() {
    let measurements = Measurements {
        lengths: vec![Kilometer(5), Kilometer(42)],
        masses: vec![Gram(1000), Gram(5000)],
    };

    println!("{:?}", measurements.into_base());
}

/// Helpers dealing with units
mod units {
    #[derive(Debug)]
    pub struct Meter(pub u32);

    #[derive(Debug)]
    pub struct Kilometer(pub u32);

    #[derive(Debug)]
    pub struct Gram(pub u32);

    #[derive(Debug)]
    pub struct Kilogram(pub u32);

    impl From<Kilometer> for Meter {
        fn from(Kilometer(km): Kilometer) -> Self {
            Self(km * 1000)
        }
    }

    impl From<Gram> for Kilogram {
        fn from(Gram(g): Gram) -> Self {
            Self(g / 1000)
        }
    }
}
