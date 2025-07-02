


use std::any::{Any, TypeId};



fn main() {
    let o_1 = MyObject;
    let o_2 = OtherObject;
    assert!(<dyn Object>::is_dyn::<dyn Render>(&o_1));
    assert!(!<dyn Object>::is_dyn::<dyn Render>(&o_2));
}



trait Object {
    fn is_dyn_by_id(&self, id: TypeId) -> bool;
}

impl dyn Object {
    fn is_dyn<T: Any + ?Sized>(&self) -> bool {
        self.is_dyn_by_id(TypeId::of::<T>())
    }
}

macro_rules! impl_object {
    ($name:ty; $($type:path),+) => {
        impl Object for $name {
            fn is_dyn_by_id(&self, id: TypeId) -> bool {
                if false {
                    false
                }
                $(
                    else if id == TypeId::of::<dyn $type>() {
                        true
                    }
                )*
                else {
                    false
                }
            }
        }
    }
}



pub trait Marker {}

pub trait Render {
    fn render(&self);
}



struct MyObject;

impl_object!(MyObject; Render);

impl Render for MyObject {
    fn render(&self) {}
}

struct OtherObject;

impl_object!(OtherObject; Marker);
