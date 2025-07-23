
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
    db.add_to_table(rec_1, Health(100.0));
    db.add_to_table(rec_1, Regen(5.0));
    let rec_2 = db.alloc();
    db.add_to_table(rec_2, Health(50.0));
    db.add_to_table(rec_2, Regen(7.0));
    db.add_to_table(rec_2, Armor(3.0));
    let rec_3 = db.alloc();
    db.add_to_table(rec_3, Armor(15.0));

    for (i, table) in db.tables.iter().enumerate() {
        println!(
            "TABLE #{i}: {:?}",
            table.records.iter().take_while(|i| **i != u32::MAX).collect::<Vec<_>>(),
        );
    }

    for health in db.table::<Health>().iter() {
        println!("HEALTH: {}", health.0);
    }
    for regen in db.table::<Regen>().iter() {
        println!("REGEN: {}", regen.0);
    }
    for armor in db.table::<Armor>().iter() {
        println!("ARMOR: {}", armor.0);
    }
}



#[derive(Default)]
struct Database {
    records: RecordSet,
    tables: Vec<TableImpl>,
    table_index: TypeIdMap<u32>,
}

impl Database {
    pub fn alloc(&mut self) -> Record {
        self.records.alloc()
    }

    pub fn add_to_table<T: Table>(&mut self, record: Record, mut row: T) {
        let field_id = TypeId::of::<T>();
        let table_id = self.table_index
            .get(&field_id)
            .copied()
            .unwrap_or_else(|| {
                let x = self.tables.len() as u32;
                self.tables.push(TableImpl::new(TypeInfo::of::<T>()));
                let old = self.table_index.insert(field_id, x);
                debug_assert!(old.is_none(), "inserted duplicate field");
                x
            });

        let table = &mut self.tables[table_id as usize];
        unsafe {
            let index = table.alloc(record.id);
            table.put_dynamic((&mut row as *mut T).cast::<u8>(), index);
            core::mem::forget(row);
            self.records.meta[record.id as usize].index = index;
        }
    }

    pub fn table<T: Table>(&mut self) -> TableRef<'_, T> {
        let type_id = TypeId::of::<T>();
        let table_id = self.table_index
            .get(&type_id)
            .copied()
            .unwrap_or_else(|| {
                let x = self.tables.len() as u32;
                self.tables.push(TableImpl::new(TypeInfo::of::<T>()));
                let old = self.table_index.insert(type_id, x);
                debug_assert!(old.is_none(), "inserted duplicate field");
                x
            });

        TableRef::new(&self.tables[table_id as usize])
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

pub trait Table: 'static {}

impl<T: 'static> Table for T {}

struct TableImpl {
    data: NonNull<u8>,
    len: u32,
    records: Box<[u32]>,
    type_info: TypeInfo,
}

impl TableImpl {
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
        // deallocate the old table data without risking `self.ptr` being left
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

    unsafe fn get_base<T: Table>(&self) -> NonNull<T> {
        unsafe {
            NonNull::new_unchecked(self.data.as_ptr().cast::<T>())
        }
    }
}

impl TableImpl {
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

impl Drop for TableImpl {
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



pub struct TableRef<'a, T: Table> {
    data: &'a [T],
    _table: &'a TableImpl,
}

impl<'a, T: Table> TableRef<'a, T> {
    fn new(table: &'a TableImpl) -> Self {
        let ptr = unsafe { table.get_base::<T>() };
        let data = unsafe { core::slice::from_raw_parts(ptr.as_ptr(), table.len as usize) };

        Self {
            data,
            _table: table,
        }
    }
}

impl<T: Table> Deref for TableRef<'_, T> {
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
        db.add_to_table(rec_1, Health(100.0));
        db.add_to_table(rec_1, Regen(5.0));
        let rec_2 = db.alloc();
        db.add_to_table(rec_2, Health(50.0));
        db.add_to_table(rec_2, Regen(7.0));

        for i in 0..100 {
            let rec = db.alloc();
            db.add_to_table(rec, Armor(i as f32));
        }

        // for (i, table) in db.tables.iter().enumerate() {
        //     println!(
        //         "TABLE #{i}: {:?}",
        //         table.records.iter().take_while(|i| **i != u32::MAX).collect::<Vec<_>>(),
        //     );
        // }

        db
    }

    #[test]
    fn works() {
        let mut db = define_test_db();
        assert!(db.tables.len() == 3);
        let mut armor_acc = 0.0;
        for armor in db.table::<Armor>().iter() {
            armor_acc += armor.0;
        }

        assert!(armor_acc == (0..100).into_iter().fold(0.0, |acc, i| acc + i as f32));
    }

    #[bench]
    fn bench_pure_iter_100(bencher: &mut test::Bencher) {
        let mut db = define_test_db();

        assert!(db.table::<Armor>().len() == 100);

        bencher.iter(|| {
            for armor in db.table::<Armor>().iter() {
                // Use a black box to ensure the compiler doesn't optimize this away. A noop is
                // about 12 times faster on my machine.
                test::black_box(&armor);
            }
        });
    }

    #[bench]
    fn bench_simple_acc_100(bencher: &mut test::Bencher) {
        let mut db = define_test_db();

        assert!(db.table::<Armor>().len() == 100);

        bencher.iter(|| {
            let mut acc = 0.0;
            // This table iteration is almost (within a few nanoseconds) as fast as:
            //      `(0..100).into_iter().fold(0.0, |acc, i| acc + i as f32)`
            for armor in db.table::<Armor>().iter() {
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
            db.add_to_table(record, Health(1.0));
            assert!(db.table::<Health>().last() == Some(&Health(1.0)));
        });
    }
}
