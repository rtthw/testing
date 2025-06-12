


use std::{
    any::{Any, TypeId},
    collections::HashMap,
    marker::PhantomData,
    sync::mpsc::{channel, Sender},
};



fn main() {
    struct TestMessage {
        num: u8,
    }

    impl Message for TestMessage {}

    struct TestObj {
        link: Link<Self>,
    }

    impl Object for TestObj {
        type Message = TestMessage;

        fn new(link: Link<Self>) -> Self {
            println!("[TestObj::new]");
            Self {
                link,
            }
        }

        fn update(&mut self, message: Self::Message) -> bool {
            println!("[TestObj::update] message = {}", message.num);
            true
        }

        fn resize(&mut self, area: Rect) -> bool {
            println!("[TestObj::resize] area = {:?}", area);
            self.link.send(TestMessage { num: area.w as u8 });
            true
        }
    }


    let (sender, _receiver) = channel();
    let mut tree = Tree {
        root: TestObj::node(),
        sender: Box::new(sender),
        objects: HashMap::with_capacity(5),
    };

    tree.update(Rect::ZERO);
    tree.update(Rect::ONE);
    tree.update(Rect::ZERO);
}



// --- Core types



pub trait Object: Sized + 'static {
    type Message;

    fn new(link: Link<Self>) -> Self;
    fn update(&mut self, message: Self::Message) -> bool;
    fn resize(&mut self, area: Rect) -> bool;
}

pub trait ObjectExt: Object {
    fn node() -> Node {
        Node::new::<Self>()
    }
}

impl<T: Object> ObjectExt for T {}

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

pub struct MessagePacket {
    object_id: Id,
    message: Box<dyn Any>,
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

impl Rect {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
    pub const ONE: Self = Self { x: 1.0, y: 1.0, w: 1.0, h: 1.0 };
}



// --- Utility types



trait DummyObject {
    fn update(&mut self, message: Box<dyn Any>) -> bool;
    fn resize(&mut self, area: Rect) -> bool;
}

impl<T: Object> DummyObject for T {
    #[inline]
    fn update(&mut self, message: Box<dyn Any>) -> bool {
        <Self as Object>::update(
            self,
            *message.downcast().expect("incorrect object message type when downcasting"),
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
}

struct ObjectDef<T: Object> {
    _obj: PhantomData<T>,
}

impl<T: Object> ObjectDef<T> {
    fn new() -> Self {
        Self {
            _obj: PhantomData,
        }
    }
}

impl<T: Object> ObjectTemplate for ObjectDef<T> {
    #[inline]
    fn create(&mut self, sender: Box<dyn MessageSender>) -> Box<dyn DummyObject + 'static> {
        Box::new(T::new(Link::new(sender)))
    }

    #[inline]
    fn id(&self) -> Id {
        Id(TypeId::of::<T>())
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
                println!("Creating new object #{id:?}...");
                let dummy = template.create(sender.clone_box());
                newly_created = true;
                TreeObject {
                    dummy,
                    area,
                }
            });

            if !newly_created {
                if area != tree_object.area {
                    let _changed = tree_object.resize(area);
                }
            }
        });
    }

    pub fn handle_message(&mut self, message: MessagePacket) {
        self.objects.get_mut(&message.object_id)
            .map(|o| o.dummy.update(message.message))
            .unwrap();
    }
}

pub struct Node(NodeInner);

impl Node {
    pub fn new<T: Object>() -> Self {
        Self(NodeInner::Object(Box::new(ObjectDef::<T>::new())))
    }

    pub fn with<T: Object>(children: impl IntoIterator<Item = Node>) -> Self {
        Self(NodeInner::Container {
            object: Box::new(ObjectDef::<T>::new()),
            children: children.into_iter().map(|n| n.0).collect(),
        })
    }
}

enum NodeInner {
    Container {
        object: Box<dyn ObjectTemplate>,
        children: Vec<NodeInner>
    },
    Object(Box<dyn ObjectTemplate>),
}

impl NodeInner {
    fn crawl(&mut self, area: Rect, func: &mut impl FnMut(ObjectNode)) {
        match self {
            Self::Container { object, children } => {
                func(ObjectNode {
                    template: object,
                    area,
                });
                for child in children.iter_mut() {
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

impl TreeObject {
    pub fn resize(&mut self, area: Rect) -> bool {
        self.area = area;
        self.dummy.resize(area)
    }
}

struct ObjectNode<'a> {
    template: &'a mut Box<dyn ObjectTemplate>,
    area: Rect,
}
