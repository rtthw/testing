


use hecs::{Entity, World};



fn main() {
    let mut data = Data {
        world: World::new(),
        effects: Vec::new(),
    };

    let _steel_helmet = data.activate_effect::<SteelHelmet>();
    assert!(get_player_armor(&data.world) == 2.0);
    let steel_banner = data.activate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 4.0);
    data.deactivate_effect(steel_banner);
    assert!(get_player_armor(&data.world) == 2.0);
    println!("WORKS");
}



pub struct Data {
    pub world: World,
    pub effects: Vec<Effect>,
}

impl Data {
    // FIXME: Don't use indexing here, because it can (obviously) break.
    pub fn activate_effect<T: ToEffect>(&mut self) -> ActiveEffect {
        let effect = T::to_effect();
        let entity = (effect.activate)(&mut self.world);
        let index = self.effects.len();
        self.effects.push(effect);
        // (self.effects[index].on_equip)(self);

        ActiveEffect {
            effect,
            index,
            entity,
        }
    }

    // FIXME: Don't use indexing here, because it can (obviously) break.
    pub fn deactivate_effect(&mut self, effect: ActiveEffect) {
        let _ = self.effects.remove(effect.index);
        (effect.effect.deactivate)(&mut self.world, effect.entity);
    }
}



fn get_player_armor(world: &World) -> f32 {
    world.query::<&Armor>()
        .into_iter()
        .fold(0.0, |total, armor| total + armor.1.0)
}



#[derive(Clone, Copy)]
pub struct ActiveEffect {
    pub effect: Effect,
    pub index: usize,
    pub entity: Entity,
}

#[derive(Clone, Copy)]
pub struct Effect {
    pub activate: fn(&mut World) -> Entity,
    pub deactivate: fn(&mut World, Entity),
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
                    activate: |world| $name.activate(world),
                    deactivate: |world, entity| $name.deactivate(world, entity),
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
    fn activate(&self, world: &mut World) -> Entity {
        println!("Activating steel helmet...");
        world.spawn((
            Armor(2.0), Equippable, Steel, HeadSlot,
        ))
    }

    fn deactivate(&self, world: &mut World, entity: Entity) {
        println!("Deactivating steel helmet...");
        world.despawn(entity).unwrap();
    }
}

effect! {
    SteelBanner; activate,
}

impl SteelBanner {
    fn activate(&self, world: &mut World) -> Entity {
        println!("Activating steel banner...");
        for (_id, (armor, _steel)) in world.query_mut::<(&mut Armor, &Steel)>() {
            armor.0 *= 2.0;
        }
        world.spawn((
            Equippable, Steel, OffhandSlot,
        ))
    }

    fn deactivate(&self, world: &mut World, entity: Entity) {
        println!("Deactivating steel banner...");
        for (_id, (armor, _steel)) in world.query_mut::<(&mut Armor, &Steel)>() {
            armor.0 /= 2.0;
        }
        world.despawn(entity).unwrap();
    }
}
