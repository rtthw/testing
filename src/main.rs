


use std::{any::{Any, TypeId}, collections::HashMap, mem::transmute};



fn main() {
    struct Print;

    struct A(u8);
    struct B(&'static str);

    impl Handle<Print> for A {
        fn handle(&mut self, _data: Print) { println!("A({});", self.0); }
    }
    impl Handle<Print> for B {
        fn handle(&mut self, _data: Print) { println!("B({});", self.0); }
    }

    let mut objects: Vec<Box<dyn Any>> = Vec::new();
    objects.push(Box::new(A(1)));
    objects.push(Box::new(B("Two")));

    let mut callbacks = Callbacks { map: HashMap::default() };
    callbacks.register::<A, Print>();
    callbacks.register::<B, Print>();

    for object in objects.iter_mut() {
        assert!(callbacks.call(object, Print));
    }
}



struct Callbacks {
    map: HashMap<(TypeId, TypeId), usize>,
}

impl Callbacks {
    fn register<T: Handle<U> + 'static, U: 'static>(&mut self) {
        self.map.insert((TypeId::of::<T>(), TypeId::of::<U>()), T::handle as usize);
    }

    fn call<U: 'static>(&self, data: &mut Box<dyn Any>, args: U) -> bool {
        let id = data.as_ref().type_id();
        if let Some(ptr) = self.map.get(&(id, TypeId::of::<U>())) {
            unsafe {
                (transmute::<usize, fn(&mut dyn Handle<U>, U)>(*ptr))(
                    transmute::<&mut dyn Any, &mut dyn Handle<U>>(data.as_mut()),
                    args,
                )
            }
            return true;
        }

        false
    }
}

trait Handle<T> {
    fn handle(&mut self, data: T);
}
