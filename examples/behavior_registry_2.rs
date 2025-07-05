


use hecs::{Entity, World};



fn main() {
    let mut data = Data {
        world: World::new(),
        effects: Vec::new(),
    };

    data.activate_effect::<SteelHelmet>();
    assert!(get_player_armor(&data.world) == 2.0);

    data.activate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 4.0);

    data.deactivate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 2.0);

    data.activate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 4.0);

    data.deactivate_effect::<SteelHelmet>();
    assert!(get_player_armor(&data.world) == 0.0);

    println!("WORKS");
}



pub struct Data {
    pub world: World,
    pub effects: Vec<(Effect, Entity)>,
}

impl Data {
    pub fn activate_effect<T: ToEffect>(&mut self) {
        let effect = T::to_effect();
        let entity = (effect.activate)(self);

        debug_assert!(!self.effects.contains(&(effect, entity)));

        self.effects.push((effect, entity));

        // (self.effects[index].on_equip)(self);
    }

    pub fn deactivate_effect<T: ToEffect>(&mut self) {
        let effect = T::to_effect();

        let mut entity = None;
        self.effects.retain(|(eff, ent)| if eff == &effect {
            entity = Some(*ent);
            false
        } else {
            true
        });

        debug_assert!(entity.is_some());

        (effect.deactivate)(self, entity.unwrap());
    }
}



fn get_player_armor(world: &World) -> f32 {
    world.query::<&Armor>()
        .into_iter()
        .fold(0.0, |total, armor| total + armor.1.0)
}



#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Effect {
    pub activate: fn(&mut Data) -> Entity,
    pub deactivate: fn(&mut Data, Entity),
}

#[allow(unused)]
pub unsafe trait ToEffect {
    fn to_effect() -> Effect;
}

macro_rules! effect {
    ($name:ident; activate,) => {
        #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
        pub struct $name;

        unsafe impl ToEffect for $name {
            fn to_effect() -> Effect {
                Effect {
                    activate: |data| $name.activate(data),
                    deactivate: |data, entity| $name.deactivate(data, entity),
                }
            }
        }
    };
}



struct Equippable;
struct Armor(f32);
struct Steel;
struct HeadSlot;
struct OffhandSlot;



effect! {
    SteelHelmet; activate,
}

impl SteelHelmet {
    fn activate(&self, data: &mut Data) -> Entity {
        println!("Activating steel helmet...");
        data.world.spawn((
            Armor(2.0), Equippable, Steel, HeadSlot,
        ))
    }

    fn deactivate(&self, data: &mut Data, entity: Entity) {
        println!("Deactivating steel helmet...");
        data.world.despawn(entity).unwrap();
    }
}

effect! {
    SteelBanner; activate,
}

impl SteelBanner {
    fn activate(&self, data: &mut Data) -> Entity {
        println!("Activating steel banner...");
        for (_id, (armor, _steel)) in data.world.query_mut::<(&mut Armor, &Steel)>() {
            armor.0 *= 2.0;
        }
        data.world.spawn((
            Equippable, Steel, OffhandSlot,
        ))
    }

    fn deactivate(&self, data: &mut Data, entity: Entity) {
        println!("Deactivating steel banner...");
        for (_id, (armor, _steel)) in data.world.query_mut::<(&mut Armor, &Steel)>() {
            armor.0 /= 2.0;
        }
        data.world.despawn(entity).unwrap();
    }
}
