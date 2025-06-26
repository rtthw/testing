


use hecs::{Entity, World};



fn main() {
    let mut world = World::new();

    let root = world.spawn((
        Area::ZERO,
        Render {
            layer: 0,
        },
    ));
    let branch = world.spawn((
        Parent(root),
    ));
    let _leaf = world.spawn((
        Name("leaf"),
        Parent(branch),
        Area::ZERO,
        Render {
            layer: 1,
        },
    ));

    let mut pass = RenderPass {
        layer_fills: vec![Vec::new(), Vec::new()],
    };

    render(&mut world, &mut pass);
}



// --- Core types



pub struct Name(pub &'static str);

pub struct Parent(pub Entity);

pub struct Render {
    pub layer: usize,
}



// --- Utility types



#[derive(Clone, Copy, Debug)]
pub struct Area {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Area {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, w: 0.0, h: 0.0 };
}

struct RenderPass {
    layer_fills: Vec<Vec<Area>>,
}



// --- Implementation



fn render(world: &mut World, pass: &mut RenderPass) {
    for (_, (name, render, area)) in world.query_mut::<(Option<&Name>, &Render, &Area)>() {
        println!(
            "Rendering {} {area:?} at {}",
            name.map_or("unnamed", |name| name.0),
            render.layer,
        );
        pass.layer_fills[render.layer].push(*area)
    }
}
