use std::marker::PhantomData;




fn main() {}



// --- Core types



pub trait Object: 'static {}

pub trait Process {
    type Args;
    type Input;
    type Output;

    fn begin(&mut self, args: Self::Args) {}
    fn process(&mut self, input: Self::Input) -> Self::Output;
    fn end(&mut self) {}
}


pub struct Renderer<'pass> {
    _pass: &'pass mut PhantomData<()>,
}

pub trait Render {
    fn render(&mut self, renderer: &mut Renderer);
}

impl<'pass> Process for Renderer<'pass> {
    type Args = ();
    type Input = &'pass mut dyn Render;
    type Output = ();

    fn process(&mut self, input: Self::Input) -> Self::Output {
        input.render(self);
    }
}



// --- Utility types



// --- Implementation
