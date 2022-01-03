use crate::{FuncMap, TypeParam};

use core::cell::{Cell, RefCell, UnsafeCell};
use core::cmp::Reverse;
use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::num::Wrapping;
use core::ops::{Bound, ControlFlow, Range, RangeFrom, RangeInclusive, RangeTo, RangeToInclusive};
use core::panic::AssertUnwindSafe;
use core::task::Poll;
use core::{option, result};

impl<A, B, const N: usize> FuncMap<A, B> for [A; N] {
    type Output = [B; N];

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B> FuncMap<A, B> for AssertUnwindSafe<A> {
    type Output = AssertUnwindSafe<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        AssertUnwindSafe(f(self.0))
    }
}

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

impl<A, B> FuncMap<A, B> for Cell<A> {
    type Output = Cell<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        f(self.into_inner()).into()
    }
}

impl<D, E, C> FuncMap<D, E, TypeParam<0>> for ControlFlow<D, C> {
    type Output = ControlFlow<E, C>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(D) -> E,
    {
        match self {
            Self::Break(value) => ControlFlow::Break(f(value)),
            Self::Continue(value) => ControlFlow::Continue(value),
        }
    }
}

impl<B, D, E> FuncMap<D, E, TypeParam<1>> for ControlFlow<B, D> {
    type Output = ControlFlow<B, E>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(D) -> E,
    {
        match self {
            Self::Break(value) => ControlFlow::Break(value),
            Self::Continue(value) => ControlFlow::Continue(f(value)),
        }
    }
}

impl<A, B> FuncMap<A, B> for ManuallyDrop<A> {
    type Output = ManuallyDrop<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        ManuallyDrop::new(f(Self::into_inner(self)))
    }
}

impl<A, B> FuncMap<A, B> for Option<A> {
    type Output = Option<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
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

impl<A, B> FuncMap<A, B> for PhantomData<A> {
    type Output = PhantomData<B>;

    fn func_map<F>(self, _: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        PhantomData
    }
}

impl<A, B> FuncMap<A, B> for Poll<A> {
    type Output = Poll<B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<A, B> FuncMap<A, B> for Range<A> {
    type Output = Range<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        f(self.start)..f(self.end)
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

impl<A, B> FuncMap<A, B> for RangeTo<A> {
    type Output = RangeTo<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        ..f(self.end)
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

impl<A, B> FuncMap<A, B> for RefCell<A> {
    type Output = RefCell<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        RefCell::new(f(self.into_inner()))
    }
}

impl<A, B, E> FuncMap<A, B, TypeParam<0>> for Result<A, E> {
    type Output = Result<B, E>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map(f)
    }
}

impl<T, A, B> FuncMap<A, B, TypeParam<1>> for Result<T, A> {
    type Output = Result<T, B>;

    fn func_map<F>(self, f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        self.map_err(f)
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

impl<A, B> FuncMap<A, B> for Reverse<A> {
    type Output = Reverse<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Reverse(f(self.0))
    }
}

impl<A, B> FuncMap<A, B> for UnsafeCell<A> {
    type Output = UnsafeCell<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        UnsafeCell::new(f(self.into_inner()))
    }
}

impl<A, B> FuncMap<A, B> for Wrapping<A> {
    type Output = Wrapping<B>;

    fn func_map<F>(self, mut f: F) -> Self::Output
    where
        F: FnMut(A) -> B,
    {
        Wrapping(f(self.0))
    }
}
