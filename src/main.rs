


use std::{any::Any, marker::PhantomData};



fn main() {}



// --- Core types



pub trait Object: 'static {}

pub trait Process {
    type Args;
    type Input;
    type Output;

    fn begin(args: Self::Args) -> Self;
    fn process(&mut self, input: Self::Input) -> Self::Output;
    fn end(&mut self) {}
}



// --- Utility types



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
    objects: Vec<Box<dyn Any>>,
}

impl Tree {
    fn render(&mut self, render_pass: &mut Vec<u8>) {
        let mut pass = RenderPass::begin(Renderer { pass: render_pass });
        self.objects
            .iter_mut()
            .filter_map(|o| o.downcast_mut::<Box<dyn Render>>())
            .for_each(|o| pass.process(o));
        pass.end();
    }

    fn update<'pass, C: 'static>(&'pass mut self, pass: &'pass mut UpdatePass<'pass, C>) {
        self.objects
            .iter_mut()
            .filter_map(|o| o.downcast_mut::<Box<dyn Update<Context = C>>>())
            .for_each(|o| pass.process(o));
    }
}



pub struct App {
    tree: Tree,
}

pub enum AppInput {
    RenderRequest,
    UpdateRequest,
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
            AppInput::UpdateRequest => {
                let mut nothing = ();
                let mut pass = UpdatePass::begin(&mut nothing);
                self.tree.update(&mut pass);
            }
        }
    }
}
