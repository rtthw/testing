


use std::marker::PhantomData;



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

pub unsafe trait Zst: 'static {
    fn owned() -> Self;
    fn static_ref() -> &'static Self;
}

// unsafe trait Cast<T> {
//     fn cast() -> T;
// }

macro_rules! define_zst {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        impl From<()> for $name {
            fn from(_value: ()) -> Self {
                $name
            }
        }

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

// trait ZstMarker: 'static {}

// impl<T: Zst> ZstMarker for T {}

// fn test() {
//     let dyn_zst = DynZst::<dyn Render>::new::<Something>();
// }

// pub struct DynZst<T: ?Sized> {
//     zst: (),
//     call: fn(()),
//     _marker: PhantomData<T>,
// }

// impl<T: ?Sized> DynZst<T> {
//     pub fn new<U: Zst + Cast<T>>() -> Self {
//         Self {
//             zst: (),
//             call: |_| U::cast().,
//             _marker: PhantomData,
//         }
//     }
// }

pub struct ZstRender {
    // zst: &'static dyn ZstMarker,
    zst: (),
    op: fn(()),
}

impl ZstRender {
    pub fn new<T: Zst + Render>() -> Self {
        Self {
            zst: (), // T::static_ref(),
            op: |_elided_zst| { T::owned().render(); },
        }
    }

    pub fn render(&self) {
        (self.op)(self.zst)
    }
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
