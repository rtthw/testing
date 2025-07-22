
#![allow(unstable_features)]
#![feature(test)]


extern crate test;


use std::{alloc::{alloc, dealloc, handle_alloc_error, Layout}, any::TypeId, num::NonZeroU32, ops::Deref, ptr::NonNull, sync::atomic::AtomicIsize};

use type_id::*;



fn main() {
    #[derive(PartialEq)]
    struct Health(f32);
    #[derive(PartialEq)]
    struct Regen(f32);
    #[derive(PartialEq)]
    struct Armor(f32);

    let mut db = Database::default();

    let rec_1 = db.alloc();
    db.add_field(rec_1, Health(100.0));
    db.add_field(rec_1, Regen(5.0));
    let rec_2 = db.alloc();
    db.add_field(rec_2, Health(50.0));
    db.add_field(rec_2, Regen(7.0));
    db.add_field(rec_2, Armor(3.0));
    let rec_3 = db.alloc();
    db.add_field(rec_3, Armor(15.0));

    for (i, column) in db.columns.iter().enumerate() {
        println!(
            "COLUMN #{i}: {:?}",
            column.records.iter().take_while(|i| **i != u32::MAX).collect::<Vec<_>>(),
        );
    }

    for health in db.column::<Health>().iter() {
        println!("HEALTH: {}", health.0);
    }
    for regen in db.column::<Regen>().iter() {
        println!("REGEN: {}", regen.0);
    }
    for armor in db.column::<Armor>().iter() {
        println!("ARMOR: {}", armor.0);
    }
}



#[derive(Default)]
struct Database {
    records: RecordSet,
    columns: Vec<Column>,
    column_index: TypeIdMap<u32>,
}

impl Database {
    pub fn alloc(&mut self) -> Record {
        self.records.alloc()
    }

    pub fn add_field<T: Field>(&mut self, record: Record, mut field: T) {
        let field_id = TypeId::of::<T>();
        let column_id = self.column_index
            .get(&field_id)
            .copied()
            .unwrap_or_else(|| {
                let x = self.columns.len() as u32;
                self.columns.push(Column::new(TypeInfo::of::<T>()));
                let old = self.column_index.insert(field_id, x);
                debug_assert!(old.is_none(), "inserted duplicate field");
                x
            });

        let column = &mut self.columns[column_id as usize];
        unsafe {
            let index = column.alloc(record.id);
            column.put_dynamic((&mut field as *mut T).cast::<u8>(), index);
            core::mem::forget(field);
            self.records.meta[record.id as usize].index = index;
        }
    }

    pub fn column<T: Field>(&mut self) -> ColumnRef<'_, T> {
        let field_id = TypeId::of::<T>();
        let column_id = self.column_index
            .get(&field_id)
            .copied()
            .unwrap_or_else(|| {
                let x = self.columns.len() as u32;
                self.columns.push(Column::new(TypeInfo::of::<T>()));
                let old = self.column_index.insert(field_id, x);
                debug_assert!(old.is_none(), "inserted duplicate field");
                x
            });

        ColumnRef::new(&self.columns[column_id as usize])
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
            let id = u32::try_from(self.meta.len()).expect("too many records");
            self.meta.push(RecordMeta::EMPTY);
            Record {
                generation: NonZeroU32::new(1).unwrap(),
                id,
            }
        }
    }

    pub fn free(&mut self, record: Record) {
        let meta = self.meta.get_mut(record.id as usize).expect("no such record");
        if meta.generation != record.generation || meta.index == u32::MAX {
            panic!("no such record");
        }

        meta.generation = NonZeroU32::new(u32::from(meta.generation).wrapping_add(1))
            .unwrap_or_else(|| NonZeroU32::new(1).unwrap());

        let _ = core::mem::replace(&mut meta.index, RecordMeta::EMPTY.index);

        self.pending.push(record.id);

        let new_free_cursor = self.pending.len() as isize;
        *self.free_cursor.get_mut() = new_free_cursor;
        self.len -= 1;
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
    index: u32,
}

impl RecordMeta {
    const EMPTY: RecordMeta = RecordMeta {
        generation: match NonZeroU32::new(1) {
            Some(x) => x,
            None => unreachable!(),
        },
        index: u32::MAX, // Dummy value, to be filled in later.
    };
}

pub trait Field: 'static {}

impl<T: 'static> Field for T {}

struct Column {
    data: NonNull<u8>,
    len: u32,
    records: Box<[u32]>,
    type_info: TypeInfo,
}

impl Column {
    pub fn new(type_info: TypeInfo) -> Self {
        Self {
            data: NonNull::new(type_info.layout.align() as *mut u8).unwrap(),
            len: 0,
            records: Box::new([]),
            type_info,
        }
    }

    fn capacity(&self) -> u32 {
        self.records.len() as u32
    }

    unsafe fn alloc(&mut self, id: u32) -> u32 {
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

        let new_data = if self.type_info.layout.size() == 0 {
            NonNull::new(self.type_info.layout.align() as *mut u8).unwrap()
        } else {
            let layout = Layout::from_size_align(
                self.type_info.layout.size() * new_cap,
                self.type_info.layout.align(),
            ).unwrap();
            unsafe {
                let mem = alloc(layout);
                let mem = NonNull::new(mem)
                    .unwrap_or_else(|| handle_alloc_error(layout));
                core::ptr::copy_nonoverlapping(
                    self.data.as_ptr(),
                    mem.as_ptr(),
                    self.type_info.layout.size() * old_count,
                );
                mem
            }
        };

        // Now that we've successfully constructed a replacement, we can
        // deallocate the old column data without risking `self.ptr` being left
        // partially deallocated on OOM.
        if old_cap > 0 {
            if self.type_info.layout.size() != 0 {
                unsafe {
                    dealloc(
                        self.data.as_ptr(),
                        Layout::from_size_align(
                            self.type_info.layout.size() * old_cap,
                            self.type_info.layout.align(),
                        )
                            .unwrap(),
                    );
                }
            }
        }

        self.data = new_data;
    }

    fn clear(&mut self) {
        for index in 0..self.len {
            unsafe {
                let removed = self.data.as_ptr().add(index as usize * self.type_info.layout.size());
                (self.type_info.drop)(removed);
            }
        }
        self.len = 0;
    }

    unsafe fn get_base<T: Field>(&self) -> NonNull<T> {
        unsafe {
            NonNull::new_unchecked(self.data.as_ptr().cast::<T>())
        }
    }
}

impl Column {
    unsafe fn get_dynamic(&self, index: u32) -> Option<NonNull<u8>> {
        debug_assert!(index <= self.len);
        Some(unsafe { NonNull::new_unchecked(
            self.data
                .as_ptr()
                .add(self.type_info.layout.size() * index as usize)
                .cast::<u8>(),
        ) })
    }

    unsafe fn put_dynamic(&mut self, field: *mut u8, index: u32) {
        let ptr = unsafe { self.get_dynamic(index) }
            .unwrap()
            .as_ptr()
            .cast::<u8>();
        unsafe { core::ptr::copy_nonoverlapping(field, ptr, self.type_info.layout.size()); }
    }
}

impl Drop for Column {
    fn drop(&mut self) {
        self.clear();
        if self.records.is_empty() {
            return;
        }
        if self.type_info.layout.size() != 0 {
            unsafe {
                dealloc(
                    self.data.as_ptr(),
                    Layout::from_size_align_unchecked(
                        self.type_info.layout.size() * self.records.len(),
                        self.type_info.layout.align(),
                    ),
                );
            }
        }
    }
}

#[derive(Clone)]
pub struct TypeInfo {
    id: TypeId,
    layout: Layout,
    drop: unsafe fn(*mut u8),
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
        }
    }

    pub fn id(&self) -> TypeId {
        self.id
    }

    pub unsafe fn drop(&self, data: *mut u8) {
        unsafe { (self.drop)(data) }
    }
}



pub struct ColumnRef<'a, T: Field> {
    data: &'a [T],
    _column: &'a Column,
}

impl<'a, T: Field> ColumnRef<'a, T> {
    fn new(column: &'a Column) -> Self {
        let ptr = unsafe { column.get_base::<T>() };
        let data = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), column.len as usize) };

        Self {
            data,
            _column: column,
        }
    }
}

impl<T: Field> Deref for ColumnRef<'_, T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        self.data
    }
}



mod type_id {
    use core::{any::TypeId, hash::{BuildHasherDefault, Hasher}};

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
}



#[cfg(test)]
mod tests {
    use super::*;

    #[derive(PartialEq)]
    struct Health(f32);
    #[derive(PartialEq)]
    struct Regen(f32);
    #[derive(PartialEq)]
    struct Armor(f32);

    fn define_test_db() -> Database {
        let mut db = Database::default();

        let rec_1 = db.alloc();
        db.add_field(rec_1, Health(100.0));
        db.add_field(rec_1, Regen(5.0));
        let rec_2 = db.alloc();
        db.add_field(rec_2, Health(50.0));
        db.add_field(rec_2, Regen(7.0));

        for i in 0..100 {
            let rec = db.alloc();
            db.add_field(rec, Armor(i as f32));
        }

        // for (i, column) in db.columns.iter().enumerate() {
        //     println!(
        //         "COLUMN #{i}: {:?}",
        //         column.records.iter().take_while(|i| **i != u32::MAX).collect::<Vec<_>>(),
        //     );
        // }

        db
    }

    #[test]
    fn works() {
        let mut db = define_test_db();
        assert!(db.columns.len() == 3);
        let mut armor_acc = 0.0;
        for armor in db.column::<Armor>().iter() {
            armor_acc += armor.0;
        }

        assert!(armor_acc == (0..100).into_iter().fold(0.0, |acc, i| acc + i as f32));
    }

    #[bench]
    fn bench_pure_iter_100(bencher: &mut test::Bencher) {
        let mut db = define_test_db();

        assert!(db.column::<Armor>().len() == 100);

        bencher.iter(|| {
            for armor in db.column::<Armor>().iter() {
                // Use a black box to ensure the compiler doesn't optimize this away. A noop is
                // about 12 times faster on my machine.
                test::black_box(&armor);
            }
        });
    }

    #[bench]
    fn bench_simple_acc_100(bencher: &mut test::Bencher) {
        let mut db = define_test_db();

        assert!(db.column::<Armor>().len() == 100);

        bencher.iter(|| {
            let mut acc = 0.0;
            // This column iteration is almost (within a few nanoseconds) as fast as:
            //      `(0..100).into_iter().fold(0.0, |acc, i| acc + i as f32)`
            for armor in db.column::<Armor>().iter() {
                acc += armor.0;
            }
            // Uncommenting this is a 2x slow down (as would be expected).
            // assert!(acc == (0..100).into_iter().fold(0.0, |acc, i| acc + i as f32));

            acc
        });
    }

    #[bench]
    fn bench_2(bencher: &mut test::Bencher) {
        let mut db = define_test_db();
        let record = db.alloc();

        bencher.iter(|| {
            db.add_field(record, Health(1.0));
            assert!(db.column::<Health>().last() == Some(&Health(1.0)));
        });
    }
}
