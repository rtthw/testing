


use std::{any::{Any, TypeId}, collections::HashMap};



fn main() {}



// --- Core types



pub trait Object: 'static {
    type State;

    fn new(state: Self::State) -> Self;
    fn update(&mut self, state: Self::State) -> bool;
    fn resize(&mut self, area: Rect) -> bool;
}

// TODO: Separate IDs for objects of the same type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Id(TypeId);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}



// --- Utility types



trait DummyObject {
    fn update(&mut self, state: Box<dyn Any>) -> bool;
    fn resize(&mut self, area: Rect) -> bool;
}

impl<T: Object> DummyObject for T {
    #[inline]
    fn update(&mut self, state: Box<dyn Any>) -> bool {
        <Self as Object>::update(
            self,
            *state.downcast().expect("incorrect object state type when downcasting"),
        )
    }

    #[inline]
    fn resize(&mut self, area: Rect) -> bool {
        <Self as Object>::resize(self, area)
    }
}

trait ObjectTemplate {
    fn create(&mut self) -> Box<dyn DummyObject + 'static>;
    fn id(&self) -> Id;
    fn state(&mut self) -> Box<dyn Any>;
}

struct ObjectDef<T: Object> {
    state: Option<T::State>,
}

impl<T: Object> ObjectDef<T> {
    fn take_state(&mut self) -> T::State {
        let mut state = None;
        std::mem::swap(&mut state, &mut self.state);
        state.expect("should work")
    }
}

impl<T: Object> ObjectTemplate for ObjectDef<T> {
    #[inline]
    fn create(&mut self) -> Box<dyn DummyObject + 'static> {
        Box::new(T::new(self.take_state()))
    }

    #[inline]
    fn id(&self) -> Id {
        Id(TypeId::of::<T>())
    }

    #[inline]
    fn state(&mut self) -> Box<dyn Any> {
        Box::new(self.take_state())
    }
}



// --- Implementation



pub struct Tree {
    root: Node,
    objects: HashMap<Id, TreeObject>,
}

impl Tree {
    pub fn update(&mut self, area: Rect) {
        let Tree { root, objects } = self;

        root.crawl(area, &mut | ObjectNode { template, area } | {
            let id = template.id();
            let mut newly_created = false;
            let tree_object = objects.entry(id).or_insert_with(|| {
                let dummy = template.create();
                newly_created = true;
                TreeObject {
                    dummy,
                    area,
                }
            });

            if !newly_created {
                let mut changed = tree_object.dummy.update(template.state());
                if area != tree_object.area {
                    changed = tree_object.dummy.resize(area) || changed;
                }
            }
        });
    }
}

struct TreeObject {
    dummy: Box<dyn DummyObject>,
    area: Rect,
}

enum Node {
    Container(Vec<Node>),
    Object(Box<dyn ObjectTemplate>),
}

impl Node {
    fn crawl(&mut self, area: Rect, func: &mut impl FnMut(ObjectNode)) {
        match self {
            Self::Container(nodes) => {
                for child in nodes.iter_mut() {
                    // TODO: Do layout here.
                    child.crawl(area, func);
                }
            }
            Self::Object(template) => {
                func(ObjectNode {
                    template,
                    area,
                });
            }
        }
    }
}

struct ObjectNode<'a> {
    template: &'a mut Box<dyn ObjectTemplate>,
    area: Rect,
}
