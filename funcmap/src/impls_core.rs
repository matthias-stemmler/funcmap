use crate::array::try_map_array;
use crate::{FuncMap, TypeParam};

use core::cell::{Cell, RefCell, UnsafeCell};
use core::marker::PhantomData;
use core::ops::{Bound, ControlFlow, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use core::task::Poll;
use core::{option, result};

impl<A, B, const N: usize> FuncMap<A, B> for [A; N] {
    type Output = [B; N];

    fn try_func_map<F, E>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        try_map_array(self, f)
    }
}

impl<A, B> FuncMap<A, B> for Bound<A> {
    type Output = Bound<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
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

impl<A, B> FuncMap<A, B> for Cell<A> {
    type Output = Cell<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(self.into_inner())?.into())
    }
}

impl<T, U, C> FuncMap<T, U, TypeParam<0>> for ControlFlow<T, C> {
    type Output = ControlFlow<U, C>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
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

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(T) -> Result<U, E>,
    {
        Ok(match self {
            Self::Break(value) => ControlFlow::Break(value),
            Self::Continue(value) => ControlFlow::Continue(f(value)?),
        })
    }
}

impl<A, B> FuncMap<A, B> for Option<A> {
    type Output = Option<B>;

    fn try_func_map<F, E>(self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        self.map(f).transpose()
    }
}

impl<A, B> FuncMap<A, B> for option::IntoIter<A> {
    type Output = option::IntoIter<B>;

    fn try_func_map<F, E>(mut self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(self.next().map(f).transpose()?.into_iter())
    }
}

impl<A, B> FuncMap<A, B> for PhantomData<A> {
    type Output = PhantomData<B>;

    fn try_func_map<F, E>(self, _: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(PhantomData)
    }
}

impl<A, B> FuncMap<A, B> for Poll<A> {
    type Output = Poll<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(match self {
            Self::Ready(value) => Poll::Ready(f(value)?),
            Self::Pending => Poll::Pending,
        })
    }
}

impl<A, B> FuncMap<A, B> for Range<A> {
    type Output = Range<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(self.start)?..f(self.end)?)
    }
}

impl<A, B> FuncMap<A, B> for RangeFrom<A> {
    type Output = RangeFrom<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(self.start)?..)
    }
}

impl<A, B> FuncMap<A, B> for RangeInclusive<A> {
    type Output = RangeInclusive<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        let (start, end) = self.into_inner();
        Ok(f(start)?..=f(end)?)
    }
}

impl<A, B> FuncMap<A, B> for RangeTo<A> {
    type Output = RangeTo<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(..f(self.end)?)
    }
}

impl<A, B> FuncMap<A, B> for RangeToInclusive<A> {
    type Output = RangeToInclusive<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(..=f(self.end)?)
    }
}

impl<A, B> FuncMap<A, B> for RefCell<A> {
    type Output = RefCell<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(self.into_inner())?.into())
    }
}

impl<A, B, U> FuncMap<A, B, TypeParam<0>> for Result<A, U> {
    type Output = Result<B, U>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
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

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
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

    fn try_func_map<F, E>(mut self, f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(self.next().map(f).transpose()?.ok_or(()).into_iter())
    }
}

impl<A, B> FuncMap<A, B> for UnsafeCell<A> {
    type Output = UnsafeCell<B>;

    fn try_func_map<F, E>(self, mut f: F) -> Result<Self::Output, E>
    where
        F: FnMut(A) -> Result<B, E>,
    {
        Ok(f(self.into_inner())?.into())
    }
}
