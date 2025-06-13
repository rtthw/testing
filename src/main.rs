


use std::{any::{Any, TypeId}, marker::PhantomData};


macro_rules! impl_upcast {
    ($(dyn $type:path),+) => {
        fn upcast_mut(&mut self, id: TypeId) -> Option<&mut dyn Any> {
            if false {
               None
            }
            $(
                else if id == TypeId::of::<dyn $type>() {
                    Some(unsafe { core::mem::transmute::<&mut dyn $type, &mut dyn Any>(
                        self as &mut dyn $type
                    ) })
                }
            )*
            else {
                None
            }
        }
    }
}

fn main() {
    struct A;
    impl Upcast for A {
        impl_upcast!(dyn Render);
    }
    impl Render for A {
        fn render(&mut self, pass: &mut RenderPass) {
            println!("Rendering A");
            pass.renderer.pass.push(1);
        }
    }
    struct B;
    impl Upcast for B {
        impl_upcast!(dyn Render);
    }
    impl Render for B {
        fn render(&mut self, pass: &mut RenderPass) {
            println!("Rendering B");
            pass.renderer.pass.push(2);
        }
    }
    struct C;
    impl Upcast for C {
        // C implements Render, but it isn't declared, so it won't be called.
        impl_upcast!(dyn EventHandler<Event>);
    }
    impl Render for C {
        fn render(&mut self, pass: &mut RenderPass) {
            println!("Rendering C");
            pass.renderer.pass.push(3);
        }
    }
    impl EventHandler<Event> for C {
        fn handle_event(&mut self, event: Event) -> bool {
            println!("Handling {:?} @ C", event);
            true
        }
    }


    let mut objects: Vec<Box<dyn Upcast>> = vec![];
    objects.push(Box::new(A));
    objects.push(Box::new(B));
    objects.push(Box::new(C));

    let mut app = App::begin(Tree { objects });

    app.process(AppInput::RenderRequest);
}



// --- Core types




pub trait Process {
    type Args;
    type Input;
    type Output;

    fn begin(args: Self::Args) -> Self;
    fn process(&mut self, input: Self::Input) -> Self::Output;
    fn end(&mut self) {}
}



// --- Utility types



pub trait Upcast {
    fn upcast_mut(&mut self, id: TypeId) -> Option<&mut dyn Any>;
}



// --- Implementation



pub struct RenderPass<'pass> {
    renderer: Renderer<'pass>,
}

pub struct Renderer<'pass> {
    pass: &'pass mut Vec<u8>, // Pretend this is a rendering pass.
}

pub trait Render {
    fn render(&mut self, pass: &mut RenderPass);
}

impl Render for Box<dyn Render> {
    fn render(&mut self, pass: &mut RenderPass) {
        self.as_mut().render(pass);
    }
}

impl<'pass> Process for RenderPass<'pass> {
    type Args = Renderer<'pass>;
    type Input = &'pass mut dyn Render;
    type Output = ();

    fn begin(args: Self::Args) -> Self {
        Self {
            renderer: args,
        }
    }

    fn process(&mut self, input: Self::Input) -> Self::Output {
        input.render(self);
    }
}



pub struct UpdatePass<'c, C> {
    context: &'c mut C,
}

pub trait Update {
    type Context;

    fn update(&mut self, context: &mut Self::Context);
}

impl<C> Update for Box<dyn Update<Context = C>> {
    type Context = C;

    fn update(&mut self, context: &mut Self::Context) {
        self.as_mut().update(context);
    }
}

impl<'c, C> Process for UpdatePass<'c, C> {
    type Args = &'c mut C;
    type Input = &'c mut dyn Update<Context = C>;
    type Output = ();

    fn begin(args: Self::Args) -> Self {
        Self {
            context: args,
        }
    }

    fn process(&mut self, input: Self::Input) -> Self::Output {
        input.update(self.context)
    }
}



pub struct Tree {
    pub objects: Vec<Box<dyn Upcast>>,
}

impl Tree {
    fn render(&mut self, render_pass: &mut Vec<u8>) {
        let mut pass = RenderPass::begin(Renderer { pass: render_pass });
        self.objects
            .iter_mut()
            .filter_map(|obj| {
                unsafe {
                    obj.as_mut()
                        .upcast_mut(TypeId::of::<dyn Render>())
                        .map(|dst| core::mem::transmute::<&mut dyn Any, &mut dyn Render>(dst))
                }
            })
            .for_each(|o| pass.process(o));
        pass.end();
    }

    // fn update<'pass, C: 'static>(&'pass mut self, pass: &'pass mut UpdatePass<'pass, C>) {
    //     self.objects
    //         .iter_mut()
    //         .filter_map(|o| o.as_mut().downcast_mut::<Box<dyn Update<Context = C>>>())
    //         .for_each(|o| pass.process(o));
    // }

    // fn handle_event(&mut self, event: Event) {
    //     let mut pass = EventPass::begin(event);
    //     self.objects
    //         .iter_mut()
    //         .filter_map(|o| o.as_mut().downcast_mut::<Box<dyn EventHandler<Event>>>())
    //         .for_each(|o| { pass.process(o); });
    //     pass.end();
    // }
}



pub struct App {
    tree: Tree,
}

pub enum AppInput {
    RenderRequest,
    // UpdateRequest,
    // Event(Event),
}

impl Process for App {
    type Args = Tree;
    type Input = AppInput;
    type Output = ();

    fn begin(args: Self::Args) -> Self {
        Self {
            tree: args,
        }
    }

    fn process(&mut self, input: Self::Input) -> Self::Output {
        match input {
            AppInput::RenderRequest => {
                self.tree.render(&mut vec![]);
            }
            // AppInput::UpdateRequest => {
            //     let mut nothing = ();
            //     let mut pass = UpdatePass::begin(&mut nothing);
            //     self.tree.update(&mut pass);
            // }
            // AppInput::Event(event) => {
            //     self.tree.handle_event(event);
            // }
        }
    }
}



pub trait EventHandler<E> {
    #[allow(unused)]
    fn handle_event(&mut self, event: E) -> bool { false }
}

impl<E> EventHandler<E> for Box<dyn EventHandler<E>> {
    fn handle_event(&mut self, event: E) -> bool {
        self.as_mut().handle_event(event)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    SomethingHappened(u8),
}

pub struct EventPass<'e, E> {
    event: E,
    _handler: PhantomData<&'e ()>,
}

impl<'e, E: Copy + 'e> Process for EventPass<'e, E> {
    type Args = E;
    type Input = &'e mut dyn EventHandler<E>;
    type Output = bool;

    fn begin(args: Self::Args) -> Self {
        Self {
            event: args,
            _handler: PhantomData,
        }
    }

    fn process(&mut self, input: Self::Input) -> Self::Output {
        input.handle_event(self.event)
    }
}
