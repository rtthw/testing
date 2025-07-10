


use std::marker::PhantomData;

use hecs::{Entity, World};



fn main() {
    thread_assert::set_thread_id();
    thread_assert::same_thread();
    set_data(Data {
        world: World::new(),
    });

    let thing_a = Thing::new(); // &mut data().world
    assert!(thing_a.class() == Thing);
}



pub trait Class: Sized + 'static {
    fn new() -> Instance<Self>; // world: &mut World
    fn class() -> Self;
    fn class_ref() -> &'static Self;
}



#[derive(PartialEq)]
pub struct Thing;

pub struct __Thing<'a> {
    pub field: &'a u8,
}

impl Class for Thing {
    fn new() -> Instance<Self> { // world: &mut World
        let id = data().world.spawn(());

        Instance { id, _class: PhantomData }
    }

    fn class() -> Self { Thing }

    fn class_ref() -> &'static Self { &Thing }
}

impl Render for Thing {
    fn render(_this: &Instance<Self>) where Self: Class {
        println!("Rendering thing...");
    }
}



pub struct Instance<T: Class> {
    pub id: Entity,
    _class: PhantomData<T>,
}

impl<T: Class> Instance<T> {
    #[inline]
    pub fn class(&self) -> T {
        T::class()
    }
}

pub struct AnyInstance {
    pub id: Entity,
}



pub trait Render {
    fn render(this: &Instance<Self>) where Self: Class;
}



// ---



struct Data {
    world: World,
    // instances: Instances,
}

static mut DATA: Option<Data> = None;

#[allow(static_mut_refs)]
fn data() -> &'static mut Data {
    thread_assert::same_thread();
    unsafe { DATA.as_mut().unwrap_or_else(|| panic!()) }
}

fn set_data(data: Data) {
    unsafe { DATA = Some(data) };
}

mod thread_assert {
    static mut THREAD_ID: Option<std::thread::ThreadId> = None;

    pub fn set_thread_id() {
        unsafe {
            THREAD_ID = Some(std::thread::current().id());
        }
    }

    #[allow(static_mut_refs)]
    pub fn same_thread() {
        unsafe {
            thread_local! {
                static CURRENT_THREAD_ID: std::thread::ThreadId = std::thread::current().id();
            }
            assert!(THREAD_ID.is_some());
            assert!(THREAD_ID.unwrap() == CURRENT_THREAD_ID.with(|id| *id));
        }
    }
}
