


use std::any::{Any, TypeId};

use bog::prelude::TypeMap;



fn main() {}



#[test]
fn works() {
    struct A {
        field_1: u8,
        field_2: &'static str,
    }

    trait Render {
        fn render(&mut self, renderer: Renderer);
    }

    struct Renderer(u8);

    impl Render for A {
        fn render(&mut self, renderer: Renderer) {
            self.field_1 += renderer.0;
        }
    }

    let def = {
        let mut define = DefineType::<A>::new();
        unsafe {
            define.field(|a| &mut a.field_1);
            define.field(|a| &mut a.field_2);
            define.method::<Renderer>(A::render as _);
        }
        define.finish()
    };

    assert!(def.is::<A>());

    {
        let mut a = A {
            field_1: 5,
            field_2: "Something",
        };

        assert_eq!(def.field::<u8>(&a).copied(), def.field_mut::<u8>(&mut a).copied());

        assert_eq!(def.field::<u8>(&a).copied(), Some(5));
        assert_eq!(def.field::<&'static str>(&a).copied(), Some("Something"));

        *def.field_mut::<u8>(&mut a).unwrap() = 7;
        *def.field_mut::<&'static str>(&mut a).unwrap() = "Other";

        assert_eq!(def.field::<u8>(&a), Some(&7));
        assert_eq!(def.field::<&'static str>(&a), Some(&"Other"));

        assert_eq!(def.field::<u8>(&a), Some(&a.field_1));
        assert_eq!(def.field::<&'static str>(&a), Some(&a.field_2));

        assert!(def.field::<usize>(&a).is_none());
    }

    {
        let mut a = A {
            field_1: 5,
            field_2: "Something",
        };

        assert_eq!(a.field_1, 5);

        let render_a = def.method::<Renderer>().unwrap();
        render_a(&mut a, Renderer(2));

        assert_eq!(a.field_1, 7);
    }
}



pub struct DefineType<T: 'static> {
    field_offsets: TypeMap<isize>,
    method_ptrs: TypeMap<usize>,
    _type: std::marker::PhantomData<T>,
}

impl<T: 'static> DefineType<T> {
    pub fn new() -> Self {
        Self {
            field_offsets: TypeMap::new(),
            method_ptrs: TypeMap::new(),
            _type: std::marker::PhantomData,
        }
    }

    pub unsafe fn field<U, F>(&mut self, f: F)
    where
        U: 'static,
        F: for<'a> FnOnce(&'a mut T) -> &'a mut U,
    {
        let offset = unsafe {
            let mut base = std::mem::MaybeUninit::<T>::uninit();
            let field = f(std::mem::transmute(base.as_mut_ptr())) as *mut _ as *mut u8;

            (field as *mut u8).offset_from(base.as_mut_ptr() as *mut u8)
        };
        self.field_offsets.insert::<U>(offset);
    }

    pub unsafe fn method<U: 'static>(&mut self, ptr: fn(&mut T, U)) {
        self.method_ptrs.insert::<U>(ptr as usize);
    }

    pub fn finish(self) -> TypeDefinition {
        TypeDefinition {
            id: TypeId::of::<T>(),
            field_offsets: self.field_offsets,
            method_ptrs: self.method_ptrs,
        }
    }
}

pub struct TypeDefinition {
    id: TypeId,
    field_offsets: TypeMap<isize>,
    method_ptrs: TypeMap<usize>,
}

impl TypeDefinition {
    pub fn is<T: 'static>(&self) -> bool {
        self.id == TypeId::of::<T>()
    }

    pub fn field<T: 'static>(&self, data: &dyn Any) -> Option<&T> {
        let offset = *self.field_offsets.get::<T>()?;
        Some(unsafe { &*((data as *const _ as *const u8).offset(offset) as *const T) })
    }

    pub fn field_mut<T: 'static>(&self, data: &mut dyn Any) -> Option<&mut T> {
        let offset = *self.field_offsets.get::<T>()?;
        Some(unsafe { &mut *((data as *mut _ as *mut u8).offset(offset) as *mut T) })
    }

    pub fn method<T: 'static>(&self) -> Option<fn(&mut dyn Any, T)> {
        let ptr = self.method_ptrs.get::<T>()?;
        Some(unsafe { std::mem::transmute(*ptr) })
    }
}
