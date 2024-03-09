//! Helper functions for arrays

use core::{
    mem::{self, MaybeUninit},
    ptr,
};

/// Tries to apply a given closure to every element of a given array, producing
/// a new array of the same length
///
/// This is a fallible version of [`array::map`].
///
/// It can be replaced with [`array::try_map`] once that is stabilized. In fact,
/// the implementation is heavily inspired by the standard library's
/// implementation of [`array::try_map`].
///
/// # Errors
/// Fails if and only if `f` fails, returning the first error according to the
/// order of the elements in `array`
pub(crate) fn try_map<A, B, E, F, const N: usize>(array: [A; N], mut f: F) -> Result<[B; N], E>
where
    F: FnMut(A) -> Result<B, E>,
{
    // This guards the target array, making sure the part of it that has already
    // been filled is dropped if `f` returns `Err(_)` or panics
    struct Guard<'a, T, const N: usize> {
        // mutable borrow of the target array
        array_mut: &'a mut [MaybeUninit<T>; N],

        // index in ..=N up to which (exclusive) `array_mut` is initialized
        init_until_idx: usize,
    }

    impl<T, const N: usize> Drop for Guard<'_, T, N> {
        fn drop(&mut self) {
            // - `self.init_until_idx <= N` is always satisfied
            // - if `self.init_until_idx == N`, the target array is fully
            //   initialized and hence the guard must not be dropped
            debug_assert!(self.init_until_idx < N);

            // SAFETY: as `self.init_until_idx <= N`, the range is within bounds of `self.array_mut`
            let init_slice = unsafe { self.array_mut.get_unchecked_mut(..self.init_until_idx) };

            // SAFETY: by definition of `init_until_idx`, `init_slice` is fully initialized
            let init_slice = unsafe { &mut *(init_slice as *mut [MaybeUninit<T>]).cast::<T>() };

            // SAFETY:
            // - `init_slice` is valid for dropping
            // - `self.array_mut` (and hence `init_slice`) is not used after `self` is dropped
            unsafe { ptr::drop_in_place(init_slice) };
        }
    }

    if N == 0 {
        // SAFETY: an empty array is zero-sized and hence has no invalid bit patterns
        return Ok(unsafe { mem::zeroed() });
    }

    // This can be replaced with a call to `MaybeUninit::uninit_array` once that is stabilized
    //
    // SAFETY: an array of `MaybeUninit<_>` is always initialized
    let mut mapped: [MaybeUninit<B>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut guard = Guard {
        array_mut: &mut mapped,
        init_until_idx: 0,
    };

    for value in array {
        // SAFETY: the iterator yields exactly `N` elements,
        // so `guard.init_until_idx` has been increased at most `N - 1` times
        // and hence is within bounds of `guard.array_mut`
        unsafe {
            guard
                .array_mut
                .get_unchecked_mut(guard.init_until_idx)
                // if `f` returns `Err(_)` or panics, then `guard` is dropped
                .write(f(value)?);
        }

        guard.init_until_idx += 1;
    }

    // now `guard.init_until_idx == N` and the target array is fully initialized,
    // so make sure the guard isn't dropped
    mem::forget(guard);

    // SAFETY: `mapped` is fully initialized
    let mapped = unsafe { ptr::addr_of!(mapped).cast::<[B; N]>().read() };

    Ok(mapped)
}

#[cfg(test)]
mod tests {
    use super::*;

    mod correctness {
        use super::*;

        #[test]
        fn mapping_empty_array_succeeds_even_when_function_always_fails() {
            let array: [T1; 0] = [];

            let result: Result<[T2; 0], _> = try_map(array, |_| Err(MappingError));

            assert!(result.is_ok());
        }

        #[test]
        fn mapping_non_empty_array_succeeds_when_function_succeeds() {
            let array = [T1::Mappable, T1::Mappable, T1::Mappable];

            let result: Result<[T2; 3], _> = try_map(array, TryInto::try_into);

            assert_eq!(result, Ok([T2, T2, T2]));
        }

        #[test]
        fn mapping_non_empty_array_fails_with_first_error_when_function_fails() {
            let array = [
                T1::NotMappable("First Error"),
                T1::Mappable,
                T1::NotMappable("Second Error"),
            ];

            let result: Result<[T2; 3], _> = try_map(array, TryInto::try_into);

            assert_eq!(result, Err(MappingError("First Error")));
        }

        #[derive(Debug, PartialEq)]
        enum T1 {
            Mappable,
            NotMappable(&'static str),
        }

        #[derive(Debug, PartialEq)]
        struct T2;

        impl TryFrom<T1> for T2 {
            type Error = MappingError;

            fn try_from(value: T1) -> Result<Self, Self::Error> {
                match value {
                    T1::Mappable => Ok(T2),
                    T1::NotMappable(message) => Err(MappingError(message)),
                }
            }
        }

        #[derive(Debug, PartialEq)]
        struct MappingError(&'static str);
    }

    mod dropping {
        use super::*;
        use drop_trace::*;

        use std::panic::{self, AssertUnwindSafe};

        #[test]
        fn all_elements_are_dropped_when_all_are_mapped_successfully() {
            let drop_trace = DropTrace::new();

            let array = [
                Item::new("Mappable 1", MappingState::Mappable, &drop_trace),
                Item::new("Mappable 2", MappingState::Mappable, &drop_trace),
                Item::new("Mappable 3", MappingState::Mappable, &drop_trace),
            ];

            let result = try_map(array, Item::map);

            assert!(result.is_ok());
            drop(result);

            assert_eq!(
                drop_trace.into_iter().collect::<Vec<_>>(),
                ["Mappable 1", "Mappable 2", "Mappable 3"]
            );
        }

        #[test]
        fn all_elements_are_dropped_when_mapping_for_some_fails() {
            let drop_trace = DropTrace::new();

            let array = [
                Item::new("Mappable 1", MappingState::Mappable, &drop_trace),
                Item::new("Not Mappable", MappingState::NotMappable, &drop_trace),
                Item::new("Mappable 2", MappingState::Mappable, &drop_trace),
            ];

            let result = try_map(array, Item::map);

            assert!(result.is_err());
            drop(result);

            assert_eq!(
                drop_trace.into_iter().collect::<Vec<_>>(),
                ["Not Mappable", "Mappable 2", "Mappable 1"]
            );
        }

        #[test]
        fn all_elements_are_dropped_when_mapping_for_some_panics() {
            let drop_trace = DropTrace::new();

            let array = [
                Item::new("Mappable 1", MappingState::Mappable, &drop_trace),
                Item::new("Mapping Panics", MappingState::MappingPanics, &drop_trace),
                Item::new("Mappable 2", MappingState::Mappable, &drop_trace),
            ];

            assert_panic(|| try_map(array, Item::map));

            assert_eq!(
                drop_trace.into_iter().collect::<Vec<_>>(),
                ["Mapping Panics", "Mappable 2", "Mappable 1"]
            );
        }

        #[derive(Debug)]
        enum MappingState {
            Mappable,
            MappingPanics,
            NotMappable,
        }

        #[derive(Debug)]
        struct Item<'a, T> {
            mapping_state: MappingState,
            _guard: DropTraceGuard<'a, T>,
        }

        impl<'a, T> Item<'a, T> {
            fn new(payload: T, mapping_state: MappingState, drop_trace: &'a DropTrace<T>) -> Self {
                Self {
                    mapping_state,
                    _guard: drop_trace.guard(payload),
                }
            }

            fn map(self) -> Result<Self, MappingError> {
                match self.mapping_state {
                    MappingState::Mappable => Ok(Self {
                        mapping_state: MappingState::MappingPanics,
                        ..self
                    }),
                    MappingState::MappingPanics => panic!("panic during mapping"),
                    MappingState::NotMappable => Err(MappingError),
                }
            }
        }

        #[derive(Debug)]
        struct MappingError;

        fn assert_panic<R>(f: impl FnOnce() -> R) {
            panic::set_hook(Box::new(|_| {}));
            let result = panic::catch_unwind(AssertUnwindSafe(f));
            let _ = panic::take_hook();
            assert!(result.is_err());
        }

        mod drop_trace {
            use std::cell::RefCell;
            use std::vec;

            #[derive(Debug)]
            pub(super) struct DropTrace<T>(RefCell<Vec<T>>);

            impl<T> DropTrace<T> {
                pub(super) fn new() -> Self {
                    Self::default()
                }

                pub(super) fn guard(&self, payload: T) -> DropTraceGuard<T> {
                    DropTraceGuard {
                        drop_trace: self,
                        payload: Some(payload),
                    }
                }
            }

            impl<T> Default for DropTrace<T> {
                fn default() -> Self {
                    Self(RefCell::new(Vec::new()))
                }
            }

            impl<T> IntoIterator for DropTrace<T> {
                type Item = T;
                type IntoIter = vec::IntoIter<T>;

                fn into_iter(self) -> Self::IntoIter {
                    self.0.into_inner().into_iter()
                }
            }

            #[derive(Debug)]
            pub(super) struct DropTraceGuard<'a, T> {
                drop_trace: &'a DropTrace<T>,
                payload: Option<T>,
            }

            impl<T> Drop for DropTraceGuard<'_, T> {
                fn drop(&mut self) {
                    self.drop_trace
                        .0
                        .borrow_mut()
                        .push(self.payload.take().unwrap());
                }
            }
        }
    }
}
