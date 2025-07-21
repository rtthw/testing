


use std::{any::Any, marker::PhantomData};



fn main() {
    let base_object = Box::new(Object {
        _marker: PhantomData::<BaseClass>,
    });
    let renderable_object = Box::new(Object {
        _marker: PhantomData::<Render>,
    });
    let other_object = Box::new(Object {
        _marker: PhantomData::<Thing>,
    });

    let objects: Vec<Box<dyn Any>> = vec![base_object, other_object, renderable_object];
    for (index, object) in objects.iter().enumerate() {
        if let Some(obj) = object.downcast_ref::<Object<Thing>>() {
            obj.thing(index);
        }
    }
    for (index, object) in objects.iter().enumerate() {
        if let Some(obj) = object.downcast_ref::<Object<Render>>() {
            obj.render(index);
        }
    }
}



#[derive(Clone, Copy, Debug)]
struct BaseClass;

struct Object<T = BaseClass> {
    _marker: PhantomData<T>,
}



macro_rules! declare {
    ($($name:ident),* $(,)?) => {
        $(
            #[derive(Clone, Copy, Debug)]
            pub struct $name;
        )*
    };
}



declare! {
    Thing,
    Other,
    Render,
}

impl Object<Thing> {
    fn thing(&self, index: usize) {
        println!("#{index} is a thing.");
    }
}

impl Object<Render> {
    fn render(&self, index: usize) {
        println!("Rendering #{index}...");
    }
}
