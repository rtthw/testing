


use std::any::{Any, TypeId};



fn main() {
    let o = MyObject;
    assert!(<dyn Object>::is_dyn::<dyn Render>(&o));
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



pub trait Render {
    fn render(&self);
}



struct MyObject;

impl_object!(MyObject; Render);

impl Render for MyObject {
    fn render(&self) {}
}
