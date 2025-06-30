


use bog::prelude::*;
use slotmap::{SecondaryMap, SlotMap};



fn main() {
    let mut tree = Tree::new(
        Node::new()
            .with_children(vec![
                Node::new(),
                Node::new(),
            ])
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
    pub fn new(root: Node) -> Self {
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
            let id = nodes.insert(NodeInfo {}); // TODO: Fill out node info.
            let _ = parents.insert(id, Some(parent));
            let _ = children.insert(id, Vec::with_capacity(0));

            for child in node.children {
                digest(child, id, nodes, children, parents);
            }
        }

        let root_id = nodes.insert(NodeInfo {}); // TODO: Fill out node info.
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
}

impl Node {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: impl Into<Vec<Node>>) -> Self {
        self.children = children.into();
        self
    }
}

struct NodeInfo {}
