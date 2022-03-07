//! Implementations of [`FuncMap`](crate::FuncMap) and
//! [`TryFuncMap`](crate::TryFuncMap) for types in [`core`]

/// Implementations for [arrays](prim@array)
mod array {
    use crate::{array, FuncMap, TryFuncMap};

    impl<A, B, const N: usize> FuncMap<A, B> for [A; N] {
        type Output = [B; N];

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f)
        }
    }

    impl<A, B, const N: usize> TryFuncMap<A, B> for [A; N] {
        type Output = [B; N];

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            array::try_map(self, f)
        }
    }
}

/// Implementations for [`core::ops::Bound`]
mod bound {
    use crate::{FuncMap, TryFuncMap};

    use core::ops::Bound;

    impl<A, B> FuncMap<A, B> for Bound<A> {
        type Output = Bound<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            match self {
                Self::Included(bound) => Bound::Included(f(bound)),
                Self::Excluded(bound) => Bound::Excluded(f(bound)),
                Self::Unbounded => Bound::Unbounded,
            }
        }
    }

    impl<A, B> TryFuncMap<A, B> for Bound<A> {
        type Output = Bound<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(match self {
                Self::Included(bound) => Bound::Included(f(bound)?),
                Self::Excluded(bound) => Bound::Excluded(f(bound)?),
                Self::Unbounded => Bound::Unbounded,
            })
        }
    }
}

/// Implementations for [`core::cell::Cell`]
mod cell {
    use crate::{FuncMap, TryFuncMap};

    use core::cell::Cell;

    impl<A, B> FuncMap<A, B> for Cell<A> {
        type Output = Cell<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(self.into_inner()).into()
        }
    }

    impl<A, B> TryFuncMap<A, B> for Cell<A> {
        type Output = Cell<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(self.into_inner())?.into())
        }
    }
}

/// Implementations for [`core::ops::ControlFlow`]
mod control_flow {
    use crate::{FuncMap, TryFuncMap, TypeParam};

    use core::ops::ControlFlow;

    impl<T, U, C> FuncMap<T, U, TypeParam<0>> for ControlFlow<T, C> {
        type Output = ControlFlow<U, C>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(T) -> U,
        {
            match self {
                Self::Break(value) => ControlFlow::Break(f(value)),
                Self::Continue(value) => ControlFlow::Continue(value),
            }
        }
    }

    impl<T, U, C> TryFuncMap<T, U, TypeParam<0>> for ControlFlow<T, C> {
        type Output = ControlFlow<U, C>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(T) -> Result<U, E>,
        {
            Ok(match self {
                Self::Break(value) => ControlFlow::Break(f(value)?),
                Self::Continue(value) => ControlFlow::Continue(value),
            })
        }
    }

    impl<B, T, U> FuncMap<T, U, TypeParam<1>> for ControlFlow<B, T> {
        type Output = ControlFlow<B, U>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(T) -> U,
        {
            match self {
                Self::Break(value) => ControlFlow::Break(value),
                Self::Continue(value) => ControlFlow::Continue(f(value)),
            }
        }
    }

    impl<B, T, U> TryFuncMap<T, U, TypeParam<1>> for ControlFlow<B, T> {
        type Output = ControlFlow<B, U>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(T) -> Result<U, E>,
        {
            Ok(match self {
                Self::Break(value) => ControlFlow::Break(value),
                Self::Continue(value) => ControlFlow::Continue(f(value)?),
            })
        }
    }
}

/// Implementations for [`core::option::Option`]
mod option {
    use crate::{FuncMap, TryFuncMap};

    use core::option;

    impl<A, B> FuncMap<A, B> for Option<A> {
        type Output = Option<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f)
        }
    }

    impl<A, B> TryFuncMap<A, B> for Option<A> {
        type Output = Option<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f).transpose()
        }
    }

    impl<A, B> FuncMap<A, B> for option::IntoIter<A> {
        type Output = option::IntoIter<B>;

        fn func_map<F>(mut self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.next().map(f).into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for option::IntoIter<A> {
        type Output = option::IntoIter<B>;

        fn try_func_map<E, F>(mut self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(self.next().map(f).transpose()?.into_iter())
        }
    }
}

/// Implementations for [`core::marker::PhantomData`]
mod phantom_data {
    use crate::{FuncMap, TryFuncMap};

    use core::marker::PhantomData;

    impl<A, B> FuncMap<A, B> for PhantomData<A> {
        type Output = PhantomData<B>;

        fn func_map<F>(self, _: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            PhantomData
        }
    }

    impl<A, B> TryFuncMap<A, B> for PhantomData<A> {
        type Output = PhantomData<B>;

        fn try_func_map<E, F>(self, _: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(PhantomData)
        }
    }
}

/// Implementations for [`core::task::Poll`]
mod poll {
    use crate::{FuncMap, TryFuncMap};

    use core::task::Poll;

    impl<A, B> FuncMap<A, B> for Poll<A> {
        type Output = Poll<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            match self {
                Self::Ready(value) => Poll::Ready(f(value)),
                Self::Pending => Poll::Pending,
            }
        }
    }

    impl<A, B> TryFuncMap<A, B> for Poll<A> {
        type Output = Poll<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(match self {
                Self::Ready(value) => Poll::Ready(f(value)?),
                Self::Pending => Poll::Pending,
            })
        }
    }
}

/// Implementations for [`core::ops::Range`] etc.
mod range {
    use crate::{FuncMap, TryFuncMap};

    use core::ops::{Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};

    impl<A, B> FuncMap<A, B> for Range<A> {
        type Output = Range<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(self.start)..f(self.end)
        }
    }

    impl<A, B> TryFuncMap<A, B> for Range<A> {
        type Output = Range<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(self.start)?..f(self.end)?)
        }
    }

    impl<A, B> FuncMap<A, B> for RangeFrom<A> {
        type Output = RangeFrom<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(self.start)..
        }
    }

    impl<A, B> TryFuncMap<A, B> for RangeFrom<A> {
        type Output = RangeFrom<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(self.start)?..)
        }
    }

    impl<A, B> FuncMap<A, B> for RangeInclusive<A> {
        type Output = RangeInclusive<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            let (start, end) = self.into_inner();
            f(start)..=f(end)
        }
    }

    impl<A, B> TryFuncMap<A, B> for RangeInclusive<A> {
        type Output = RangeInclusive<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            let (start, end) = self.into_inner();
            Ok(f(start)?..=f(end)?)
        }
    }

    impl<A, B> FuncMap<A, B> for RangeTo<A> {
        type Output = RangeTo<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            ..f(self.end)
        }
    }

    impl<A, B> TryFuncMap<A, B> for RangeTo<A> {
        type Output = RangeTo<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(..f(self.end)?)
        }
    }

    impl<A, B> FuncMap<A, B> for RangeToInclusive<A> {
        type Output = RangeToInclusive<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            ..=f(self.end)
        }
    }

    impl<A, B> TryFuncMap<A, B> for RangeToInclusive<A> {
        type Output = RangeToInclusive<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(..=f(self.end)?)
        }
    }
}

/// Implementations for [`core::cell::RefCell`]
mod ref_cell {
    use crate::{FuncMap, TryFuncMap};

    use core::cell::RefCell;

    impl<A, B> FuncMap<A, B> for RefCell<A> {
        type Output = RefCell<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(self.into_inner()).into()
        }
    }

    impl<A, B> TryFuncMap<A, B> for RefCell<A> {
        type Output = RefCell<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(self.into_inner())?.into())
        }
    }
}

/// Implementations for [`core::result::Result`]
mod result {
    use crate::{FuncMap, TryFuncMap, TypeParam};

    use core::result;

    impl<A, B, U> FuncMap<A, B, TypeParam<0>> for Result<A, U> {
        type Output = Result<B, U>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            match self {
                Ok(value) => Ok(f(value)),
                Err(err) => Err(err),
            }
        }
    }

    impl<A, B, U> TryFuncMap<A, B, TypeParam<0>> for Result<A, U> {
        type Output = Result<B, U>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(match self {
                Ok(value) => Ok(f(value)?),
                Err(err) => Err(err),
            })
        }
    }

    impl<T, A, B> FuncMap<A, B, TypeParam<1>> for Result<T, A> {
        type Output = Result<T, B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            match self {
                Ok(value) => Ok(value),
                Err(err) => Err(f(err)),
            }
        }
    }

    impl<T, A, B> TryFuncMap<A, B, TypeParam<1>> for Result<T, A> {
        type Output = Result<T, B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(match self {
                Ok(value) => Ok(value),
                Err(err) => Err(f(err)?),
            })
        }
    }

    impl<A, B> FuncMap<A, B> for result::IntoIter<A> {
        type Output = result::IntoIter<B>;

        fn func_map<F>(mut self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.next().map(f).ok_or(()).into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for result::IntoIter<A> {
        type Output = result::IntoIter<B>;

        fn try_func_map<E, F>(mut self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(self.next().map(f).transpose()?.ok_or(()).into_iter())
        }
    }
}

/// Implementations for [`core::cell::UnsafeCell`]
mod unsafe_cell {
    use crate::{FuncMap, TryFuncMap};

    use core::cell::UnsafeCell;

    impl<A, B> FuncMap<A, B> for UnsafeCell<A> {
        type Output = UnsafeCell<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(self.into_inner()).into()
        }
    }

    impl<A, B> TryFuncMap<A, B> for UnsafeCell<A> {
        type Output = UnsafeCell<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(self.into_inner())?.into())
        }
    }
}
