//! Type map



use core::{any::TypeId, hash::{BuildHasherDefault, Hasher}};



#[derive(Default)]
pub struct TypeMap<V> {
    map: hashbrown::HashMap<TypeId, V, BuildHasherDefault<TypeIdHasher>>,
}

impl<V> TypeMap<V> {
    pub fn has<K: 'static>(&self) -> bool {
        self.map.contains_key(&TypeId::of::<K>())
    }

    pub fn insert<K: 'static>(&mut self, value: V) {
        let _ = self.map.insert(TypeId::of::<K>(), value);
    }

    pub fn get<K: 'static>(&self) -> Option<&V> {
        self.map.get(&TypeId::of::<K>())
    }

    pub fn get_mut<K: 'static>(&mut self) -> Option<&mut V> {
        self.map.get_mut(&TypeId::of::<K>())
    }

    pub fn clear(&mut self) {
        self.map.clear();
    }
}



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
