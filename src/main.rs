


fn main() {
    let mut something = Something;
    do_something(&mut something);
}



// --- Core types



pub trait Renderable {
    fn render(&mut self);
}

pub struct Render<'a, T: Renderable>(&'a mut T);

impl<'a, T: Renderable> Drop for Render<'a, T> {
    fn drop(&mut self) {
        println!("Rendering...");
        self.0.render();
    }
}



// --- Utility types



// --- Implementation



struct Something;

impl Renderable for Something {
    fn render(&mut self) {
        println!("Rendering something...");
    }
}

fn do_something(something: &mut Something) {
    Render(something);
}
