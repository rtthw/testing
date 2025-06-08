


use std::any::Any;



fn main() {}



// --- Core types



pub trait Object: 'static {
    type State;

    fn new(state: Self::State) -> Self;
    fn update(&mut self, state: Self::State) -> bool;
}



// --- Utility types



trait DummyObject {
    fn update(&mut self, state: Box<dyn Any>) -> bool;
}

impl<T: Object> DummyObject for T {
    #[inline]
    fn update(&mut self, state: Box<dyn Any>) -> bool {
        <Self as Object>::update(
            self,
            *state.downcast().expect("incorrect object state type when downcasting"),
        )
    }
}

trait ObjectTemplate {
    fn create(&mut self) -> Box<dyn DummyObject + 'static>;
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
    fn create(&mut self) -> Box<dyn DummyObject + 'static> {
        Box::new(T::new(self.take_state()))
    }

    fn state(&mut self) -> Box<dyn Any> {
        Box::new(self.take_state())
    }
}



// --- Implementation



pub struct Tree {
    root: Node,
}

impl Tree {
    pub fn update(&mut self) {
        self.root.crawl(&mut |object_template| {
            todo!()
        });
    }
}

enum Node {
    Container(Vec<Node>),
    Object(Box<dyn ObjectTemplate>),
}

impl Node {
    fn crawl(
        &mut self,
        func: &mut impl FnMut(&mut Box<dyn ObjectTemplate>),
    ) {
        match self {
            Self::Container(nodes) => {
                for child in nodes.iter_mut() {
                    child.crawl(func);
                }
            }
            Self::Object(template) => {
                func(template);
            }
        }
    }
}
