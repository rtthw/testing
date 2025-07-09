//! Type map



use core::any::TypeId;



#[derive(Default)]
pub struct TypeMap<V> {
    map: rustc_hash::FxHashMap<TypeId, V>,
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
