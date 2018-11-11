use core::alloc::{Alloc, Layout, AllocErr};
use core::cmp::Ordering;
use core::fmt::{self, Debug};
use core::mem;
use core::ptr::{self, NonNull};
use std::alloc::Global;

pub struct BinaryHeap<T: Ord, A: Alloc = Global> {
    vec: RawVec<T, A>,
    len: usize,
}


impl<T: Ord> BinaryHeap<T, Global> {
    pub fn new() -> Self {
        Self::new_in(Global)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global)
    }
}

impl<T: Ord, A: Alloc> BinaryHeap<T, A> {
    pub fn new_in(a: A) -> Self {
        Self {
            vec: RawVec::new_in(a),
            len: 0
        }
    }

    pub fn with_capacity_in(cap: usize, a: A) -> Self {
        Self {
            vec: RawVec::with_capacity_in(cap, a),
            len: 0
        }
    }
    
    pub fn cap(&self) -> usize {
        self.vec.cap()
    } 

    pub fn as_mut_ptr(&self) -> *mut T {
        self.vec.as_mut_ptr()
    }
    
    pub fn alloc(&self) -> &A {
        self.vec.alloc()
    }

    pub fn alloc_mut(&mut self) -> &mut A {
        self.vec.alloc_mut()
    }

    pub fn len(&self) -> usize {
        self.len 
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

impl<T: Ord, A: Alloc> BinaryHeap<T, A> {
    #[inline]
    fn swap(&mut self, a: usize, b: usize) {
        unsafe {
            let a_ptr = self.as_mut_ptr().add(a);
            let b_ptr = self.as_mut_ptr().add(b);
            ptr::swap_nonoverlapping(a_ptr, b_ptr, mem::size_of::<T>());
        }
    }

    #[inline]
    fn cmp_at(&self, a: usize, b: usize) -> Ordering {
        unsafe {
            let a = &ptr::read(self.as_mut_ptr().add(a));
            let b = &ptr::read(self.as_mut_ptr().add(b));
            a.cmp(b)
        }
    }

    #[inline]
    fn set(&mut self, index: usize, value: T) {
        unsafe {
            let ptr = self.as_mut_ptr().add(index);
            ptr::write(ptr, value);
        }
    }

    #[inline]
    fn get_ref_at(&self, index: usize) -> &T {
        unsafe {
            &*self.as_mut_ptr().add(index)
        }
    }

    #[inline]
    fn get_at(&self, index: usize) -> T {
        unsafe {
            ptr::read(self.as_mut_ptr().add(index))
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if self.is_empty() {
            None
        } else {
            Some(self.get_ref_at(0))
        }
    }

    pub fn push(&mut self, value: T) {
        if self.is_empty() || self.len() == self.cap() {
            self.vec.double();
        }
        self.set(self.len, value);
        let mut cur = self.len;
        while cur > 0 {
            let fa = (cur - 1) >> 1;
            if self.cmp_at(cur, fa) != Ordering::Less {
                break;
            }
            self.swap(cur, fa);
            cur = fa;
        }
        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }
        let ans = self.get_at(0);
        self.len -= 1;
        self.swap(0, self.len);
        let mut cur = 0;
        while cur < self.len {
            let mut nxt = (cur << 1) + 1;
            if nxt + 1 < self.len && self.cmp_at(nxt + 1, nxt) == Ordering::Less {
                nxt += 1;
            }
            if nxt >= self.len || self.cmp_at(nxt, cur) != Ordering::Less {
                break;    
            }
            self.swap(nxt, cur);
            cur = nxt;
        }
        Some(ans)
    }
}

struct RawVec<T: Ord, A: Alloc = Global> {
    ptr: *mut T,
    cap: usize,
    a: A,
}

const DEFAULT_CAPACITY: usize = 4;

impl<T: Ord, A: Alloc> RawVec<T, A> {

    pub fn cap(&self) -> usize {
        self.cap
    }

    pub fn new_in(a: A) -> Self {
        Self {
            ptr: ptr::null_mut(),
            cap: 0,
            a, 
        }
    }

    pub fn with_capacity_in(cap: usize, a: A) -> Self {
        Self::allocate_in(cap, a)
    }

    fn allocate_in(cap: usize, mut a: A) -> Self {
        unsafe {
            let elem_size = mem::size_of::<T>();
            let alloc_size = cap.checked_mul(elem_size)
                .expect("Capacity overflow!");
            let ptr = if alloc_size == 0 {
                ptr::null_mut()
            } else {
                let align = mem::align_of::<T>();
                let layout = Layout::from_size_align(alloc_size, align).unwrap();
                let ptr = a.alloc(layout).unwrap();
                ptr.cast().as_ptr()
            };
            Self {
                ptr,
                cap,
                a
            }
        }
    }

    pub fn double(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<T>();
            let (new_cap, new_ptr) = match self.current_layout() {
                Some(cur_layout) => {
                    let new_cap = 2 * self.cap;
                    let new_size = new_cap * elem_size;
                    let new_ptr = self.a.realloc(NonNull::new(self.ptr).unwrap().cast(), cur_layout, new_size)
                        .expect("Realloc error!");
                    if new_ptr.cast().as_ptr() != self.ptr {
                        ptr::copy(new_ptr.cast().as_ptr(), self.ptr, elem_size * self.cap);
                    }
                    (new_cap, new_ptr)
                },
                None => {
                    let new_cap = DEFAULT_CAPACITY;
                    let new_ptr = self.a.alloc_array(new_cap)
                        .expect("Alloc error!");
                    (new_cap, new_ptr)
                }
            };
            self.ptr = new_ptr.cast().as_ptr();
            self.cap = new_cap;
        }
    } 

    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            return None;
        }
        unsafe {
            let align = mem::align_of::<T>();
            let size = mem::size_of::<T>() * self.cap;
            Some(Layout::from_size_align_unchecked(size, align))
        }
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.ptr
    }

    pub fn alloc(&self) -> &A {
        &self.a
    }

    pub fn alloc_mut(&mut self) -> &mut A {
        &mut self.a
    }
}

impl<T: Ord> RawVec<T, Global> {

    pub fn new() -> Self {
        Self::new_in(Global)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, Global)
    }
}


impl<T: Ord, A: Alloc> Drop for RawVec<T, A> {
    fn drop(&mut self) {
        if let Some(cur_layout) = self.current_layout() {
            unsafe { self.a.dealloc(NonNull::new(self.ptr).unwrap().cast(), cur_layout) };
        }
    }
}

#[cfg(test)]
mod raw_memory_tests {
    use luos_memory_sandbox::{LuosMemory, LuosAlloc};
    use super::*;
    #[test]
    fn raw_vec_create() {
        let a = LuosAlloc::new(LuosMemory::new());
        let mut vec: RawVec<u128, LuosAlloc> = RawVec::new_in(a);
        vec.double();
        vec.double();
        drop(vec);
    }
}

#[cfg(test)]
mod logic_tests {
    use luos_memory_sandbox::{LuosMemory, LuosAlloc};
    use super::*;
    #[test]
    fn test_push() {
        let a = LuosAlloc::new(LuosMemory::new());
        let mut bh: BinaryHeap<u8, LuosAlloc> = BinaryHeap::new_in(a);
        for i in 1..=9 {
            bh.push(2 * i);
        }
        for i in 1..=9 {
            bh.push(2 * i - 1);
        }
        assert_eq!(&bh.alloc().inner()[..bh.len()], 
            &[1, 2, 5, 8, 3, 6, 9, 13, 17, 10, 4, 12, 7, 14, 11, 16, 15, 18]);
    }

    #[test]
    fn test_pop() {
        let a = LuosAlloc::new(LuosMemory::new_filled_with(0xCC));
        let mut bh: BinaryHeap<u8, LuosAlloc> = BinaryHeap::new_in(a);
        for &i in &[1, 7, 2, 8, 9, 3, 4, 5, 6] {
            bh.push(i);
        }
        for i in 1..=9 {
            assert_eq!(bh.pop(), Some(i));
        }
        assert_eq!(bh.pop(), None);
    }
}
