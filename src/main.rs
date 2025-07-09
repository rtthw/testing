


fn main() {
    render_something::<Something>();
}



fn render_something<T: Zst + Render>() {
    takes_a_static_reference(Box::leak(Box::new(T::owned())));
    takes_a_static_reference(T::static_ref());
    takes_any_reference(T::static_ref());
}

fn takes_a_static_reference(thing: &'static dyn Render) {
    thing.render()
}

fn takes_any_reference(thing: &dyn Render) {
    thing.render()
}

unsafe trait Zst: 'static {
    fn owned() -> Self;
    fn static_ref() -> &'static Self;
}

macro_rules! define_zst {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        unsafe impl Zst for $name {
            fn owned() -> Self {
                $name
            }

            fn static_ref() -> &'static Self {
                &$name
            }
        }
    };
}



pub trait Render {
    fn render(&self) {}
}

define_zst!(Something);

impl Render for Something {
    fn render(&self) {
        println!("One...");
    }
}
