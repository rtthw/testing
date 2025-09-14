


use std::any::{Any, TypeId};

use bog::prelude::TypeMap;



fn main() {}



#[test]
fn works() {
    struct A {
        field_1: u8,
        field_2: &'static str,
    }

    let def = {
        let mut define = DefineType::<A>::new();
        unsafe {
            define.field(|a| &mut a.field_1);
            define.field(|a| &mut a.field_2);
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
}



pub struct DefineType<T: 'static> {
    field_offsets: TypeMap<isize>,
    _type: std::marker::PhantomData<T>,
}

impl<T: 'static> DefineType<T> {
    pub fn new() -> Self {
        Self {
            field_offsets: TypeMap::new(),
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

    pub fn finish(self) -> TypeDefinition {
        TypeDefinition {
            id: TypeId::of::<T>(),
            field_offsets: self.field_offsets,
        }
    }
}

pub struct TypeDefinition {
    id: TypeId,
    field_offsets: TypeMap<isize>,
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
}
