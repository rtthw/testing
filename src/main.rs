


use slotmap::{SecondaryMap, SlotMap};



fn main() {}



slotmap::new_key_type! { pub struct Id; }

pub struct Data {
    map: SlotMap<Id, Box<dyn Object>>,

    render: SecondaryMap<Id, Box<dyn Render>>,
}

impl Data {
    pub fn insert<T: ObjectImpl>(&mut self) -> Id {
        let object = T::new();

        self.map.insert(Box::new(object))
    }
}

pub struct DataSet<T> {
    map: SecondaryMap<Id, Box<T>>,
}



#[derive(Clone, Copy)]
pub struct Void;



pub trait Object: 'static {}

pub unsafe trait ObjectImpl: Object {
    fn new() -> Self;
}

pub unsafe trait Cast<T: ObjectImpl> {
    fn cast() -> T;
}
