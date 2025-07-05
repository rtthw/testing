


fn main() {}



pub struct Object {
    new: fn(&mut Registry),
}

#[allow(unused)]
pub unsafe trait ToObject {
    fn to_object() -> Object;
}

macro_rules! object {
    ($name:ident : $($type:path),*; register,) => {
        object!($name; register,);
        $(
            unsafe impl Cast<$type> for $name {
                fn cast() -> $type {
                    $type
                }
            }
        )*
    };
    ($name:ident; register,) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        unsafe impl ToObject for $name {
            fn to_object() -> Object {
                Object {
                    new: |reg| $name.new(reg),
                }
            }
        }
    };
}

pub struct Registry {

}

pub unsafe trait Cast<T> {
    fn cast() -> T;
}



object!(Thing; register,);

impl Thing {
    fn new(&self, registry: &mut Registry) {}
}

object!(Something; register,);

impl Something {
    fn new(&self, registry: &mut Registry) {}
}

object!(Other: Thing, Something; register,);

impl Other {
    fn new(&self, registry: &mut Registry) {
        <Self as Cast<Thing>>::cast().new(registry)
    }
}
