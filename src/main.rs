


fn main() {
    set_context(Context {});

    let app = App::new();
    let game = Game::new();

    loop {
    }
}



pub struct Context {}

static mut CONTEXT: Option<Context> = None;

#[allow(static_mut_refs)]
fn get_context() -> &'static mut Context {
    // TODO: Assertion for main thread.
    unsafe { CONTEXT.as_mut().unwrap_or_else(|| panic!()) }
}

fn set_context(context: Context) {
    unsafe { CONTEXT = Some(context) };
}



#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Object {
    // new: fn(&mut Context),
}

pub struct Instance<T: ToObject> {
    _type: T,
    object: Object,
}

impl<T: ToObject> Instance<T> {
    pub fn new() -> Self {
        Self {
            _type: T::raw(),
            object: T::to_object(),
        }
    }
}

#[allow(unused)]
pub unsafe trait ToObject {
    fn raw() -> Self;
    fn to_object() -> Object;
}

pub trait Instantiate: ToObject {
    fn new() -> Instance<Self> where Self: Sized;
}

// TODO: TT muncher.
macro_rules! object {
    ($name:ident : $($super:path),* { $($member:path),* }) => {
        object!($name);
        $(
            unsafe impl Cast<$super> for $name {
                fn cast() -> $super {
                    $super
                }
            }
        )*
    };
    ($name:ident { $($member:path),* }) => {
        object!($name);

        $(
            unsafe impl Get<$member> for $name {
                fn get() -> $member {
                    // TODO: Fill out context.
                    let _context = get_context();
                    $member
                }
            }
        )*
    };
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        unsafe impl ToObject for $name {
            #[inline]
            fn raw() -> Self {
                Self
            }

            fn to_object() -> Object {
                Object {
                    // new: |reg| $name.new(reg),
                }
            }
        }

        impl Instantiate for $name {
            fn new() -> Instance<Self> {
                Instance::new()
            }
        }
    };
}

pub unsafe trait Cast<T> {
    fn cast() -> T;
}

pub unsafe trait Get<T> {
    fn get() -> T;
}



object!(App { Window });
object!(Game {});
object!(Window {});
object!(Entity {});
object!(Player: Entity {});
