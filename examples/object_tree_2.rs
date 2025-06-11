


use std::{any::TypeId, borrow::Borrow, fmt::Debug};



fn main() {}



// --- Core types



pub trait Object: Debug + 'static {
    fn tag(&self) -> Tag {
        Tag::untyped()
    }

    fn diff(&self, node: &mut Node) {
        node.children.clear();
    }

    fn children(&self) -> Vec<Node> {
        Vec::new()
    }
}

#[derive(Debug)]
pub struct DynObject {
    object: Box<dyn Object>,
}

impl DynObject {
    pub fn new(object: impl Object) -> Self {
        Self {
            object: Box::new(object),
        }
    }
}

impl<'a> Borrow<dyn Object + 'a> for DynObject {
    fn borrow(&self) -> &(dyn Object + 'a) {
        self.object.borrow()
    }
}

impl<'a> Borrow<dyn Object + 'a> for &DynObject {
    fn borrow(&self) -> &(dyn Object + 'a) {
        self.object.borrow()
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
    pub fn typed<T: ?Sized + 'static>() -> Self {
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



// --- Implementation



#[derive(Debug)]
pub struct Node {
    tag: Tag,
    pub children: Vec<Node>,
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.tag == other.tag
            && self.children == other.children
    }
}

impl Node {
    pub fn new<'a, T: Borrow<dyn Object> + 'a>(object: T) -> Self {
        let object = object.borrow();
        Self {
            tag: object.tag(),
            children: object.children(),
        }
    }

    pub fn diff<'a, T: Borrow<dyn Object> + 'a>(&mut self, new: T) {
        if self.tag == new.borrow().tag() {
            new.borrow().diff(self);
        } else {
            *self = Self::new(new);
        }
    }

    pub fn diff_children<'a, T: Borrow<dyn Object> + 'a>(&mut self, new_children: &[T]) {
        self.diff_children_custom(
            new_children,
            |node, object| node.diff(object.borrow()),
            |object| Self::new(object.borrow()),
        );
    }

    pub fn diff_children_custom<T>(
        &mut self,
        new_children: &[T],
        diff: impl Fn(&mut Node, &T),
        new_state: impl Fn(&T) -> Self,
    ) {
        if self.children.len() > new_children.len() {
            self.children.truncate(new_children.len());
        }

        for (child_state, new) in self.children.iter_mut().zip(new_children.iter()) {
            diff(child_state, new);
        }

        if self.children.len() < new_children.len() {
            self.children.extend(
                new_children[self.children.len()..].iter().map(new_state),
            );
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct A;
    #[derive(Debug)]
    struct B;
    #[derive(Debug)]
    struct C {
        children: Vec<DynObject>,
    }

    impl Object for A {}
    impl Object for B {}

    impl Object for C {
        fn diff(&self, node: &mut Node) {
            node.diff_children(&self.children);
        }
    }

    #[test]
    fn simple_diffing() {
        let a = DynObject::new(A);
        let b = DynObject::new(B);
        let mut node_a = Node::new(&a);
        let node_b = Node::new(&b);

        node_a.diff(&b);
        assert_eq!(node_a, node_b);

        node_a.diff(&a);
        assert_eq!(node_a, Node::new(a));
        assert_ne!(node_a, Node::new(b));
    }

    #[test]
    fn children_diffing() {
        let c = DynObject::new(C {
            children: vec![DynObject::new(A), DynObject::new(B)]
        });
        let mut node_c = Node::new(&c);
    }
}
