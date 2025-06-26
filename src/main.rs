


use std::{marker::PhantomData, ptr::NonNull};



fn main() {
    let a = Box::new(A);
    let b = Box::new(B);
    let c = Box::new(C);
    let d = Box::new(D);
    let e = Box::new(E);

    let mut tree = Node::Branch(a, vec![
        Node::Branch(b, vec![
            Node::Leaf(d),
            Node::Leaf(e),
        ]),
        Node::Leaf(c),
    ]);

    let mut pass = RenderPass {
        shapes: Vec::new(),
    };

    println!("Pass (before): {:?}", pass.shapes);

    tree.crawl(&mut |object| {
        object.as_renderable().map(|obj| obj.render(&mut pass));
    });

    println!("Pass (after): {:?}", pass.shapes);
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

enum Node {
    Branch(Box<dyn Object>, Vec<Node>),
    Leaf(Box<dyn Object>),
}

impl Node {
    fn crawl(&mut self, func: &mut impl FnMut(&mut Box<dyn Object>)) {
        match self {
            Node::Branch(object, children) => {
                func(object);
                for child in children {
                    child.crawl(func);
                }
            }
            Node::Leaf(object) => {
                func(object);
            }
        }
    }
}



// --- Implementation



struct A;
impl Object for A {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> {
        Some(Renderable::new(self))
    }
}
impl Render for A {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push(1);
    }
}

struct B;
impl Object for B {}

struct C;
impl Object for C {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> {
        Some(Renderable::new(self))
    }
}
impl Render for C {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push(3);
    }
}

struct D;
impl Object for D {}

struct E;
impl Object for E {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> {
        Some(Renderable::new(self))
    }
}
impl Render for E {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push(5);
    }
}
