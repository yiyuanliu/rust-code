use std::alloc::{alloc, dealloc, handle_alloc_error, realloc, Layout};
use std::marker::PhantomData;
use std::mem;
use std::ops::{Deref, DerefMut};
use std::ptr;

struct Vec<T> {
    data: RawVec<T>,
    len: usize,
}

struct Unique<T> {
    ptr: *const T,
    _marker: PhantomData<T>,
}

impl<T> Unique<T> {
    fn dangling() -> Self {
        Self::new_unchecked(mem::align_of::<T>() as *mut _)
    }

    fn new_unchecked(ptr: *mut T) -> Self {
        Self {
            ptr: ptr as _,
            _marker: PhantomData,
        }
    }

    fn as_ptr(&self) -> *mut T {
        self.ptr as _
    }
}

struct RawVec<T> {
    ptr: Unique<T>,
    cap: usize,
}

impl<T> RawVec<T> {
    fn new() -> Self {
        Self::with_capacity(0)
    }

    fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            Self {
                ptr: Unique::dangling(),
                cap: 0,
            }
        } else {
            let layout = Layout::array::<T>(cap).unwrap();
            let ptr = unsafe { alloc(layout) };
            if ptr.is_null() {
                handle_alloc_error(layout);
            }
            Self {
                ptr: Unique::new_unchecked(ptr as _),
                cap,
            }
        }
    }

    fn cap(&self) -> usize {
        if mem::size_of::<T>() == 0 {
            usize::MAX
        } else {
            self.cap
        }
    }

    fn reserve(&mut self, target_cap: usize) {
        if target_cap <= self.cap {
            return;
        }

        if self.cap == 0 {
            *self = Self::with_capacity(target_cap);
        } else {
            let layout = Layout::array::<T>(target_cap).unwrap();
            let ptr = unsafe { realloc(self.as_ptr() as _, layout, layout.size()) };
            if ptr.is_null() {
                handle_alloc_error(layout);
            }

            self.ptr = Unique::new_unchecked(ptr as _);
            self.cap = target_cap;
        }
    }

    fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    fn shrink(&mut self, _target_cap: usize) {
        todo!()
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        // todo: add may dangle
        if self.cap > 0 {
            unsafe {
                dealloc(
                    self.ptr.as_ptr() as _,
                    Layout::array::<T>(self.cap).unwrap(),
                );
            }
        }
    }
}

impl<T> Vec<T> {
    fn new() -> Self {
        Self {
            data: RawVec::new(),
            len: 0,
        }
    }

    fn with_capacity(cap: usize) -> Self {
        if cap == 0 {
            Self::new()
        } else {
            Self {
                data: RawVec::with_capacity(cap),
                len: 0,
            }
        }
    }

    fn as_ptr(&self) -> *mut T {
        self.data.as_ptr()
    }

    fn len(&self) -> usize {
        self.len
    }

    fn reserve(&mut self, cap: usize) {
        self.data.reserve(cap);
    }

    fn push(&mut self, val: T) {
        if self.data.cap() < self.len + 1 {
            assert_eq!(self.data.cap(), self.len);
            self.data.reserve(std::cmp::max(2 * self.len, self.len + 1));
        }

        unsafe {
            ptr::write(self.as_ptr().add(self.len), val);
        }
        self.len += 1;
    }

    fn insert(&mut self, idx: usize, val: T) {
        if self.data.cap() < self.len + 1 {
            assert_eq!(self.data.cap(), self.len);
            self.data.reserve(std::cmp::max(2 * self.len, self.len + 1));
        }

        // this can be optimized...
        unsafe {
            println!("copy {}", self.len() - idx);
            ptr::copy(
                self.as_ptr().add(idx),
                self.as_ptr().add(idx + 1),
                self.len - idx,
            );
            ptr::write(self.as_ptr().add(idx), val);
        }
        self.len += 1;
    }

    fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            unsafe { Some(ptr::read(self.as_ptr().add(self.len))) }
        }
    }

    fn remove(&mut self, idx: usize) {
        unsafe {
            ptr::drop_in_place(self.as_ptr().add(idx));
            ptr::copy(
                self.as_ptr().add(idx + 1),
                self.as_ptr().add(idx),
                self.len - idx - 1,
            );
        }
        self.len -= 1;
    }

    fn get(&self, idx: usize) -> Option<&T> {
        if self.len <= idx {
            None
        } else {
            unsafe { Some(&*self.as_ptr().add(self.len)) }
        }
    }

    fn get_mut(&mut self, idx: usize) -> Option<&mut T> {
        // what if we map get_mut to get?
        if self.len <= idx {
            None
        } else {
            unsafe { Some(&mut *self.as_ptr().add(self.len)) }
        }
    }
}

impl<T> Drop for Vec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.len {
                ptr::drop_in_place(self.as_ptr().add(i));
            }
        }
    }
}

impl<T> Deref for Vec<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        unsafe { std::slice::from_raw_parts(self.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for Vec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { std::slice::from_raw_parts_mut(self.as_ptr(), self.len) }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::random;

    fn rand_op(my_vec: &mut Vec<i32>, std_vec: &mut std::vec::Vec<i32>) {
        let op = random::<i32>() % 5;
        match op {
            0 => {
                // push
                let val = random();
                my_vec.push(val);
                std_vec.push(val);
                println!("push {}", val);
            }
            1 => {
                // pop
                assert_eq!(my_vec.pop(), std_vec.pop());
                println!("pop");
            }
            2 => {
                // insert
                let idx = random::<usize>() % (my_vec.len() + 1);
                let val = random::<i32>();
                my_vec.insert(idx, val);
                std_vec.insert(idx, val);
                println!("insert {} {}", idx, val);
            }
            3 => {
                // get with slice
                if my_vec.len() > 0 {
                    let idx = random::<usize>() % my_vec.len();
                    assert_eq!(my_vec[idx], std_vec[idx]);
                }
            }
            4 => {
                // put with slice
                if my_vec.len() > 0 {
                    let idx = random::<usize>() % my_vec.len();
                    assert_eq!(my_vec[idx], std_vec[idx]);
                    let val = random();
                    my_vec[idx] = val;
                    std_vec[idx] = val;
                }
            }
            _ => {
                // remove
                if my_vec.len() != 0 {
                    let idx = random::<usize>() % my_vec.len();
                    my_vec.remove(idx);
                    std_vec.remove(idx);
                    println!("remove {}", idx);
                }
            }
        }
        assert_eq!(&my_vec[..], &std_vec[..]);
    }

    #[test]
    fn test() {
        let mut my_vec = Vec::new();
        let mut std_vec = std::vec::Vec::new();

        for _ in 0..10000 {
            rand_op(&mut my_vec, &mut std_vec);
        }
    }

    #[test]
    fn test_with_capacity() {
        let mut my_vec = Vec::with_capacity(100);
        let mut std_vec = std::vec::Vec::new();
        std_vec.iter();

        for _ in 0..10000 {
            rand_op(&mut my_vec, &mut std_vec);
        }
    }
}
