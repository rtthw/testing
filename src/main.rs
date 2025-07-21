


use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, any::{type_name, TypeId}, num::NonZeroU32, ptr::NonNull, sync::atomic::AtomicIsize};



// NOTE: This is based heavily on https://github.com/Ralith/hecs/blob/master, all credit should go
//       there.

// TODO: Use macros to define tablees/members.



fn main() {
    struct Health(f32);
    struct Regen(f32);

    let healthy_table = Table::new(vec![
        TypeInfo::of::<Health>(),
        TypeInfo::of::<Regen>(),
    ]);

    let records = RecordSet::default();

    {
        let all_healths = Column::<Health>::new(&healthy_table).unwrap();
        let all_regens = Column::<Regen>::new(&healthy_table).unwrap();
    }
}



pub struct Record {
    id: u32,
    generation: NonZeroU32,
}

#[derive(Default)]
pub struct RecordSet {
    meta: Vec<RecordMeta>,
    pending: Vec<u32>,
    free_cursor: AtomicIsize,
    len: u32,
}

impl RecordSet {
    pub fn alloc(&mut self) -> Record {
        self.len += 1;
        if let Some(id) = self.pending.pop() {
            let new_free_cursor = self.pending.len() as isize;
            *self.free_cursor.get_mut() = new_free_cursor;
            Record {
                generation: self.meta[id as usize].generation,
                id,
            }
        } else {
            let id = u32::try_from(self.meta.len()).expect("too many entities");
            self.meta.push(RecordMeta::EMPTY);
            Record {
                generation: NonZeroU32::new(1).unwrap(),
                id,
            }
        }
    }

    pub fn free(&mut self, record: Record) -> Location {
        let meta = self.meta.get_mut(record.id as usize).expect("no such record");
        if meta.generation != record.generation || meta.location.index == u32::MAX {
            panic!("no such record");
        }

        meta.generation = NonZeroU32::new(u32::from(meta.generation).wrapping_add(1))
            .unwrap_or_else(|| NonZeroU32::new(1).unwrap());

        let loc = core::mem::replace(&mut meta.location, RecordMeta::EMPTY.location);

        self.pending.push(record.id);

        let new_free_cursor = self.pending.len() as isize;
        *self.free_cursor.get_mut() = new_free_cursor;
        self.len -= 1;

        loc
    }

    pub fn clear(&mut self) {
        self.meta.clear();
        self.pending.clear();
        *self.free_cursor.get_mut() = 0;
        self.len = 0;
    }
}

#[derive(Clone, Copy, Debug)]
struct RecordMeta {
    generation: NonZeroU32,
    location: Location,
}

impl RecordMeta {
    const EMPTY: RecordMeta = RecordMeta {
        generation: match NonZeroU32::new(1) {
            Some(x) => x,
            None => unreachable!(),
        },
        location: Location {
            table: 0,
            index: u32::MAX, // dummy value, to be filled in
        },
    };
}

#[derive(Clone, Copy, Debug)]
pub struct Location {
    pub table: u32,
    pub index: u32,
}



pub struct Table {
    type_ids: Box<[TypeId]>,
    index: OrderedTypeIdMap<usize>,
    len: u32,
    records: Box<[u32]>,
    data: Box<[TypeData]>,
    types: Vec<TypeInfo>,
}

impl Table {
    pub fn new(types: Vec<TypeInfo>) -> Self {
        let max_align = types.first().map_or(1, |ty| ty.layout.align());
        let member_count = types.len();

        Self {
            type_ids: types.iter().map(|ty| ty.id).collect(),
            index: OrderedTypeIdMap::new(types.iter().enumerate().map(|(i, ty)| (ty.id, i))),
            records: Box::new([]),
            len: 0,
            data: (0..member_count)
                .map(|_| TypeData {
                    state: AtomicBorrow::new(),
                    storage: NonNull::new(max_align as *mut u8).unwrap(),
                })
                .collect(),

            types,
        }
    }

    fn capacity(&self) -> u32 {
        self.records.len() as u32
    }

    fn types(&self) -> &[TypeInfo] {
        &self.types
    }

    fn type_ids(&self) -> &[TypeId] {
        &self.type_ids
    }

    unsafe fn allocate(&mut self, id: u32) -> u32 {
        if self.len as usize == self.records.len() {
            self.grow(64);
        }

        self.records[self.len as usize] = id;
        self.len += 1;
        self.len - 1
    }

    fn grow(&mut self, min_increment: u32) {
        // Double capacity or increase it by `min_increment`, whichever is larger.
        self.grow_exact(self.capacity().max(min_increment))
    }

    fn grow_exact(&mut self, increment: u32) {
        let old_count = self.len as usize;
        let old_cap = self.records.len();
        let new_cap = self.records.len() + increment as usize;
        let mut new_records = vec![!0; new_cap].into_boxed_slice();
        new_records[0..old_count].copy_from_slice(&self.records[0..old_count]);
        self.records = new_records;

        let new_data = self
            .types
            .iter()
            .zip(&*self.data)
            .map(|(info, old)| {
                let storage = if info.layout.size() == 0 {
                    NonNull::new(info.layout.align() as *mut u8).unwrap()
                } else {
                    let layout = Layout::from_size_align(
                        info.layout.size() * new_cap,
                        info.layout.align(),
                    ).unwrap();
                    unsafe {
                        let mem = alloc(layout);
                        let mem = NonNull::new(mem)
                            .unwrap_or_else(|| handle_alloc_error(layout));
                        core::ptr::copy_nonoverlapping(
                            old.storage.as_ptr(),
                            mem.as_ptr(),
                            info.layout.size() * old_count,
                        );
                        mem
                    }
                };
                TypeData {
                    state: AtomicBorrow::new(), // &mut self guarantees no outstanding borrows
                    storage,
                }
            })
            .collect::<Box<[_]>>();

        // Now that we've successfully constructed a replacement, we can
        // deallocate the old column data without risking `self.data` being left
        // partially deallocated on OOM.
        if old_cap > 0 {
            for (info, data) in self.types.iter().zip(&*self.data) {
                if info.layout.size() == 0 {
                    continue;
                }
                unsafe {
                    dealloc(
                        data.storage.as_ptr(),
                        Layout::from_size_align(info.layout.size() * old_cap, info.layout.align())
                            .unwrap(),
                    );
                }
            }
        }

        self.data = new_data;
    }

    fn clear(&mut self) {
        for (ty, data) in self.types.iter().zip(&*self.data) {
            for index in 0..self.len {
                unsafe {
                    let removed = data.storage.as_ptr().add(index as usize * ty.layout.size());
                    (ty.drop)(removed);
                }
            }
        }
        self.len = 0;
    }

    fn get_state<T: Field>(&self) -> Option<usize> {
        self.index.get(&TypeId::of::<T>()).copied()
    }

    unsafe fn get_base<T: Field>(&self, state: usize) -> NonNull<T> {
        debug_assert_eq!(self.types[state].id, TypeId::of::<T>());

        unsafe {
            NonNull::new_unchecked(self.data.get_unchecked(state).storage.as_ptr().cast::<T>())
        }
    }

    fn borrow<T: Field>(&self, state: usize) {
        assert_eq!(self.types[state].id, TypeId::of::<T>());

        if !self.data[state].state.borrow() {
            panic!("{} already borrowed uniquely", type_name::<T>());
        }
    }

    fn release<T: Field>(&self, state: usize) {
        assert_eq!(self.types[state].id, TypeId::of::<T>());

        self.data[state].state.release();
    }
}

impl Drop for Table {
    fn drop(&mut self) {
        self.clear();
        if self.records.is_empty() {
            return;
        }
        for (info, data) in self.types.iter().zip(&*self.data) {
            if info.layout.size() != 0 {
                unsafe {
                    dealloc(
                        data.storage.as_ptr(),
                        Layout::from_size_align_unchecked(
                            info.layout.size() * self.records.len(),
                            info.layout.align(),
                        ),
                    );
                }
            }
        }
    }
}

pub trait Field: Send + Sync + 'static {}

impl<T: Send + Sync + 'static> Field for T {}

pub struct TypeInfo {
    id: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
    #[cfg(debug_assertions)]
    type_name: &'static str,
}

impl TypeInfo {
    pub fn of<T: 'static>() -> Self {
        unsafe fn drop_ptr<T>(x: *mut u8) {
            unsafe { x.cast::<T>().drop_in_place() }
        }

        Self {
            id: TypeId::of::<T>(),
            layout: Layout::new::<T>(),
            drop: drop_ptr::<T>,
            #[cfg(debug_assertions)]
            type_name: core::any::type_name::<T>(),
        }
    }

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub unsafe fn drop(&self, data: *mut u8) {
        unsafe { (self.drop)(data) }
    }
}

struct TypeData {
    state: AtomicBorrow,
    storage: NonNull<u8>,
}



pub struct RecordRef<'a> {
    table: &'a Table,
    record: Record,
    index: u32,
}



pub struct Column<'a, T: Field> {
    table: &'a Table,
    column: &'a [T],
}

impl<'a, T: Field> Column<'a, T> {
    pub fn new(table: &'a Table) -> Option<Self> {
        let state = table.get_state::<T>()?;
        let ptr = unsafe { table.get_base::<T>(state) };
        let column = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), table.len as usize) };
        table.borrow::<T>(state);

        Some(Self {
            table,
            column,
        })
    }
}

impl<T: Field> Drop for Column<'_, T> {
    fn drop(&mut self) {
        let state = self.table.get_state::<T>().unwrap();
        self.table.release::<T>(state);
    }
}



// REFERENCE: https://github.com/Ralith/hecs/blob/master/src/borrow.rs
mod borrow {
    use core::sync::atomic::{AtomicUsize, Ordering};

    /// A bit mask used to signal the `AtomicBorrow` has an active mutable borrow.
    const UNIQUE_BIT: usize = !(usize::MAX >> 1);

    const COUNTER_MASK: usize = usize::MAX >> 1;

    /// An atomic integer used to dynamicaly enforce borrowing rules
    ///
    /// The most significant bit is used to track mutable borrow, and the rest is a
    /// counter for immutable borrows.
    ///
    /// It has four possible states:
    ///  - `0b00000000...` the counter isn't mut borrowed, and ready for borrowing
    ///  - `0b0_______...` the counter isn't mut borrowed, and currently borrowed
    ///  - `0b10000000...` the counter is mut borrowed
    ///  - `0b1_______...` the counter is mut borrowed, and some other thread is trying to borrow
    pub struct AtomicBorrow(AtomicUsize);

    impl AtomicBorrow {
        pub const fn new() -> Self {
            Self(AtomicUsize::new(0))
        }

        pub fn borrow(&self) -> bool {
            // Add one to the borrow counter
            let prev_value = self.0.fetch_add(1, Ordering::Acquire);

            // If the previous counter had all of the immutable borrow bits set,
            // the immutable borrow counter overflowed.
            if prev_value & COUNTER_MASK == COUNTER_MASK {
                core::panic!("immutable borrow counter overflowed")
            }

            // If the mutable borrow bit is set, immutable borrow can't occur. Roll back.
            if prev_value & UNIQUE_BIT != 0 {
                self.0.fetch_sub(1, Ordering::Release);
                false
            } else {
                true
            }
        }

        pub fn borrow_mut(&self) -> bool {
            self.0
                .compare_exchange(0, UNIQUE_BIT, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
        }

        pub fn release(&self) {
            let value = self.0.fetch_sub(1, Ordering::Release);
            debug_assert!(value != 0, "unbalanced release");
            debug_assert!(value & UNIQUE_BIT == 0, "shared release of unique borrow");
        }

        pub fn release_mut(&self) {
            let value = self.0.fetch_and(!UNIQUE_BIT, Ordering::Release);
            debug_assert_ne!(value & UNIQUE_BIT, 0, "unique release of shared borrow");
        }
    }
}

use borrow::*;

mod type_id {
    use std::{any::TypeId, hash::{BuildHasherDefault, Hasher}};

    use hashbrown::HashMap;

    #[derive(Default)]
    pub struct TypeIdHasher {
        hash: u64,
    }

    impl Hasher for TypeIdHasher {
        fn write_u64(&mut self, n: u64) {
            // Only a single value can be hashed, so the old hash should be zero.
            debug_assert_eq!(self.hash, 0);
            self.hash = n;
        }

        fn write_u128(&mut self, n: u128) {
            debug_assert_eq!(self.hash, 0);
            self.hash = n as u64;
        }

        fn write(&mut self, _bytes: &[u8]) {
            panic!("Type ID is the wrong type!")
        }

        fn finish(&self) -> u64 {
            self.hash
        }
    }

    pub type TypeIdMap<V> = HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>;

    pub struct OrderedTypeIdMap<V>(Box<[(TypeId, V)]>);

    impl<V> OrderedTypeIdMap<V> {
        pub fn new(iter: impl Iterator<Item = (TypeId, V)>) -> Self {
            let mut vals = iter.collect::<Box<[_]>>();
            vals.sort_unstable_by_key(|(id, _)| *id);
            Self(vals)
        }

        pub fn search(&self, id: &TypeId) -> Option<usize> {
            self.0.binary_search_by_key(id, |(id, _)| *id).ok()
        }

        pub fn contains_key(&self, id: &TypeId) -> bool {
            self.search(id).is_some()
        }

        pub fn get(&self, id: &TypeId) -> Option<&V> {
            self.search(id).map(move |idx| &self.0[idx].1)
        }
    }
}

use type_id::*;
