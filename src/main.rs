


use std::{
    alloc::{alloc, dealloc, handle_alloc_error, Layout},
    any::{type_name, TypeId},
    borrow::Borrow,
    collections::HashMap,
    marker::PhantomData,
    num::NonZeroU32,
    ops::Deref,
    ptr::{copy_nonoverlapping, NonNull},
    sync::atomic::{AtomicIsize, Ordering},
};



// NOTE: This is based heavily on https://github.com/Ralith/hecs/blob/master, all credit should go
//       there.

// TODO: Use macros to define tablees/members.



fn main() {
    #[derive(PartialEq)]
    struct Health(f32);
    #[derive(PartialEq)]
    struct Regen(f32);

    let mut db = Database::new();

    let rec_1 = db.alloc();
    db.put(rec_1, (Health(100.0), Regen(5.0)));

    {
        let rec = db.record(rec_1);

        let all_healths = Column::<Health>::new(rec.table).unwrap();
        let all_regens = Column::<Regen>::new(rec.table).unwrap();

        assert!(all_healths.len() == 1);
        assert!(all_regens.len() == 1);

        assert!(*all_healths.get(rec_1.id as usize).unwrap() == Health(100.0));
        assert!(*all_regens.get(rec_1.id as usize).unwrap() == Regen(5.0));
    }
}



pub struct Database {
    records: RecordSet,
    tables: TableSet,
}

impl Database {
    pub fn new() -> Self {
        Self {
            records: RecordSet::default(),
            tables: TableSet::new(),
        }
    }

    pub fn alloc(&mut self) -> Record {
        self.records.alloc()
    }

    pub fn put(&mut self, record: Record, fields: impl FieldSet) {
        let table_id = fields.with_ids(|ids| self.tables.get(ids, || fields.type_info()));

        let table = &mut self.tables.tables[table_id as usize];
        unsafe {
            let index = table.allocate(record.id);
            fields.put(|ptr, ty| {
                table.put_dynamic(ptr, ty.id(), ty.layout.size(), index);
            });
            self.records.meta[record.id as usize].location = Location {
                table: table_id,
                index,
            };
        }
    }

    pub fn record(&self, record: Record) -> RecordRef<'_> {
        let loc = self.records.get(record);

        RecordRef {
            table: &self.tables.tables[loc.table as usize],
            record,
            index: loc.index,
        }
    }
}

struct TableSet {
    index: HashMap<Box<[TypeId]>, u32>,
    tables: Vec<Table>,
}

impl TableSet {
    fn new() -> Self {
        Self {
            index: Some((Box::default(), 0)).into_iter().collect(),
            tables: vec![Table::new(Vec::new())],
        }
    }

    fn get<T: Borrow<[TypeId]> + Into<Box<[TypeId]>>>(
        &mut self,
        fields: T,
        info: impl FnOnce() -> Vec<TypeInfo>,
    ) -> u32 {
        self.index
            .get(fields.borrow())
            .copied()
            .unwrap_or_else(|| self.insert(fields.into(), info()))
    }

    fn insert(&mut self, fields: Box<[TypeId]>, info: Vec<TypeInfo>) -> u32 {
        let x = self.tables.len() as u32;
        self.tables.push(Table::new(info));
        let old = self.index.insert(fields, x);
        debug_assert!(old.is_none(), "inserted duplicate table");
        x
    }
}



#[derive(Clone, Copy)]
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

    pub fn get(&self, record: Record) -> Location {
        if self.meta.len() <= record.id as usize {
            // Check if this could have been obtained from `reserve_entity`
            let free = self.free_cursor.load(Ordering::Relaxed);
            if record.generation.get() == 1
                && free < 0
                && (record.id as isize) < (free.abs() + self.meta.len() as isize)
            {
                return Location {
                    table: 0,
                    index: u32::MAX,
                };
            } else {
                panic!("no such record");
            }
        }
        let meta = &self.meta[record.id as usize];
        if meta.generation != record.generation || meta.location.index == u32::MAX {
            panic!("no such record");
        }
        meta.location
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

impl Table {
    unsafe fn get_dynamic(
        &self,
        ty: TypeId,
        size: usize,
        index: u32,
    ) -> Option<NonNull<u8>> {
        debug_assert!(index <= self.len);
        Some(unsafe { NonNull::new_unchecked(
            self.data
                .get_unchecked(*self.index.get(&ty)?)
                .storage
                .as_ptr()
                .add(size * index as usize)
                .cast::<u8>(),
        ) })
    }

    unsafe fn put_dynamic(
        &mut self,
        field: *mut u8,
        ty: TypeId,
        size: usize,
        index: u32,
    ) {
        let ptr = unsafe { self.get_dynamic(ty, size, index) }
            .unwrap()
            .as_ptr()
            .cast::<u8>();
        unsafe { copy_nonoverlapping(field, ptr, size); }
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

pub unsafe trait FieldSet {
    fn key(&self) -> Option<TypeId>;
    fn with_ids<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T;
    fn type_info(&self) -> Vec<TypeInfo>;
    unsafe fn put(self, f: impl FnMut(*mut u8, TypeInfo));
}

unsafe trait FieldSetInner: FieldSet {
    fn with_static_ids<T>(f: impl FnOnce(&[TypeId]) -> T) -> T;
    fn with_static_type_info<T>(f: impl FnOnce(&[TypeInfo]) -> T) -> T;
}

mod macros {
    use super::*;

    macro_rules! tuple_impl {
        ($($name: ident),*) => {
            unsafe impl<$($name: Field),*> FieldSet for ($($name,)*) {
                fn key(&self) -> Option<TypeId> {
                    Some(TypeId::of::<Self>())
                }

                fn with_ids<T>(&self, f: impl FnOnce(&[TypeId]) -> T) -> T {
                    Self::with_static_ids(f)
                }

                fn type_info(&self) -> Vec<TypeInfo> {
                    Self::with_static_type_info(|info| info.to_vec())
                }

                #[allow(unused_variables, unused_mut)]
                unsafe fn put(self, mut f: impl FnMut(*mut u8, TypeInfo)) {
                    #[allow(non_snake_case)]
                    let ($(mut $name,)*) = self;
                    $(
                        f(
                            (&mut $name as *mut $name).cast::<u8>(),
                            TypeInfo::of::<$name>()
                        );
                        core::mem::forget($name);
                    )*
                }
            }

            unsafe impl<$($name: Field),*> FieldSetInner for ($($name,)*) {
                fn with_static_ids<T>(f: impl FnOnce(&[TypeId]) -> T) -> T {
                    const N: usize = count!($($name),*);
                    let mut xs: [(usize, TypeId); N] = [$((core::mem::align_of::<$name>(), TypeId::of::<$name>())),*];
                    xs.sort_unstable_by(|x, y| x.0.cmp(&y.0).reverse().then(x.1.cmp(&y.1)));
                    let mut ids = [TypeId::of::<()>(); N];
                    for (slot, &(_, id)) in ids.iter_mut().zip(xs.iter()) {
                        *slot = id;
                    }
                    f(&ids)
                }

                fn with_static_type_info<T>(f: impl FnOnce(&[TypeInfo]) -> T) -> T {
                    const N: usize = count!($($name),*);
                    let mut xs: [TypeInfo; N] = [$(TypeInfo::of::<$name>()),*];
                    xs.sort_unstable();
                    f(&xs)
                }
            }
        };
    }

    macro_rules! count {
        () => { 0 };
        ($x: ident $(, $rest: ident)*) => { 1 + count!($($rest),*) };
    }

    macro_rules! reverse_apply {
        ($m: ident [] $($reversed:tt)*) => {
            $m!{$($reversed),*}  // base case
        };
        ($m: ident [$first:tt $($rest:tt)*] $($reversed:tt)*) => {
            reverse_apply!{$m [$($rest)*] $first $($reversed)*}
        };
    }

    macro_rules! smaller_tuples_too {
        ($m: ident, $next: tt) => {
            $m!{}
            $m!{$next}
        };
        ($m: ident, $next: tt, $($rest: tt),*) => {
            smaller_tuples_too!{$m, $($rest),*}
            reverse_apply!{$m [$next $($rest)*]}
        };
    }

    smaller_tuples_too!(tuple_impl, O, N, M, L, K, J, I, H, G, F, E, D, C, B, A);
}

#[derive(Clone)]
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

impl PartialEq for TypeInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TypeInfo {}

impl PartialOrd for TypeInfo {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TypeInfo {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.layout
            .align()
            .cmp(&other.layout.align())
            .reverse()
            .then_with(|| self.id.cmp(&other.id))
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

pub trait FieldRef<'a> {
    type Ref;
    type Column;

    fn get_field(record: RecordRef<'a>) -> Option<Self::Ref>;
    fn get_column(table: &'a Table) -> Option<Self::Column>;
}

impl<'a, T: Field> FieldRef<'a> for &'a T {
    type Ref = Ref<'a, T>;
    type Column = Column<'a, T>;

    fn get_field(record: RecordRef<'a>) -> Option<Self::Ref> {
        Some(unsafe { Ref::new(record.table, record.index)? })
    }

    fn get_column(table: &'a Table) -> Option<Self::Column> {
        Column::new(table)
    }
}



pub struct Ref<'a, T: ?Sized> {
    borrow: FieldBorrow<'a>,
    target: NonNull<T>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: Field> Ref<'a, T> {
    unsafe fn new(table: &'a Table, index: u32) -> Option<Self> {
        let (target, borrow) = unsafe { FieldBorrow::new::<T>(table, index) }?;

        Some(Self {
            borrow,
            target,
            _marker: PhantomData,
        })
    }
}

impl<T: ?Sized> Deref for Ref<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { self.target.as_ref() }
    }
}

struct FieldBorrow<'a> {
    table: &'a Table,
    state: usize,
}

impl<'a> FieldBorrow<'a> {
    // This method is unsafe as if the `index` is out of bounds,
    // then this will cause undefined behavior as the returned
    // `target` will point to undefined memory.
    unsafe fn new<T: Field>(table: &'a Table, index: u32) -> Option<(NonNull<T>, Self)> {
        let state = table.get_state::<T>()?;

        let target = unsafe {
            NonNull::new_unchecked(table.get_base::<T>(state).as_ptr().add(index as usize))
        };

        table.borrow::<T>(state);

        Some((target, Self { table, state }))
    }
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

impl<T: Field> Deref for Column<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.column
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
