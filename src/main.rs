


use std::{marker::PhantomData, ptr::NonNull};



fn main() {
    run()
}



// --- Core types



pub trait Object {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> { None }
}

pub trait Render {
    fn render(&self, pass: &mut RenderPass);
}

pub struct RenderPass {
    shapes: Vec<u8>,
}

pub struct Renderable<'a> {
    _ref: PhantomData<&'a ()>,
    object: NonNull<()>,
    render: unsafe fn(NonNull<()>, &mut RenderPass),
}

impl<'a> Renderable<'a> {
    pub fn new<R: Render>(object: &'a R) -> Self {
        Self {
            _ref: PhantomData,
            object: NonNull::from(object).cast(),
            render: |obj, pass| unsafe { obj.cast::<R>().as_ref() }.render(pass),
        }
    }

    pub fn render(&self, pass: &mut RenderPass) {
        unsafe { (self.render)(self.object, pass) }
    }
}



// --- Implementation



struct A;

impl Render for A {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push(4);
    }
}

impl Object for A {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> {
        Some(Renderable::new(self))
    }
}



fn run() {
    let a = A;

    let mut pass = RenderPass {
        shapes: Vec::new(),
    };
    println!("Pass (before): {:?}", pass.shapes);
    if let Some(renderable) = a.as_renderable() {
        renderable.render(&mut pass);
    }
    println!("Pass (after): {:?}", pass.shapes);
}
