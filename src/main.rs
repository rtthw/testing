


use hecs::{Component as Member, ComponentError as MemberError, ComponentRef as MemberRef, Entity, World};



fn main() {
    let mut data = Data::new();

    let root = Object::spawn(&mut data);
    root.insert(&mut data, Something).unwrap();

    let _root_something = root.get::<&Something>(&data).unwrap();
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
    pub fn spawn(data: &mut Data) -> Self {
        Self(data.world.spawn(()))
    }

    pub fn get<'a, T: MemberRef<'a>>(&self, data: &'a Data) -> Result<T::Ref, MemberError> {
        Ok(data.world
            .entity(self.0)?
            .get::<T>()
            .ok_or_else(hecs::MissingComponent::new::<T::Component>)?)
    }

    pub fn insert(&self, data: &mut Data, member: impl Member) -> Result<(), hecs::NoSuchEntity> {
        data.world.insert_one(self.0, member)
    }
}



pub struct Something;
