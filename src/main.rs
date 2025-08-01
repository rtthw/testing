
#![allow(soft_unstable)]
#![feature(test)]

use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, any::TypeId, marker::PhantomData, ops::Deref, ptr::NonNull};

use testing::TypeMap;


extern crate test;



fn main() {
    let mut map = Map::default();
    map.register::<u8>();
    let a = map.alloc();
    map.insert(a, 11_u8);

    let u8s = map.values::<u8>();

    assert_eq!(u8s.len(), 1);
    assert_eq!(u8s.first(), Some(&11));
}



#[derive(Default)]
pub struct Map {
    inner: TypeMap<List>,
    elements: Vec<Element>,
}

impl Map {
    pub fn alloc(&mut self) -> u32 {
        let key = self.elements.len();
        self.elements.push(Element {
            values: Vec::with_capacity(1),
        });

        key as _
    }

    pub fn register<T: 'static>(&mut self) {
        self.inner.insert::<T>(List::new(Layout::new::<T>()));
    }

    pub fn insert<T: 'static>(&mut self, key: u32, mut value: T) {
        let type_id = TypeId::of::<T>();

        unsafe {
            let list = self.inner.get_mut::<T>().unwrap();
            let index = list.alloc(key);
            list.put_dynamic((&mut value as *mut T).cast::<u8>(), index);
            core::mem::forget(value);
            self.elements[key as usize].values.push(Value { type_id, index });
        }
    }

    pub fn values<T: 'static>(&self) -> ValuesRef<'_, T> {
        let list = self.inner.get::<T>().unwrap();
        ValuesRef::new(list)
    }
}

pub struct Value {
    type_id: TypeId,
    index: u32,
}

impl Value {
    pub fn get<T: 'static>(&self, map: &Map) -> Option<&T> {
        debug_assert!(self.type_id == TypeId::of::<T>());

        Some(unsafe { self.get_unchecked(map) })
    }

    pub unsafe fn get_unchecked<T: 'static>(&self, map: &Map) -> &T {
        unsafe {
            map.inner.get::<T>().unwrap()
                .get_dynamic(self.index).unwrap()
                .cast::<T>()
                .as_ref()
        }
    }
}

pub struct Element {
    values: Vec<Value>,
}

pub struct List {
    data: NonNull<u8>,
    len: u32,
    keys: Box<[u32]>,
    layout: Layout,
}

// NOTE: This is never actually used. Just necessary to ensure `Map` implements `Default`.
impl Default for List {
    fn default() -> Self {
        unreachable!()
    }
}

impl List {
    fn new(layout: Layout) -> Self {
        Self {
            data: NonNull::new(layout.align() as *mut u8).unwrap(),
            len: 0,
            keys: Box::new([]),
            layout,
        }
    }

    fn capacity(&self) -> u32 {
        self.keys.len() as u32
    }

    unsafe fn alloc(&mut self, id: u32) -> u32 {
        if self.len as usize == self.keys.len() {
            self.grow(64);
        }

        self.keys[self.len as usize] = id;
        self.len += 1;
        self.len - 1
    }

    fn grow(&mut self, min_increment: u32) {
        // Double capacity or increase it by `min_increment`, whichever is larger.
        self.grow_exact(self.capacity().max(min_increment))
    }

    fn grow_exact(&mut self, increment: u32) {
        let old_count = self.len as usize;
        let old_cap = self.keys.len();
        let new_cap = self.keys.len() + increment as usize;
        let mut new_keys = vec![!0; new_cap].into_boxed_slice();
        new_keys[0..old_count].copy_from_slice(&self.keys[0..old_count]);
        self.keys = new_keys;

        let new_data = if self.layout.size() == 0 {
            NonNull::new(self.layout.align() as *mut u8).unwrap()
        } else {
            let layout = Layout::from_size_align(
                self.layout.size() * new_cap,
                self.layout.align(),
            ).unwrap();
            unsafe {
                let mem = alloc(layout);
                let mem = NonNull::new(mem)
                    .unwrap_or_else(|| handle_alloc_error(layout));
                core::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    mem.as_ptr(),
                    self.layout.size() * old_count,
                );
                mem
            }
        };

        // Now that we've successfully constructed a replacement, we can
        // deallocate the old value data without risking `self.data` being left
        // partially deallocated on OOM.
        if old_cap > 0 {
            if self.layout.size() != 0 {
                unsafe {
                    dealloc(
                        self.data.as_ptr(),
                        Layout::from_size_align(self.layout.size() * old_cap, self.layout.align())
                            .unwrap(),
                    );
                }
            }
        }

        self.data = new_data;
    }

    // fn clear(&mut self) {
    //     for index in 0..self.len {
    //         unsafe {
    //             let removed = self.data.as_ptr().add(index as usize * self.layout.size());
    //             (self.type_info.drop)(removed);
    //         }
    //     }
    //     self.len = 0;
    // }

    unsafe fn get_base<T: 'static>(&self) -> NonNull<T> {
        unsafe {
            NonNull::new_unchecked(self.data.as_ptr().cast::<T>())
        }
    }
}

impl List {
    unsafe fn get_dynamic(&self, index: u32) -> Option<NonNull<u8>> {
        debug_assert!(index <= self.len);
        Some(unsafe { NonNull::new_unchecked(
            self.data
                .as_ptr()
                .add(self.layout.size() * index as usize)
                .cast::<u8>(),
        ) })
    }

    unsafe fn put_dynamic(&mut self, field: *mut u8, index: u32) {
        let ptr = unsafe { self.get_dynamic(index) }
            .unwrap()
            .as_ptr()
            .cast::<u8>();
        unsafe { core::ptr::copy_nonoverlapping(field, ptr, self.layout.size()); }
    }
}

pub struct ValuesRef<'a, T: 'static> {
    elements: &'a [T],
    _data: PhantomData<&'a List>,
}

impl<'a, T: 'static> ValuesRef<'a, T> {
    fn new(list: &'a List) -> Self {
        let ptr = unsafe { list.get_base::<T>() };
        let elements = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), list.len as usize) };

        Self {
            elements,
            _data: PhantomData,
        }
    }
}

impl<T: 'static> Deref for ValuesRef<'_, T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.elements
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn works() {
        let mut map = Map::default();
        map.register::<u8>();
        let a = map.alloc();
        map.insert(a, 11_u8);

        let u8s = map.values::<u8>();

        assert_eq!(u8s.len(), 1);
        assert_eq!(u8s.first(), Some(&11));
    }

    #[bench]
    fn bench_iter_acc(bencher: &mut test::Bencher) {
        let mut map = Map::default();
        map.register::<u32>();

        for i in 0..1000_u32 {
            let key = map.alloc();
            map.insert(key, i);
        }

        bencher.iter(|| {
            let mut acc = 0;
            for i in map.values::<u32>().iter() {
                acc += i;
            }

            acc
        });
    }

    #[bench]
    fn bench_iter_acc_vec(bencher: &mut test::Bencher) {
        let vec = (0..1000_u32).collect::<Vec<_>>();

        bencher.iter(|| {
            let mut acc = 0;
            for i in vec.iter() {
                acc += i;
            }

            acc
        });
    }
}
