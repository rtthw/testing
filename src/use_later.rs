


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



#[derive(Clone)]
pub struct Callback<D>(Arc<dyn Fn(&mut D) + Send + Sync>);

impl<D> Callback<D> {
    pub fn from_fn<F>(call: F) -> Self
    where
        F: Fn(&mut D) + Send + Sync + 'static,
    {
        Callback(Arc::new(move |data| call(data) ))
    }
}

impl<D> Deref for Callback<D> {
    type Target = dyn Fn(&mut D) + 'static;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}
