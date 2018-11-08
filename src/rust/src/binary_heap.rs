use core::alloc::{Alloc, Layout, AllocErr};
use core::fmt::{self, Debug};
use core::marker::PhantomData;
use core::mem::{align_of, size_of};
use core::ptr::{self, NonNull};
use std::alloc::System;

const DEFAULT_CAPACITY: usize = 16;

pub struct BinaryHeap<T: Ord> {
    ptr: Option<NonNull<T>>,
    len: usize,
    cap: usize,
    a: System,
    layout: Layout,
    _marker: PhantomData<T>,
}

impl<T: Ord> BinaryHeap<T> {

    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(cap: usize) -> Self {
        let size_of_t = size_of::<T>();
        let alloc_size = cap.checked_mul(size_of_t)
            .expect("Capacity overflow");
        let layout = Layout::from_size_align(align_of::<T>(), alloc_size).unwrap();
        BinaryHeap {
            ptr: None,
            len: 0,
            cap,
            a: System,
            layout, 
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, value: T) {
        self.try_push(value).expect("Out of memory");
    }

    fn try_push(&mut self, value: T) -> Result<(), AllocErr> {
        if self.ptr == None || self.len == self.cap {
            self.buf_double()?;
        }
        if let Some(_ptr) = self.ptr {
            self.len += 1;
            unsafe { ptr::write(self.as_mut_ptr().add(self.len), value) };
            let mut now = self.len;
            while now > 1 {
                let next = now >> 1;
                let now_ptr = unsafe { self.as_mut_ptr().add(now) };
                let next_ptr = unsafe { self.as_mut_ptr().add(next) };
                if unsafe { &*now_ptr } < unsafe { &*next_ptr } {
                    unsafe { ptr::swap(now_ptr, next_ptr) };
                }
                now = next;
            } 
        }
        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if !self.is_empty() {
            let ans = unsafe { ptr::read(self.as_mut_ptr().add(1)) };
            unsafe { ptr::swap(self.as_mut_ptr().add(1), self.as_mut_ptr().add(self.len)) };
            self.len -= 1;
            let mut now = 1;
            loop {
                let mut next = now << 1;
                if next > self.len {
                    break;
                }
                let next_ptr = unsafe { self.as_mut_ptr().add(next) };
                let now_ptr = unsafe { self.as_mut_ptr().add(now) };
                if (next + 1) <= self.len && unsafe { &*self.as_mut_ptr().add(next + 1) 
                    < &*next_ptr } {
                    next += 1;
                }
                let next_ptr = unsafe { self.as_mut_ptr().add(next) };
                unsafe {
                    if &*next_ptr < &*now_ptr  {
                        ptr::swap(now_ptr,  next_ptr);
                    }
                }
                now = next;
            }
            Some(ans)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<&T> {
        if !self.is_empty() {            
            let ans = unsafe { &*self.as_mut_ptr().add(1) };
            Some(ans)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        if self.len > 0 {
            let mut cur_ptr = self.as_mut_ptr();
            for _i in 1..=self.len {
                unsafe { 
                    cur_ptr = cur_ptr.add(1);
                    ptr::drop_in_place(cur_ptr); 
                };
            }
        }
        if let Some(ptr) = self.ptr {
            unsafe { self.a.dealloc(ptr.cast::<u8>(), self.layout) };
            self.ptr = None;
        }
        self.len = 0;
        self.cap = DEFAULT_CAPACITY;
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter {
            inner: self
        }
    }  

    fn buf_double(&mut self) -> Result<(), AllocErr> {
        if let Some(ptr) = self.ptr {
            self.cap *= 2;
            let new_size = self.cap.checked_mul(size_of::<T>())
                .expect("Capacity overflow when doubling buffer");
            let new_ptr = unsafe { self.a.realloc(ptr.cast::<u8>(), self.layout, new_size) }?;
            unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.cast::<T>().as_ptr(), self.cap) };
            self.ptr = Some(new_ptr.cast::<T>());
        } else {
            self.ptr = Some(unsafe { self.a.alloc(self.layout)? }.cast::<T>());
        }
        Ok(())
    }

    fn as_mut_ptr(&self) -> *mut T { 
        self.ptr.unwrap().as_ptr()
    }
}

impl<T: Ord> Drop for BinaryHeap<T> {
    fn drop(&mut self) {
        self.clear()
    }
}

impl<T: Ord + Debug> Debug for BinaryHeap<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[")?;
        if self.len > 0 {
            let mut cur_ptr = self.as_mut_ptr();
            for i in 1..=self.len {
                unsafe { 
                    cur_ptr = cur_ptr.add(1);
                    write!(f, "{:?}", &*cur_ptr)?;
                };
                if i != self.len {
                    write!(f, ", ");
                }
            }
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<T: Ord + Clone> Clone for BinaryHeap<T> {
    fn clone(&self) -> Self {
        if let Some(ptr) = self.ptr {
            let layout = self.layout;
            let new_ptr = unsafe { System.alloc(layout) }
                .expect("Failed to alloc");
            unsafe { ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.cast::<T>().as_ptr(), self.cap + 1) };
            
        println!("Clone!");
            BinaryHeap {
                ptr: Some(new_ptr.cast::<T>()),
                len: self.len,
                cap: self.cap,
                a: System,
                layout,
                _marker: PhantomData,
            }
        } else {
            BinaryHeap::with_capacity(self.cap)
        }
    }
}

pub struct IntoIter<T: Ord> {
    inner: BinaryHeap<T>
}

impl<T: Ord> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<T> {
        self.inner.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::BinaryHeap;

    #[test]
    fn test_heap_sort() {
        let mut bh = BinaryHeap::new();
        let data = vec![4, 5, 7, 3, 2, 1, 6, 9, 8];
        for i in data {
            bh.push(i);
        }
        let mut result = Vec::with_capacity(9);
        for i in bh.into_iter() {
            result.push(i);
        }
        assert_eq!(result, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_peek_len() {
        let mut bh = BinaryHeap::new();
        bh.push(4);
        assert_eq!((Some(&4), Some(&4), 1), (bh.peek(), bh.peek(), bh.len()));
        bh.push(5);
        assert_eq!((Some(&4), Some(&4), 2), (bh.peek(), bh.peek(), bh.len()));
        bh.push(3);
        assert_eq!((Some(&3), Some(&3), 3), (bh.peek(), bh.peek(), bh.len()));
        bh.pop();
        assert_eq!((Some(&4), Some(&4), 2), (bh.peek(), bh.peek(), bh.len()));
        bh.pop();
        assert_eq!((Some(&5), Some(&5), 1), (bh.peek(), bh.peek(), bh.len()));
    }

    #[test]
    fn test_clear() {
        let mut bh = BinaryHeap::new();
        bh.push(5);
        bh.push(1);
        assert!(!bh.is_empty());
        assert_eq!(Some(&1), bh.peek());
        bh.clear();
        assert!(bh.is_empty());
        assert_eq!(None, bh.peek());
    }

    #[test]
    fn test_drop_safe() {
        #[derive(Eq, PartialEq, Ord, PartialOrd)]
        struct Data(i128);

        impl Drop for Data {
            fn drop(&mut self) {
                println!("Data: {}", self.0);
            }
        }

        let mut bh = BinaryHeap::new();
        for i in 1..=10 {
            bh.push(Data(i));
        }
        bh.peek(); // does not drop
        drop(bh.pop());
        println!("Before drop");
        drop(bh);
        println!("After drop");
    }

    #[test]
    fn test_debug() {
        let mut bh = BinaryHeap::new();
        assert_eq!(format!("{:?}", bh), "[]");
        for i in 0..5 {
            bh.push(i)
        }
        assert_eq!(format!("{:?}", bh), "[0, 1, 2, 3, 4]");
    }

    #[test]
    fn test_batch_push() {
        let mut bh = BinaryHeap::new();
        for i in (0..1000u128).rev() {
            bh.push(i);
            println!("{:?}", bh);
        }
        for i in 0..1000 {
            assert_eq!(Some(i), bh.pop());
        }

    }

    #[test]
    fn test_clone() {
        let mut bh = BinaryHeap::new();
        for i in 0..50 {
            bh.push(i);
        }
        let mut bh2 = bh.clone();
        for i in 0..50 {
            assert_eq!(Some(i), bh2.pop());
        }
    }
}