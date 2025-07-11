


use std::{any::Any, cell::Cell, marker::PhantomData, ops::Drop};



fn main() {}

#[test]
fn test() {
    struct Thing {
        a: u8,
    }

    impl Node for Thing {
        fn ready(mut node: RefMut<Self>) where Self: Sized {
            node.a = 11;
        }

        fn update(mut node: RefMut<Self>) where Self: Sized {
            node.a += 1;
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Other {
        a: u8,
    }

    impl Node for Other {
        fn ready(mut node: RefMut<Self>) where Self: Sized {
            node.a = 13;
        }

        fn update(mut node: RefMut<Self>) where Self: Sized {
            node.a += 1;
        }
    }

    let thing_id;
    let other_id;

    {
        let thing = add_node(Thing { a: 4 });
        let other = add_node(Other { a: 5 });

        thing_id = thing.id.unwrap();
        other_id = other.id.unwrap();

        assert!(thing.lens(|thing| &mut thing.a).get() == Some(&mut 4));
        assert!(other.lens(|other| &mut other.a).get() == Some(&mut 5));

        update();

        assert!(thing.lens(|thing| &mut thing.a).get() == Some(&mut 12));
        assert!(other.lens(|other| &mut other.a).get() == Some(&mut 14));

        update();
    }

    assert!(find_node_by_type::<Thing>().unwrap().a == 13);
    assert!(find_node_by_type::<Other>().unwrap().a == 15);

    find_node_by_type::<Thing>().unwrap().add_member(Other { a: 1 });
    find_node_by_type::<Other>().unwrap().add_member(Other { a: 2 });

    assert!(find_nodes_by_type::<Other>().count() == 1);

    // Should always iterate in order.
    {
        let mut other_members = find_members::<Other>();
        assert!(other_members.next().unwrap().member == Other { a: 1 });
        assert!(other_members.next().unwrap().member == Other { a: 2 });
    }
    {
        let mut other_members = find_members::<Other>();
        assert!(other_members.next().unwrap().node == thing_id);
        assert!(other_members.next().unwrap().node == other_id);
    }
}



#[allow(unused)]
pub trait Node {
    fn ready(node: RefMut<Self>) where Self: Sized {}
    fn update(node: RefMut<Self>) where Self: Sized  {}
    fn render(node: RefMut<Self>) where Self: Sized  {}
}

trait NodeAny: Any + Node {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: Node + 'static> NodeAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}



#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Id {
    id: usize,
    generation: u64,
}

pub struct Handle<T: 'static> {
    id: Option<Id>,
    _marker: PhantomData<T>,
}

impl<T: 'static> std::fmt::Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.id)?;

        Ok(())
    }
}

impl<T: 'static> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        Handle {
            id: self.id,
            _marker: PhantomData,
        }
    }
}

impl<T: 'static> Copy for Handle<T> {}

unsafe impl<T: 'static> Send for Handle<T> {}

impl<T> Handle<T> {
    pub fn lens<F, U>(&self, f: F) -> Field<U>
    where
        F: for<'a> FnOnce(&'a mut T) -> &'a mut U,
    {
        assert!(self.id.is_some());

        let offset = unsafe {
            let mut base = std::mem::MaybeUninit::<T>::uninit();
            let field = f(std::mem::transmute(base.as_mut_ptr())) as *mut _ as *mut u8;

            (field as *mut u8).offset_from(base.as_mut_ptr() as *mut u8)
        };

        Field {
            handle: self.id.unwrap(),
            offset,
            _marker: PhantomData,
        }
    }
}

pub struct Field<T> {
    handle: Id,
    offset: isize,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    pub fn get(&mut self) -> Option<&mut T> {
        let node = get_untyped_node(self.handle)?;

        Some(unsafe { &mut *((node.data as *mut u8).offset(self.offset) as *mut T) })
    }
}



pub struct Member<T> {
    pub node: Id,
    pub member: T,
}

pub struct RefMut<T: 'static> {
    data: *mut T,
    handle: Handle<T>,
    members: *mut Vec<Box<dyn Any>>,
    used: *mut bool,
}

impl<T: 'static> RefMut<T> {
    pub fn add_member<S: Any + Copy>(&mut self, x: S) {
        unsafe { (*self.members).push(Box::new(x)) };
    }

    // pub fn set_flag(&self, flag: bool) {
    //     unsafe { get_scene() }.nodes[self.handle.id.unwrap().id]
    //         .as_mut()
    //         .unwrap()
    //         .flag = flag;
    // }

    pub fn delete(self) {
        assert!(self.handle.id.is_some());

        unsafe {
            *self.used = false;
        }
        unsafe { get_scene() }.delete(self.handle.id.unwrap());

        std::mem::forget(self);
    }
}

impl<T> std::ops::Deref for RefMut<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T> std::ops::DerefMut for RefMut<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: 'static> Drop for RefMut<T> {
    fn drop(&mut self) {
        assert_eq!(unsafe { *self.used }, true);
        unsafe {
            *self.used = false;
        }
    }
}

pub struct RefMutAny<'a> {
    data: *mut (),
    used: *mut bool,
    vtable: *mut (),
    members: *mut Vec<Box<dyn Any>>,
    handle: Id,

    _marker: PhantomData<&'a ()>,
}

impl<'a> RefMutAny<'a> {
    pub fn delete(self) {
        unsafe {
            *self.used = false;
        }
        unsafe { get_scene() }.delete(self.handle);
        std::mem::forget(self);
    }

    pub fn to_typed<T>(self) -> RefMut<T> {
        let res = RefMut {
            data: self.data as *mut T,
            handle: Handle {
                id: Some(self.handle),
                _marker: PhantomData,
            },
            members: self.members,
            used: self.used,
        };

        // NOTE: This reference is transferred to `RefMut`, its `Drop` call cleans up everything.
        std::mem::forget(self);

        res
    }
}

impl<'a> std::ops::Deref for RefMutAny<'a> {
    type Target = dyn Any;

    fn deref(&self) -> &Self::Target {
        let trait_obj: &dyn NodeAny = unsafe { std::mem::transmute((self.data, self.vtable)) };

        trait_obj.as_any()
    }
}

impl<'a> std::ops::DerefMut for RefMutAny<'a> {
    fn deref_mut(&mut self) -> &mut dyn Any {
        let trait_obj: &mut dyn NodeAny = unsafe { std::mem::transmute((self.data, self.vtable)) };

        trait_obj.as_any_mut()
    }
}

impl<'a> Drop for RefMutAny<'a> {
    fn drop(&mut self) {
        assert_eq!(unsafe { *self.used }, true);

        unsafe {
            *self.used = false;
        }
    }
}

struct Slot {
    id: Id,
    data: *mut (),
    vtable: *mut (),
    members: Vec<Box<dyn Any>>,
    ready: *const fn(RefMut<()>),
    update: *const fn(RefMut<()>),
    render: *const fn(RefMut<()>),
    virtual_drop: *const fn(*mut ()),
    data_len: usize,
    initialized: bool,
    used: *mut bool,
}

unsafe impl Sync for Scene {}

fn virtual_drop<T: Node + 'static>(data: *mut ()) {
    unsafe {
        std::ptr::drop_in_place(data as *mut T);
    }
}

impl Slot {
    fn new<T: Node + 'static>(id: Id, data: *mut (), vtable: *mut (), used: *mut bool) -> Self {
        Slot {
            id,
            data,
            vtable,
            members: vec![],
            ready: (&(Node::ready as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast(),
            update: (&(Node::update as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast(),
            render: (&(Node::render as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast(),
            virtual_drop: &(virtual_drop::<T> as fn(*mut ())) as *const fn(*mut ()),
            data_len: size_of::<T>(),
            initialized: false,
            used,
        }
    }

    fn put<T: Node + 'static>(&mut self, data: T) {
        assert!(size_of::<T>() <= self.data_len);

        let trait_obj = &data as &dyn NodeAny;
        let (_, vtable) = unsafe { std::mem::transmute::<_, (*mut (), *mut ())>(trait_obj) };

        self.vtable = vtable;
        self.ready = (&(Node::ready as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast();
        self.update = (&(Node::update as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast();
        self.render = (&(Node::render as fn(RefMut<T>)) as *const fn(RefMut<T>)).cast();
        self.virtual_drop = &(virtual_drop::<T> as fn(*mut ())) as *const fn(*mut ());

        unsafe {
            std::ptr::copy_nonoverlapping::<T>(&data as *const _ as *mut _, self.data as *mut _, 1);
        }
        self.id.generation += 1;
        self.initialized = false;

        self.members.clear();

        std::mem::forget(data);
    }
}

struct Scene {
    dense: Vec<Id>,
    dense_ongoing: Vec<Result<Id, Id>>,
    nodes: Vec<Option<Slot>>,
    arena: Arena,
    free_nodes: Vec<Slot>,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            dense: Vec::new(),
            dense_ongoing: Vec::new(),
            nodes: Vec::new(),
            arena: Arena::new(),
            free_nodes: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        for cell in &mut self.nodes {
            if let Some(cell) = cell.take() {
                assert!(unsafe { *cell.used == false });

                unsafe {
                    (*cell.virtual_drop)(cell.data);
                }
                let ix = self.dense.iter().position(|i| *i == cell.id).unwrap();
                self.dense.remove(ix);

                self.free_nodes.push(cell);
            }
        }
    }

    pub fn get_any(&mut self, handle: Id) -> Option<RefMutAny> {
        let cell = self.nodes.get_mut(handle.id)?;

        if cell.is_none() {
            return None;
        }
        let cell = cell.as_mut().unwrap();

        if cell.id.generation != handle.generation {
            return None;
        }

        if unsafe { *cell.used } {
            return None;
        }

        unsafe { *cell.used = true };

        Some(RefMutAny {
            data: cell.data,
            vtable: cell.vtable,
            members: &mut cell.members as _,
            handle: cell.id,
            used: cell.used,

            _marker: PhantomData,
        })
    }

    pub fn get<T>(&mut self, handle: Handle<T>) -> Option<RefMut<T>> {
        let ref_mut_any = self.get_any(handle.id?)?;
        Some(ref_mut_any.to_typed())
    }

    fn iter(&self) -> SceneIterator {
        SceneIterator {
            n: 0,
            len: self.dense.len(),
        }
    }

    fn add_node<T: Node + 'static>(&mut self, data: T) -> Handle<T> {
        let id;

        if let Some(i) = self
            .free_nodes
            .iter()
            .position(|free_node| free_node.data_len >= size_of::<T>())
        {
            let mut free_node = self.free_nodes.remove(i);

            free_node.put::<T>(data);

            id = free_node.id;

            self.nodes[id.id] = Some(free_node);
        } else {
            let trait_obj = &data as &dyn NodeAny;
            let (_, vtable) = unsafe { std::mem::transmute::<_, (*mut (), *mut ())>(trait_obj) };

            let ptr = self.arena.alloc(size_of::<T>()) as *mut _ as *mut T;
            unsafe {
                std::ptr::write(ptr, data);
            }
            let ptr = ptr as *mut ();
            let used = self.arena.alloc(1) as *mut _ as *mut bool;
            unsafe {
                std::ptr::write(used, false);
            }
            let used = used as *mut _ as *mut bool;

            id = Id {
                id: self.nodes.len(),
                generation: 0,
            };
            self.nodes.push(Some(Slot::new::<T>(id, ptr, vtable, used)));
        }

        self.dense.push(id);

        Handle {
            id: Some(id),
            _marker: PhantomData,
        }
    }

    pub fn delete(&mut self, id: Id) {
        if let Some(node) = self.nodes[id.id].take() {
            assert_eq!(node.id.generation, id.generation);

            self.dense_ongoing.push(Err(id));

            unsafe {
                (*node.virtual_drop)(node.data);
            }
            self.free_nodes.push(node);
        }
    }

    pub fn update(&mut self) {
        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.id].as_mut().unwrap();
            if cell.initialized == false {
                cell.initialized = true;

                let node: RefMut<()> = node.to_typed::<()>();
                unsafe { (*cell.ready)(node) };
            }
        }

        for node in &mut self.iter() {
            let cell = self.nodes[node.handle.id].as_mut().unwrap();
            let node: RefMut<()> = node.to_typed::<()>();
            unsafe { (*cell.update)(node) };
        }

        for id in self.dense_ongoing.drain(0..) {
            match id {
                Ok(id) => {
                    self.dense.push(id);
                }
                Err(id) => {
                    let ix = self.dense.iter().position(|i| *i == id).unwrap();
                    self.dense.remove(ix);
                }
            }
        }
    }
}



const ARENA_BLOCK: usize = 64 * 1024; // 64KiB

pub struct Arena {
    store: Cell<Vec<Vec<u8>>>,
    ptr: Cell<*mut u8>,
    offset: Cell<usize>,
}

impl Arena {
    pub fn new() -> Self {
        let mut store = vec![Vec::with_capacity(ARENA_BLOCK)];
        let ptr = store[0].as_mut_ptr();

        Arena {
            store: Cell::new(store),
            ptr: Cell::new(ptr),
            offset: std::cell::Cell::new(0),
        }
    }

    pub fn alloc(&self, size: usize) -> *mut u8 {
        // This should be optimized away for size known at compile time.
        if size > ARENA_BLOCK {
            return self.alloc_bytes(size);
        }

        let size = match size % size_of::<usize>() {
            0 => size,
            n => size + (size_of::<usize>() - n),
        };

        let offset = self.offset.get();
        let cap = offset + size;

        if cap > ARENA_BLOCK {
            self.grow();

            self.offset.set(size);
            self.ptr.get()
        } else {
            self.offset.set(cap);
            unsafe { self.ptr.get().add(offset) }
        }
    }

    #[inline]
    fn alloc_byte_vec(&self, mut val: Vec<u8>) -> *mut u8 {
        let ptr = val.as_mut_ptr();

        let mut temp = self.store.replace(Vec::new());
        temp.push(val);
        self.store.replace(temp);

        ptr
    }

    pub fn grow(&self) {
        let ptr = self.alloc_byte_vec(Vec::with_capacity(ARENA_BLOCK));
        self.ptr.set(ptr);
    }

    fn alloc_bytes(&self, size: usize) -> *mut u8 {
        self.alloc_byte_vec(Vec::with_capacity(size))
    }

    #[doc(hidden)]
    #[inline]
    pub unsafe fn offset(&self) -> usize {
        self.offset.get()
    }
}



pub struct SceneIterator {
    n: usize,
    len: usize,
}

impl Iterator for SceneIterator {
    type Item = RefMutAny<'static>;

    fn next(&mut self) -> Option<RefMutAny<'static>> {
        let scene = unsafe { get_scene() };
        let nodes = &mut scene.nodes;
        let dense = &scene.dense;
        if self.n >= self.len {
            return None;
        }
        let ix = dense[self.n];
        let cell = &mut nodes[ix.id];
        self.n += 1;

        if cell.is_none() {
            return self.next();
        }
        let cell = cell.as_mut().unwrap();

        if unsafe { *cell.used } {
            return self.next();
        }

        unsafe { *cell.used = true };

        Some(RefMutAny {
            data: cell.data,
            vtable: cell.vtable,
            members: &mut cell.members as _,
            handle: cell.id,
            used: cell.used,
            _marker: PhantomData,
        })
    }
}



static mut SCENE: Option<Scene> = None;

#[allow(static_mut_refs)]
unsafe fn get_scene() -> &'static mut Scene {
    unsafe { SCENE.get_or_insert_with(|| Scene::new()) }
}

pub fn allocated_memory() -> usize {
    unsafe { get_scene().arena.offset() }
}

pub fn clear() {
    unsafe { get_scene() }.clear()
}

/// Get node and panic if the node is borrowed or deleted
pub fn get_node<T: Node>(handle: Handle<T>) -> RefMut<T> {
    unsafe { get_scene() }
        .get(handle)
        .expect(&format!("No such node: {:?}", handle.id))
}

pub fn try_get_node<T: Node>(handle: Handle<T>) -> Option<RefMut<T>> {
    unsafe { get_scene() }.get(handle)
}

pub fn get_untyped_node(handle: Id) -> Option<RefMutAny<'static>> {
    unsafe { get_scene() }.get_any(handle)
}

pub fn add_node<T: Node>(node: T) -> Handle<T> {
    unsafe { get_scene() }.add_node(node)
}

pub fn update() {
    unsafe { get_scene() }.update()
}

pub fn all_nodes() -> SceneIterator {
    unsafe { get_scene() }.iter()
}

pub fn find_node_by_type<T: Any>() -> Option<RefMut<T>> {
    unsafe { get_scene() }
        .iter()
        .find(|node| node.is::<T>())
        .map(|node| node.to_typed())
}

pub fn find_members<T: Any + Copy>() -> impl Iterator<Item = Member<T>> {
    unsafe {
        get_scene().iter().filter_map(|node| {
            (*node.members)
                .iter()
                .find(|member| member.is::<T>())
                .map(|member| Member {
                    node: node.handle,
                    member: *member.downcast_ref::<T>().unwrap(),
                })
        })
    }
}

pub fn find_nodes_by_type<T: Any>() -> impl Iterator<Item = RefMut<T>> {
    unsafe { get_scene() }
        .iter()
        .filter(|node| node.is::<T>())
        .map(|node| node.to_typed())
}
