


use hecs::{Entity, World};



fn main() {
    let mut data = Data {
        world: World::new(),
        items: Vec::new(),
    };

    data.equip_item::<SteelHelmet>();
}



pub struct Data {
    pub world: World,
    pub items: Vec<Item>,
}

impl Data {
    pub fn equip_item<T: ToItem>(&mut self) {
        let item = T::to_item();
        let _entity = (item.create)(&mut self.world);
        // (item.on_equip)(self);
        let index = self.items.len();
        self.items.push(item);
        (self.items[index].on_equip)(self);
    }
}



#[derive(Clone, Copy)]
pub struct Item {
    pub create: fn(&mut World) -> Entity,
    pub on_equip: fn(&mut Data),
}

#[allow(unused)]
pub unsafe trait ToItem {
    fn to_item() -> Item;
}

macro_rules! item {
    ($name:ident; create, on_equip,) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        unsafe impl ToItem for $name {
            fn to_item() -> Item {
                Item {
                    create: |world| $name.create(world),
                    on_equip: |data| $name.on_equip(data),
                }
            }
        }
    };
}



struct Equippable;
struct Armor;
struct Steel;
struct HeadSlot;



item! {
    SteelHelmet; create, on_equip,
}

impl SteelHelmet {
    fn create(&self, world: &mut World) -> Entity {
        println!("Creating steel helmet...");
        world.spawn((
            SteelHelmet, Armor, Equippable, Steel, HeadSlot,
        ))
    }

    fn on_equip(&self, _data: &mut Data) {
        println!("Equipping steel helmet...");
    }
}
