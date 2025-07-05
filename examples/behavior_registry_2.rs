


use hecs::{Entity, World};



fn main() {
    let mut data = Data {
        world: World::new(),
        effects: Vec::new(),
    };

    let steel_helmet = data.activate_effect::<SteelHelmet>();
    assert!(get_player_armor(&data.world) == 2.0);

    let steel_banner = data.activate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 4.0);

    data.deactivate_effect(steel_banner);
    assert!(get_player_armor(&data.world) == 2.0);

    let _ = data.activate_effect::<SteelBanner>();
    assert!(get_player_armor(&data.world) == 4.0);

    data.deactivate_effect(steel_helmet);
    assert!(get_player_armor(&data.world) == 0.0);

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
        let entity = (effect.activate)(self);
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
        (effect.effect.deactivate)(self, effect.entity);
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
