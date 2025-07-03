


use hecs::{Component as Member, ComponentError as MemberError, ComponentRef as MemberRef, Entity, World};



fn main() {
    let mut data = Data::new();

    let root = Object::new(&mut data);
    {
        let mut root_mut = root.as_mut(&mut data);
        root_mut.insert(Something).unwrap();
    }
    {
        let root_ref = root.as_ref(&data);
        let _root_something = root_ref.get::<&Something>().unwrap();
    }

    println!("SUCCESS");
}



pub struct Data {
    pub world: World,
}

impl Data {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
}



#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Object(Entity);

impl Object {
    pub fn new(data: &mut Data) -> Self {
        Self(data.world.spawn(()))
    }

    pub fn as_ref<'a>(&'a self, data: &'a Data) -> ObjectRef<'a> {
        ObjectRef {
            object: self,
            data,
        }
    }

    pub fn as_mut<'a>(&'a self, data: &'a mut Data) -> ObjectMut<'a> {
        ObjectMut {
            object: self,
            data,
        }
    }
}

pub struct ObjectRef<'a> {
    pub object: &'a Object,
    pub data: &'a Data,
}

impl<'a> ObjectRef<'a> {
    pub fn get<T: MemberRef<'a>>(&self) -> Result<T::Ref, MemberError> {
        Ok(self.data.world
            .entity(self.object.0)?
            .get::<T>()
            .ok_or_else(hecs::MissingComponent::new::<T::Component>)?)
    }
}

pub struct ObjectMut<'a> {
    pub object: &'a Object,
    pub data: &'a mut Data,
}

impl<'a> ObjectMut<'a> {
    pub fn insert(&mut self, member: impl Member) -> Result<(), hecs::NoSuchEntity> {
        self.data.world.insert_one(self.object.0, member)
    }
}



pub struct Something;
