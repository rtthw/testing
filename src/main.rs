


use bog::prelude::NoHashMap;
use slotmap::SlotMap;



fn main() {
    let mut data = Data::new();
    let root = data.spawn();

    root.request_render(&mut data.needs_render);
}



slotmap::new_key_type! { pub struct Id; }

impl Into<u64> for Id {
    fn into(self) -> u64 {
        self.0.as_ffi()
    }
}

impl Id {
    pub fn request_render(&self, needs_render: &mut NoHashMap<Id, bool>) {
        needs_render.get_mut(self).map(|flag| *flag = true);
    }
}

pub struct Data {
    pub objects: SlotMap<Id, ()>,
    pub needs_render: NoHashMap<Id, bool>,
}

impl Data {
    pub fn new() -> Self {
        Self {
            objects: SlotMap::with_capacity_and_key(16),
            needs_render: NoHashMap::with_capacity(16),
        }
    }

    pub fn spawn(&mut self) -> Id {
        self.objects.insert(())
    }
}
