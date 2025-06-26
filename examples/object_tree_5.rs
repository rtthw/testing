


use std::{marker::PhantomData, ptr::NonNull};



fn main() {
    let a = Box::new(A);
    let b = Box::new(B);
    let c = Box::new(C);
    let d = Box::new(D);
    let e = Box::new(E);

    let mut tree = {
        use Node::*;

        Branch(a, vec![
            Branch(b, vec![
                Leaf(d),
                Leaf(e),
            ]),
            Leaf(c),
        ])
    };

    let mut pass = RenderPass {
        shapes: Vec::new(),
    };

    println!("Pass (before): {:?}", pass.shapes);

    tree.crawl(&mut |object| {
        object.as_layoutable().map(|obj| println!("Layout: {}", obj.layout()));
        object.as_renderable().map(|obj| obj.render(&mut pass));
    });

    println!("Pass (after): {:?}", pass.shapes);
}



// --- Core types



pub trait Object {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> { None }
    fn as_layoutable(&self) -> Option<&dyn Layout> { None }
}

pub trait Render {
    fn render(&self, pass: &mut RenderPass);
}

pub struct RenderPass {
    shapes: Vec<char>,
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

pub trait Layout {
    fn layout(&self) -> &str;
}

pub struct Layoutable<'a> {
    _ref: PhantomData<&'a ()>,
    object: NonNull<()>,
    layout: unsafe fn(NonNull<()>) -> &'a str,
}

impl<'a> Layoutable<'a> {
    pub fn new<L: Layout>(object: &'a L) -> Self {
        Self {
            _ref: PhantomData,
            object: NonNull::from(object).cast(),
            layout: |obj| unsafe { obj.cast::<L>().as_ref() }.layout(),
        }
    }

    pub fn layout(&self) -> &str {
        unsafe { (self.layout)(self.object) }
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
    fn as_layoutable(&self) -> Option<&dyn Layout> {
        Some(self)
    }
}
impl Render for A {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push('A');
    }
}
impl Layout for A {
    fn layout(&self) -> &str {
        "A"
    }
}

struct B;
impl Object for B {
    fn as_layoutable(&self) -> Option<&dyn Layout> {
        Some(self)
    }
}
impl Layout for B {
    fn layout(&self) -> &str {
        "B"
    }
}

struct C;
impl Object for C {
    fn as_renderable<'a>(&'a self) -> Option<Renderable<'a>> {
        Some(Renderable::new(self))
    }
}
impl Render for C {
    fn render(&self, pass: &mut RenderPass) {
        pass.shapes.push('C');
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
        pass.shapes.push('E');
    }
}
