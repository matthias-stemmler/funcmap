/// Usage of [`FuncMap`] to "deeply" convert units, using independent type
/// parameters for different units
use funcmap::{FuncMap, TypeParam};
use units::*;

/// Example data structure illustrating the use of [`FuncMap`]
/// `L` and `M` are meant to be units of length resp. mass from the [units]
/// module below
#[derive(FuncMap, Debug)]
struct Measurements<L, M> {
    lengths: Vec<L>,
    masses: Vec<M>,
}

/// Aliases for the markers for the two type parameters of [`Measurements`]
/// These are abstractions over the concrete indices `0` and `1`
type LengthParam = TypeParam<0>;
type MassParam = TypeParam<1>;

impl<L, M> Measurements<L, M> {
    fn into_base(self) -> Measurements<Meter, Kilogram>
    where
        L: Into<Meter>,
        M: Into<Kilogram>,
    {
        // use `func_map_over` to specify the type parameter of `Measurements`
        // over which the mapping is to be performed
        self.func_map_over::<LengthParam, _>(Into::into)
            .func_map_over::<MassParam, _>(Into::into)
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
