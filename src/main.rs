


use std::any::Any;



fn main() {
    let mut registry = Registry::new();
    let thing_class = registry.register_class::<Thing>();
    let thing_1_id = registry.create_object(thing_class);
    let thing_1_data_id = registry.objects[thing_class][thing_1_id]
        .downcast_ref::<__Thing>().unwrap().id;

    assert!(thing_1_id == thing_1_data_id)
}



// NOTE: Methods (not type) must be sized for dyn-compatibility.
pub trait Class: 'static {
    fn new(&self, id: usize) -> Box<dyn Any>;
    fn raw() -> Self where Self: Sized;
    fn static_ref(&self) -> &'static Self where Self: Sized;
}

pub struct Instance {
    pub object: usize,
    pub class: usize,
}

pub struct Registry {
    classes: Vec<&'static dyn Class>,
    objects: Vec<Vec<Box<dyn Any>>>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            classes: Vec::new(),
            objects: Vec::new(),
        }
    }

    pub fn register_class<T: Class>(&mut self) -> usize {
        let id = self.classes.len();
        self.objects.push(Vec::new());
        self.classes.push(T::raw().static_ref());
        id
    }

    pub fn create_object(&mut self, class: usize) -> usize {
        let id = self.objects[class].len();
        self.objects[class].push(self.classes[class].new(id));
        id
    }
}



struct Thing;

struct __Thing {
    id: usize,
}

impl Class for Thing {
    fn new(&self, id: usize) -> Box<dyn Any> {
        Box::new(__Thing {
            id,
        })
    }

    fn raw() -> Self where Self: Sized {
        Thing
    }

    fn static_ref(&self) -> &'static Self where Self: Sized {
        &Thing
    }
}



// ---



struct Data {
    registry: Registry,
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
