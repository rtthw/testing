


use bog::prelude::*;
use slotmap::{SecondaryMap, SlotMap};



fn main() {
    let mut tree = Tree::new(
        Node::new()
            .with_style(Style::default().horizontal())
            .with_children(vec![
                Node::new(),
                Node::new(),
            ]),
        Rect::new(Vec2::ZERO, vec2(100.0, 50.0)),
    );
}



slotmap::new_key_type! { struct Id; }


pub struct Tree {
    root: Id,
    nodes: SlotMap<Id, NodeInfo>,
    children: SecondaryMap<Id, Vec<Id>>,
    parents: SecondaryMap<Id, Option<Id>>,
}

impl Tree {
    pub fn new(root: Node, area: Rect) -> Self {
        let mut nodes = SlotMap::with_capacity_and_key(16);
        let mut children = SecondaryMap::with_capacity(16);
        let mut parents = SecondaryMap::with_capacity(16);

        fn digest(
            node: Node,
            parent: Id,
            nodes: &mut SlotMap<Id, NodeInfo>,
            children: &mut SecondaryMap<Id, Vec<Id>>,
            parents: &mut SecondaryMap<Id, Option<Id>>,
        ) {
            let id = nodes.insert(NodeInfo {
                area: Rect::NONE, // TODO
                style: node.style,
            });
            let _ = parents.insert(id, Some(parent));
            let _ = children.insert(id, Vec::with_capacity(0));

            for child in node.children {
                digest(child, id, nodes, children, parents);
            }
        }

        let root_id = nodes.insert(NodeInfo {
            style: root.style,
            area,
        });
        parents.insert(root_id, None);
        for root_child in root.children {
            digest(root_child, root_id, &mut nodes, &mut children, &mut parents);
        }

        Self {
            root: root_id,
            nodes,
            children,
            parents,
        }
    }
}

pub struct Node {
    children: Vec<Node>,
    style: Style,
}

impl Node {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            style: Style::default(),
        }
    }

    pub fn with_children(mut self, children: impl Into<Vec<Node>>) -> Self {
        self.children = children.into();
        self
    }

    pub fn with_style(mut self, style: impl Into<Style>) -> Self {
        self.style = style.into();
        self
    }
}

struct NodeInfo {
    area: Rect,
    style: Style,
}



pub struct Style {
    pub sizing: Sizing,
    pub axis: Axis,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            sizing: Sizing::Auto,
            axis: Axis::Vertical,
        }
    }
}

impl Style {
    pub fn horizontal(mut self) -> Self {
        self.axis = Axis::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.axis = Axis::Vertical;
        self
    }

    pub fn auto_sized(mut self) -> Self {
        self.sizing = Sizing::Auto;
        self
    }

    pub fn exact_sized(mut self, length: f32) -> Self {
        self.sizing = Sizing::Exact(length);
        self
    }

    pub fn portion_sized(mut self, portion: f32) -> Self {
        self.sizing = Sizing::Portion(portion);
        self
    }
}



#[derive(Clone)]
pub enum Sizing {
    Auto,
    Exact(f32),
    Portion(f32),
}

impl Sizing {
    pub fn is_auto(&self) -> bool {
        matches!(self, Sizing::Auto)
    }

    pub fn as_exact(&self) -> Option<f32> {
        match self {
            Sizing::Exact(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_portion(&self) -> Option<f32> {
        match self {
            Sizing::Portion(n) => Some(*n),
            _ => None,
        }
    }
}


fn resolve_sizes(length: f32, sizings: Vec<Sizing>) -> Vec<f32> {
    let mut sizes = [length / sizings.len() as f32].repeat(sizings.len());

    let auto_count: usize = sizings.iter()
        .fold(0, |acc, s| if s.is_auto() { acc + 1 } else { acc });
    let mut remaining = length;

    for (i, exact) in sizings.iter()
        .enumerate()
        .filter_map(|(i, s)| Some((i, s.as_exact()?)))
    {
        sizes[i] = exact;
        remaining -= exact;
    }

    for (i, portion) in sizings.iter()
        .enumerate()
        .filter_map(|(i, s)| Some((i, s.as_portion()?)))
    {
        let size = length * portion;
        sizes[i] = size;
        remaining -= size;
    }

    let auto_size = remaining / auto_count as f32;
    for (i, _) in sizings.iter().enumerate().filter(|(_, s)| s.is_auto()) {
        sizes[i] = auto_size;
    }

    sizes
}


pub enum Axis {
    Horizontal,
    Vertical,
}


#[cfg(test)]
#[test]
fn sizing_resolver_works() {
    // NOTE: This is needed to account for floating point precision.
    let round_sizes = |length: f32, sizings: &[Sizing]| -> Vec<f32> {
        resolve_sizes(length, sizings.to_vec())
            .into_iter()
            .map(|s| (s * 10.0).round() / 10.0)
            .collect()
    };

    assert_eq!(
        round_sizes(12.0, &[Sizing::Auto, Sizing::Exact(4.0), Sizing::Portion(0.5)]),
        vec![2.0, 4.0, 6.0],
    );
    assert_eq!(
        round_sizes(12.0, &[Sizing::Auto, Sizing::Auto, Sizing::Portion(0.5)]),
        vec![3.0, 3.0, 6.0],
    );
    assert_eq!(
        round_sizes(12.0, &[Sizing::Auto, Sizing::Auto, Sizing::Auto]),
        vec![4.0, 4.0, 4.0],
    );
    assert_eq!(
        round_sizes(12.0, &[Sizing::Portion(0.4), Sizing::Portion(0.3), Sizing::Portion(0.2)]),
        vec![4.8, 3.6, 2.4],
    );
}
