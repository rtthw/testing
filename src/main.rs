


use bog::prelude::*;
use slotmap::{SecondaryMap, SlotMap};



fn main() {
}



slotmap::new_key_type! { pub struct Id; }


// TODO: Maybe create a non-allocating tree? Would be faster but less flexible.
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
            area: Rect,
            parent: Option<Id>,
            nodes: &mut SlotMap<Id, NodeInfo>,
            children: &mut SecondaryMap<Id, Vec<Id>>,
            parents: &mut SecondaryMap<Id, Option<Id>>,
        ) -> Id {
            let axis = node.style.axis;
            let id = nodes.insert(NodeInfo {
                area,
                style: node.style,
            });
            let _ = parents.insert(id, parent);

            let length = match axis {
                Axis::Horizontal => area.w,
                Axis::Vertical => area.h,
            };
            let sizings = node.children.iter().map(|n| n.style.sizing.clone()).collect::<Vec<_>>();
            let sizes = resolve_sizes(length, sizings);

            let mut length_acc = 0.0;
            let mut node_children = Vec::with_capacity(node.children.len());
            for (child, child_length) in node.children.into_iter().zip(sizes.into_iter()) {
                let child_area = match axis {
                    Axis::Horizontal =>
                        Rect::new(vec2(length_acc, 0.0), vec2(child_length, area.h)),
                    Axis::Vertical =>
                        Rect::new(vec2(0.0, length_acc), vec2(area.w, child_length)),
                };
                length_acc += child_length;

                let child_id = digest(child, child_area, Some(id), nodes, children, parents);
                node_children.push(child_id);
            }

            let _ = children.insert(id, node_children);

            id
        }

        let root_id = digest(root, area, None, &mut nodes, &mut children, &mut parents);

        Self {
            root: root_id,
            nodes,
            children,
            parents,
        }
    }

    pub fn handle_resize(&mut self, area: Rect) -> Vec<Event> {
        // FIXME: Maybe don't early return here? (Only saves one allocation?)
        if self.nodes[self.root].area == area {
            return vec![];
        }

        fn inner(
            tree: &mut Tree,
            node: Id,
            area: Rect,
            events: &mut Vec<Event>,
        ) {
            if tree.nodes[node].area == area {
                return;
            }
            tree.nodes[node].area = area;

            events.push(Event::Resize { element: node });

            let axis = tree.nodes[node].style.axis;
            let length = match axis {
                Axis::Horizontal => area.w,
                Axis::Vertical => area.h,
            };
            let sizings = tree.children[node].iter()
                .map(|n| tree.nodes[*n].style.sizing.clone())
                .collect::<Vec<_>>();
            let sizes = resolve_sizes(length, sizings);

            let mut length_acc = 0.0;
            for (child_id, child_length) in tree.children[node].clone().into_iter()
                .zip(sizes.into_iter())
            {
                let child_area = match axis {
                    Axis::Horizontal => Rect::new(
                        vec2(area.x + length_acc, area.y),
                        vec2(child_length, area.h),
                    ),
                    Axis::Vertical => Rect::new(
                        vec2(area.x, area.y + length_acc),
                        vec2(area.w, child_length),
                    ),
                };
                length_acc += child_length;

                inner(tree, child_id, child_area, events);
            }
        }

        let mut events = vec![];

        inner(self, self.root, area, &mut events);

        events
    }
}

impl Tree {
    pub fn parent(&self, node: Id) -> Option<Id> {
        self.parents[node]
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


#[derive(Clone, Copy)]
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

#[cfg(test)]
#[test]
fn works() {
    let root_area = Rect::new(Vec2::ZERO, vec2(100.0, 50.0));
    let (left_area, right_area) = root_area.split_portion_h(0.2);

    let mut tree = Tree::new(
        Node::new()
            .with_style(Style::default().horizontal())
            .with_children(vec![
                Node::new()
                    .with_style(Style::default().portion_sized(0.2)),
                Node::new()
                    .with_style(Style::default().auto_sized()),
            ]),
        root_area,
    );


    let root_children = tree.children[tree.root].clone();
    assert!(root_children.len() == 2);

    let left = root_children[0];
    let right = root_children[1];

    assert_eq!(tree.nodes[left].area, left_area);
    assert_eq!(tree.nodes[right].area, right_area);


    let new_root_area = Rect::new(Vec2::ZERO, vec2(200.0, 10.0));
    let (new_left_area, new_right_area) = new_root_area.split_portion_h(0.2);

    let events = tree.handle_resize(new_root_area);

    assert_eq!(events, vec![
        Event::Resize { element: tree.root },
        Event::Resize { element: left },
        Event::Resize { element: right },
    ]);

    let root_children = tree.children[tree.root].clone();
    assert!(root_children.len() == 2);

    assert_eq!(left, root_children[0]);
    assert_eq!(right, root_children[1]);

    assert_ne!(tree.nodes[left].area, left_area);
    assert_ne!(tree.nodes[right].area, right_area);

    assert_eq!(tree.nodes[left].area, new_left_area);
    assert_eq!(tree.nodes[right].area, new_right_area);
}



#[derive(Debug, PartialEq)]
pub enum Event {
    Resize {
        element: Id,
    },
}
