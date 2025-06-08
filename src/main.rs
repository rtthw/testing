


use std::{any::{Any, TypeId}, collections::HashMap, marker::PhantomData, sync::mpsc::Sender};



fn main() {}



// --- Core types



pub trait Object: Sized + 'static {
    type State;

    fn new(state: Self::State, link: Link<Self>) -> Self;
    fn update(&mut self, state: Self::State) -> bool;
    fn resize(&mut self, area: Rect) -> bool;
}

pub struct Link<T: Object> {
    sender: Box<dyn MessageSender>,
    _object: PhantomData<T>,
}

impl<T: Object> Link<T> {
    fn new(sender: Box<dyn MessageSender>) -> Self {
        Self {
            sender,
            _object: PhantomData,
        }
    }

    pub fn send(&self, message: impl Message + 'static) {
        self.sender.send(Box::new(message));
    }
}

pub trait Message: Send {}

pub trait MessageSender: Send + 'static {
    fn send(&self, message: Box<dyn Message>);

    fn clone_box(&self) -> Box<dyn MessageSender>;
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
    fn create(&mut self, sender: Box<dyn MessageSender>) -> Box<dyn DummyObject + 'static>;
    fn id(&self) -> Id;
    fn state(&mut self) -> Box<dyn Any>;
}

struct ObjectDef<T: Object> {
    state: Option<T::State>,
}

impl<T: Object> ObjectDef<T> {
    fn new(state: T::State) -> Self {
        Self {
            state: Some(state),
        }
    }

    fn take_state(&mut self) -> T::State {
        let mut state = None;
        std::mem::swap(&mut state, &mut self.state);
        state.expect("should work")
    }
}

impl<T: Object> ObjectTemplate for ObjectDef<T> {
    #[inline]
    fn create(&mut self, sender: Box<dyn MessageSender>) -> Box<dyn DummyObject + 'static> {
        Box::new(T::new(self.take_state(), Link::new(sender)))
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

impl MessageSender for Sender<Box<dyn Message>> {
    fn send(&self, message: Box<dyn Message>) {
        self.send(message)
            .expect("receiver needs to outlive senders for inter-component messaging");
    }

    fn clone_box(&self) -> Box<dyn MessageSender> {
        Box::new(self.clone())
    }
}



// --- Implementation



pub struct Tree {
    root: Node,
    objects: HashMap<Id, TreeObject>,
    sender: Box<dyn MessageSender>,
}

impl Tree {
    pub fn update(&mut self, area: Rect) {
        let Tree {
            ref mut root,
            ref mut objects,
            ref sender,
        } = *self;

        root.0.crawl(area, &mut | ObjectNode { template, area } | {
            let id = template.id();
            let mut newly_created = false;
            let tree_object = objects.entry(id).or_insert_with(|| {
                let dummy = template.create(sender.clone_box());
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

pub struct Node(NodeInner);

impl Node {
    pub fn with_state<T: Object>(state: T::State) -> Self {
        Self(NodeInner::Object(Box::new(ObjectDef::<T>::new(state))))
    }
}

enum NodeInner {
    Container(Vec<NodeInner>),
    Object(Box<dyn ObjectTemplate>),
}

impl NodeInner {
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

struct TreeObject {
    dummy: Box<dyn DummyObject>,
    area: Rect,
}

struct ObjectNode<'a> {
    template: &'a mut Box<dyn ObjectTemplate>,
    area: Rect,
}
