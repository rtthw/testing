


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

    impl<T: Render + 'static> Is<dyn Render> for T {
        #[inline(always)]
        fn as_mut(&mut self) -> &mut (dyn Render + 'static) {
            self
        }
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
            define.cast::<dyn Render>();
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

        let render = def.cast::<dyn Render>(&mut a).unwrap();
        render.render(Renderer(2));

        // let render_a = def.method_mut_1::<Renderer>().unwrap();
        // render_a(&mut a, Renderer(2));

        assert_eq!(a.field_1, 7);
    }
}



pub trait Is<T: ?Sized> {
    fn as_mut(&mut self) -> &mut T;
}

pub struct DefineType<T: 'static> {
    field_offsets: TypeMap<isize>,
    cast_ptrs: TypeMap<usize>,
    _type: std::marker::PhantomData<T>,
}

impl<T: 'static> DefineType<T> {
    pub fn new() -> Self {
        Self {
            field_offsets: TypeMap::new(),
            cast_ptrs: TypeMap::new(),
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

    pub fn cast<U: ?Sized + 'static>(&mut self) where T: Is<U> {
        fn inner<T: Is<U>, U: ?Sized>(data: &mut T) -> &mut U {
            data.as_mut()
        }
        self.cast_ptrs.insert::<U>(inner::<T, U> as _);
    }

    pub fn finish(self) -> TypeDefinition {
        TypeDefinition {
            id: TypeId::of::<T>(),
            field_offsets: self.field_offsets,
            cast_ptrs: self.cast_ptrs,
        }
    }
}

pub struct TypeDefinition {
    id: TypeId,
    field_offsets: TypeMap<isize>,
    cast_ptrs: TypeMap<usize>,
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

    pub fn cast<'a, T: ?Sized + 'static>(&self, data: &'a mut dyn Any) -> Option<&'a mut T> {
        let ptr = *self.cast_ptrs.get::<T>()?;
        Some(unsafe { (std::mem::transmute::<_, fn(&mut dyn Any) -> &mut T>(ptr))(data) })
    }
}
