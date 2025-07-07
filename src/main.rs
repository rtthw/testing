


use std::{any::Any, marker::PhantomData, ptr::NonNull};



fn main() {}



pub trait Object {
    fn init();
    fn update();
    fn render();
}

pub struct Mut<T: 'static> {
    _data: PhantomData<T>,
    members: *mut Vec<Box<dyn Any>>,
}



pub struct Node {
    members: Vec<Box<dyn Any>>,
    init_fn: *const fn(),
    update_fn: *const fn(),
    render_fn: *const fn(),
}

impl Node {
    fn new<T: Object + 'static>() -> Self {
        Self {
            members: Vec::new(),
            init_fn: (&(T::init as fn()) as *const fn()).cast(),
            update_fn: (&(T::update as fn()) as *const fn()).cast(),
            render_fn: (&(T::render as fn()) as *const fn()).cast(),
        }
    }
}



pub trait Render {
    fn render(&self);
}

// NOTE: Objects are always zero-sized, so that's why there's no need to store a lifetime here.
pub struct RenderObject {
    data: NonNull<()>,
    render_fn: unsafe fn(NonNull<()>),
}

impl RenderObject {
    pub fn new<T: Render>(object: &T) -> Self {
        Self {
            data: NonNull::from(object).cast(),
            render_fn: |data| unsafe {
                data.cast::<T>().as_ref().render();
            }
        }
    }

    pub fn render(&self) {
        unsafe { (self.render_fn)(self.data) }
    }
}



pub struct App;

impl Render for App {
    fn render(&self) {
        println!("Rendering app...")
    }
}
