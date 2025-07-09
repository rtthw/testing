


fn main() {
    let op = Op::new::<dyn Render, Something>();
    op.call(());
    let op = Op::new::<dyn Render, Another>();
    op.call(());
}



pub unsafe trait Zst: 'static {
    fn owned() -> Self;
    fn static_ref() -> &'static Self;
}

macro_rules! define_zst {
    ($name:ident) => { // ; $($trait:path),*
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

        // $(
        //     unsafe impl AsDyn<dyn $trait> for $name {
        //         fn as_dyn() -> &'static dyn $trait {
        //             &$name
        //         }
        //     }
        // )*
    };
}

// trait ZstMarker: 'static {}

// impl<T: Zst> ZstMarker for T {}

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


pub struct Op<I, O> {
    call: fn(I) -> O,
}

impl<I, O> Op<I, O> {
    pub fn new<D: ?Sized, T: Zst + Process<D, I, O>>() -> Self {
        Self {
            call: |input| T::static_ref().process(input),
        }
    }

    pub fn call(&self, input: I) -> O {
        (self.call)(input)
    }
}

pub trait Process<T: ?Sized, I, O> {
    fn process(&self, input: I) -> O;
}



// pub struct ZstRender {
//     // zst: &'static dyn ZstMarker,
//     zst: (),
//     op: fn(()),
// }

// impl ZstRender {
//     pub fn new<T: Zst + Render>() -> Self {
//         Self {
//             zst: (), // T::static_ref(),
//             op: |_elided_zst| { T::owned().render(); },
//         }
//     }

//     pub fn render(&self) {
//         (self.op)(self.zst)
//     }
// }



pub trait Render {
    fn render(&self) {}
}

impl<T: Render> Process<dyn Render, (), ()> for T {
    fn process(&self, _input: ()) -> () {
        self.render();
    }
}

define_zst!(Something);

impl Render for Something {
    fn render(&self) {
        println!("Rendering something...");
    }
}

define_zst!(Another);

impl Render for Another {
    fn render(&self) {
        println!("Rendering another...");
    }
}
