#[cfg(all(feature = "rayon", not(target_arch = "wasm32")))]
pub const THREADED: bool = true;
#[cfg(not(all(feature = "rayon", not(target_arch = "wasm32"))))]
pub const THREADED: bool = false;

pub struct Cells<T> {
    ptr: *mut T,
    len: usize,
}

unsafe impl<T: Send> Send for Cells<T> {}
unsafe impl<T: Send> Sync for Cells<T> {}

impl<T> Cells<T> {
    pub fn new(s: &mut [T]) -> Cells<T> {
        Cells { ptr: s.as_mut_ptr(), len: s.len() }
    }

    /// # Safety
    /// No other thread may write element `i` while the returned reference lives.
    pub unsafe fn get(&self, i: usize) -> &T {
        debug_assert!(i < self.len);
        unsafe { &*self.ptr.add(i) }
    }

    /// # Safety
    /// No other thread may read or write element `i` while the returned reference lives.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get_mut(&self, i: usize) -> &mut T {
        debug_assert!(i < self.len);
        unsafe { &mut *self.ptr.add(i) }
    }
}

#[cfg(all(feature = "rayon", not(target_arch = "wasm32")))]
pub fn row_chunks<T: Send, F: Fn(usize, &mut [T]) + Sync + Send>(data: &mut [T], width: usize, rows: usize, f: F) {
    use rayon::prelude::*;
    let step = width * rows.max(1);
    if data.len() < 2 * step {
        f(0, data);
    } else {
        data.par_chunks_mut(step).enumerate().for_each(|(c, s)| f(c * rows.max(1), s));
    }
}

#[cfg(not(all(feature = "rayon", not(target_arch = "wasm32"))))]
pub fn row_chunks<T: Send, F: Fn(usize, &mut [T]) + Sync + Send>(data: &mut [T], _width: usize, _rows: usize, f: F) {
    f(0, data);
}

#[cfg(all(feature = "rayon", not(target_arch = "wasm32")))]
pub fn for_range<F: Fn(usize) + Sync + Send>(n: usize, min_len: usize, f: F) {
    use rayon::prelude::*;
    if n < 2 * min_len.max(1) {
        for i in 0..n {
            f(i);
        }
    } else {
        (0..n).into_par_iter().with_min_len(min_len.max(1)).for_each(f);
    }
}

#[cfg(not(all(feature = "rayon", not(target_arch = "wasm32"))))]
pub fn for_range<F: Fn(usize) + Sync + Send>(n: usize, _min_len: usize, f: F) {
    for i in 0..n {
        f(i);
    }
}
