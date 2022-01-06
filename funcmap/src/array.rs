use core::{
    mem::{self, MaybeUninit},
    ptr,
};

pub(crate) fn try_map_array<A, B, E, F, const N: usize>(
    array: [A; N],
    mut f: F,
) -> Result<[B; N], E>
where
    F: FnMut(A) -> Result<B, E>,
{
    // invariants:
    // - `init_until_idx <= N`
    // - the slice `array_mut[..init_until_idx]` is initialized
    struct Guard<'a, T, const N: usize> {
        array_mut: &'a mut [MaybeUninit<T>; N],
        init_until_idx: usize,
    }

    impl<T, const N: usize> Drop for Guard<'_, T, N> {
        fn drop(&mut self) {
            // `self.init_until_idx <= N` is an invariant of `Self`
            // if `self.init_until_idx == N`, then `self` must not be dropped
            debug_assert!(self.init_until_idx < N);

            // SAFETY: as `self.init_until_idx <= N` by invariant, the range is within bounds of `self.array_mut`
            let init_slice = unsafe { self.array_mut.get_unchecked_mut(..self.init_until_idx) };

            // SAFETY: the slice is initialized by invariant
            let init_slice = unsafe { &mut *(init_slice as *mut [MaybeUninit<T>]).cast::<T>() };

            // SAFETY: `self.array_mut` is not used after `Self` is dropped
            unsafe { ptr::drop_in_place(init_slice) };
        }
    }

    if N == 0 {
        // SAFETY: an empty array is zero-sized and hence has no invalid bit patterns
        return Ok(unsafe { mem::zeroed() });
    }

    // SAFETY: an array of `MaybeUninit<_>` is always initialized
    let mut mapped: [MaybeUninit<B>; N] = unsafe { MaybeUninit::uninit().assume_init() };

    let mut guard = Guard {
        array_mut: &mut mapped,
        init_until_idx: 0,
    };

    for value in array {
        // SAFETY: `guard.init_until_idx` is within bounds of `guard.array_mut` because the iterator yields exactly `N` elements
        // and hence `guard.init_until_idx` has been increased at most `N - 1` times
        unsafe {
            guard
                .array_mut
                .get_unchecked_mut(guard.init_until_idx)
                // if `f` panics or returns `Err(_)`, then `guard` is dropped
                .write(f(value)?);
        }

        guard.init_until_idx += 1;
    }

    // now `guard.init_until_idx == N` and all elemens are initialized, so don't drop the guard
    mem::forget(guard);

    // SAFETY: all elements are initialized at this point
    let mapped = unsafe {
        (&mapped as *const [MaybeUninit<B>; N])
            .cast::<[B; N]>()
            .read()
    };

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

            let result: Result<[T2; 0], _> = try_map_array(array, |_| Err(MappingError));

            assert!(result.is_ok());
        }

        #[test]
        fn mapping_non_empty_array_succeeds_when_function_succeeds() {
            let array = [T1::Mappable, T1::Mappable, T1::Mappable];

            let result: Result<[T2; 3], _> = try_map_array(array, TryInto::try_into);

            assert_eq!(result, Ok([T2, T2, T2]));
        }

        #[test]
        fn mapping_non_empty_array_fails_with_first_error_when_function_fails() {
            let array = [
                T1::NotMappable("First Error"),
                T1::Mappable,
                T1::NotMappable("Second Error"),
            ];

            let result: Result<[T2; 3], _> = try_map_array(array, TryInto::try_into);

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
        extern crate std;

        use super::*;
        use drop_trace::*;

        use std::boxed::Box;
        use std::panic::{self, AssertUnwindSafe};
        use std::vec::Vec;

        #[test]
        fn all_elements_are_dropped_when_all_are_mapped_successfully() {
            let drop_trace = DropTrace::new();

            let array = [
                Item::new("Mappable 1", MappingState::Mappable, &drop_trace),
                Item::new("Mappable 2", MappingState::Mappable, &drop_trace),
                Item::new("Mappable 3", MappingState::Mappable, &drop_trace),
            ];

            let result = try_map_array(array, Item::map);

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

            let result = try_map_array(array, Item::map);

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

            assert_panic(|| try_map_array(array, Item::map));

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

            fn with_mapping_state(self, mapping_state: MappingState) -> Self {
                Self {
                    mapping_state,
                    ..self
                }
            }

            fn map(self) -> Result<Self, MappingError> {
                match self.mapping_state {
                    MappingState::Mappable => {
                        Ok(self.with_mapping_state(MappingState::MappingPanics))
                    }
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
            extern crate std;

            use std::cell::RefCell;
            use std::vec::{self, Vec};

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
