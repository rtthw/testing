


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
    _pass: &'pass mut PhantomData<()>,
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
    type Args = &'pass mut PhantomData<()>;
    type Input = &'pass mut dyn Render;
    type Output = ();

    fn begin(args: Self::Args) -> Self {
        Self {
            _pass: args,
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



struct Tree {
    objects: Vec<Box<dyn Any>>,
}

impl Tree {
    fn render<'pass>(&'pass mut self, pass: &'pass mut RenderPass<'pass>) {
        for renderable in self.objects
            .iter_mut()
            .filter_map(|o| o.downcast_mut::<Box<dyn Render>>())
        {
            pass.process(renderable)
        }
    }

    fn update<'pass, C: 'static>(&'pass mut self, pass: &'pass mut UpdatePass<'pass, C>) {
        for updateable in self.objects
            .iter_mut()
            .filter_map(|o| o.downcast_mut::<Box<dyn Update<Context = C>>>())
        {
            pass.process(updateable)
        }
    }
}
