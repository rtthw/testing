


use hecs::World;



fn main() {
    thread_assert::set_thread_id();
    thread_assert::same_thread();
    set_data(Data {
        world: World::new(),
    });
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
