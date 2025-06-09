


use std::{any::TypeId, marker::PhantomData};



fn main() {}



// --- Core types



pub trait Object: Sized + 'static {
    type State;

    fn update(&mut self, state: Self::State) -> bool;
    fn resize(&mut self, area: Rect) -> bool;

    fn build(&self) -> Node {
        Node::leaf::<Self>()
    }

    fn diff(&self, node: &mut Node) {
        node.children.clear();
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Tag(TypeId);

impl Default for Tag {
    fn default() -> Self {
        Self::untyped()
    }
}

impl Tag {
    #[inline]
    pub fn typed<T: 'static>() -> Self {
        Self(TypeId::of::<T>())
    }

    #[inline]
    pub fn untyped() -> Self {
        Self(TypeId::of::<()>())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Rect {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, w: 1.0, h: 1.0 };
}



// --- Utility types



trait NodeObject {}

struct ObjectNodeDef<T: Object> {
    _obj: PhantomData<T>,
}

impl<T: Object> NodeObject for ObjectNodeDef<T> {}

impl<T: Object> ObjectNodeDef<T> {
    fn new() -> Self {
        Self {
            _obj: PhantomData,
        }
    }
}



// --- Implementation



pub struct Node {
    tag: Tag,
    object: Box<dyn NodeObject>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn leaf<T: Object>() -> Self {
        Self {
            tag: Tag::typed::<T>(),
            object: Box::new(ObjectNodeDef::<T>::new()),
            children: vec![],
        }
    }

    pub fn branch<T: Object>(children: impl IntoIterator<Item = Node>) -> Self {
        Self {
            tag: Tag::typed::<T>(),
            object: Box::new(ObjectNodeDef::<T>::new()),
            children: children.into_iter().collect(),
        }
    }

    pub fn diff<T: Object>(&mut self, new: &T) {
        if self.tag == Tag::typed::<T>() {
            new.diff(self);
        } else {
            *self = new.build();
        }
    }
}
