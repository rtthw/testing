


pub struct Object(Rc<dyn Any>);

impl Object {
    pub fn new<T: Any + Sized>(object: T) -> Self {
        Self(Rc::new(object))
    }

    pub fn as_ref(&self) -> ObjectRef {
        ObjectRef(Rc::downgrade(&self.0))
    }

    pub fn typed<T>(self) -> TypedObject<T> {
        TypedObject {
            object: self,
            _type: PhantomData,
        }
    }

    pub fn typed_ref<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

pub struct TypedObject<T> {
    pub object: Object,
    _type: PhantomData<T>,
}

pub struct ObjectRef(Weak<dyn Any>);

pub struct TypedObjectRef<T> {
    pub object: ObjectRef,
    _type: PhantomData<T>,
}
