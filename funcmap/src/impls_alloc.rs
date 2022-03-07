//! Implementations of [`FuncMap`](crate::FuncMap) and
//! [`TryFuncMap`](crate::TryFuncMap) for types in [`alloc`]

/// Implementations for types in [`alloc::collections::binary_heap`]
mod binary_heap {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::collections::{binary_heap, BinaryHeap};

    impl<A, B> FuncMap<A, B> for BinaryHeap<A>
    where
        B: Ord,
    {
        type Output = BinaryHeap<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> TryFuncMap<A, B> for BinaryHeap<A>
    where
        B: Ord,
    {
        type Output = BinaryHeap<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> FuncMap<A, B> for binary_heap::IntoIter<A>
    where
        B: Ord,
    {
        type Output = binary_heap::IntoIter<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<BinaryHeap<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for binary_heap::IntoIter<A>
    where
        B: Ord,
    {
        type Output = binary_heap::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<BinaryHeap<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`alloc::boxed`]
mod boxed {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::boxed::Box;

    impl<A, B> FuncMap<A, B> for Box<A> {
        type Output = Box<B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            f(*self).into()
        }
    }

    impl<A, B> TryFuncMap<A, B> for Box<A> {
        type Output = Box<B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            Ok(f(*self)?.into())
        }
    }
}

/// Implementations for types in [`alloc::collections::btree_map`]
mod btree_map {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap, TypeParam};

    use alloc::collections::{btree_map, BTreeMap};

    impl<A, B, V> FuncMap<A, B, TypeParam<0>> for BTreeMap<A, V>
    where
        B: Ord,
    {
        type Output = BTreeMap<B, V>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(|(k, v)| (f(k), v)).collect()
        }
    }

    impl<A, B, V> TryFuncMap<A, B, TypeParam<0>> for BTreeMap<A, V>
    where
        B: Ord,
    {
        type Output = BTreeMap<B, V>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(|(k, v)| Ok((f(k)?, v))).collect()
        }
    }

    impl<K, A, B> FuncMap<A, B, TypeParam<1>> for BTreeMap<K, A>
    where
        K: Ord,
    {
        type Output = BTreeMap<K, B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(|(k, v)| (k, f(v))).collect()
        }
    }

    impl<K, A, B> TryFuncMap<A, B, TypeParam<1>> for BTreeMap<K, A>
    where
        K: Ord,
    {
        type Output = BTreeMap<K, B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(|(k, v)| Ok((k, f(v)?))).collect()
        }
    }

    impl<A, B, V> FuncMap<A, B, TypeParam<0>> for btree_map::IntoIter<A, V>
    where
        B: Ord,
    {
        type Output = btree_map::IntoIter<B, V>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(|(k, v)| (f(k), v))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
        }
    }

    impl<A, B, V> TryFuncMap<A, B, TypeParam<0>> for btree_map::IntoIter<A, V>
    where
        B: Ord,
    {
        type Output = btree_map::IntoIter<B, V>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(|(k, v)| Ok((f(k)?, v)))
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map(IntoIterator::into_iter)
        }
    }

    impl<K, A, B> FuncMap<A, B, TypeParam<1>> for btree_map::IntoIter<K, A>
    where
        K: Ord,
    {
        type Output = btree_map::IntoIter<K, B>;

        fn func_map<F>(self, mut f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(|(k, v)| (k, f(v)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
        }
    }

    impl<K, A, B> TryFuncMap<A, B, TypeParam<1>> for btree_map::IntoIter<K, A>
    where
        K: Ord,
    {
        type Output = btree_map::IntoIter<K, B>;

        fn try_func_map<E, F>(self, mut f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(|(k, v)| Ok((k, f(v)?)))
                .collect::<Result<BTreeMap<_, _>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`alloc::collections::btree_set`]
mod btree_set {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::collections::{btree_set, BTreeSet};

    impl<A, B> FuncMap<A, B> for BTreeSet<A>
    where
        B: Ord,
    {
        type Output = BTreeSet<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> TryFuncMap<A, B> for BTreeSet<A>
    where
        B: Ord,
    {
        type Output = BTreeSet<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> FuncMap<A, B> for btree_set::IntoIter<A>
    where
        B: Ord,
    {
        type Output = btree_set::IntoIter<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<BTreeSet<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for btree_set::IntoIter<A>
    where
        B: Ord,
    {
        type Output = btree_set::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<BTreeSet<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`alloc::collections::linked_list`]
mod linked_list {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::collections::{linked_list, LinkedList};

    impl<A, B> FuncMap<A, B> for LinkedList<A> {
        type Output = LinkedList<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> TryFuncMap<A, B> for LinkedList<A> {
        type Output = LinkedList<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> FuncMap<A, B> for linked_list::IntoIter<A> {
        type Output = linked_list::IntoIter<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<LinkedList<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for linked_list::IntoIter<A> {
        type Output = linked_list::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<LinkedList<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`alloc::vec`](mod@alloc::vec)
mod vec {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::vec;

    impl<A, B> FuncMap<A, B> for Vec<A> {
        type Output = Vec<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> TryFuncMap<A, B> for Vec<A> {
        type Output = Vec<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> FuncMap<A, B> for vec::IntoIter<A> {
        type Output = vec::IntoIter<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<Vec<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for vec::IntoIter<A> {
        type Output = vec::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<Vec<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}

/// Implementations for types in [`alloc::collections::vec_deque`]
mod vec_deque {
    extern crate alloc;

    use crate::{FuncMap, TryFuncMap};

    use alloc::collections::{vec_deque, VecDeque};

    impl<A, B> FuncMap<A, B> for VecDeque<A> {
        type Output = VecDeque<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> TryFuncMap<A, B> for VecDeque<A> {
        type Output = VecDeque<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.into_iter().map(f).collect()
        }
    }

    impl<A, B> FuncMap<A, B> for vec_deque::IntoIter<A> {
        type Output = vec_deque::IntoIter<B>;

        fn func_map<F>(self, f: F) -> Self::Output
        where
            F: FnMut(A) -> B,
        {
            self.map(f).collect::<VecDeque<_>>().into_iter()
        }
    }

    impl<A, B> TryFuncMap<A, B> for vec_deque::IntoIter<A> {
        type Output = vec_deque::IntoIter<B>;

        fn try_func_map<E, F>(self, f: F) -> Result<Self::Output, E>
        where
            F: FnMut(A) -> Result<B, E>,
        {
            self.map(f)
                .collect::<Result<VecDeque<_>, _>>()
                .map(IntoIterator::into_iter)
        }
    }
}
