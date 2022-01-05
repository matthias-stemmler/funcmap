use core::{
    mem::{self, MaybeUninit},
    ptr,
};

pub fn try_map_array<A, B, E, F, const N: usize>(array: [A; N], mut f: F) -> Result<[B; N], E>
where
    F: FnMut(A) -> Result<B, E>,
{
    if N == 0 {
        // SAFETY: an empty array is zero-sized and hence has no invalid bit patterns
        return Ok(unsafe { mem::zeroed() });
    }

    // SAFETY: an array of `MaybeUninit<_>` is always initialized
    let mut mapped: [MaybeUninit<B>; N] = unsafe { MaybeUninit::uninit().assume_init() };

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
            let init_slice = unsafe { &mut *(init_slice as *mut _ as *mut T) };

            // SAFETY: `self.array_mut` is not used after `Self` is dropped
            unsafe { ptr::drop_in_place(init_slice) };
        }
    }

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
    let mapped = unsafe { (&mapped as *const _ as *const [B; N]).read() };

    Ok(mapped)
}
