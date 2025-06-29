


use bog::prelude::*;
use slotmap::{Key as _, SlotMap};



fn main() {
}



slotmap::new_key_type! { struct Node; }

pub struct UserInterface {
    root: Node,
    elements: SlotMap<Node, ElementNode>,
    mouse_pos: Vec2,
}

impl UserInterface {
    pub fn new(root: ElementNode) -> Self {
        let mut elements = SlotMap::with_capacity_and_key(16);
        let root = elements.insert(root);

        Self {
            root,
            elements,
            mouse_pos: Vec2::ZERO,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Vec<Input> {
        match event {
            InputEvent::Resize { width, height } => {
                self.handle_resize(Rect::new(Vec2::ZERO, vec2(width as _, height as _)))
            }
            InputEvent::MouseMove { x, y } => {
                self.handle_mouse_move(vec2(x, y))
            }
            _ => Vec::new(),
        }
    }

    pub fn handle_mouse_move(&mut self, pos: Vec2) -> Vec<Input> {
        if self.mouse_pos == pos {
            return Vec::new();
        }

        let delta = pos - self.mouse_pos;

        vec![Input::MouseMovement { delta }]
    }

    pub fn handle_resize(&mut self, area: Rect) -> Vec<Input> {
        self.elements[self.root].do_layout(area);
        Vec::new()
    }
}

pub enum Input {
    MouseMovement {
        /// The change in mouse position since the last `MouseMovement` input.
        delta: Vec2,
    },
    MouseEnter {
        element: u64,
    },
}

pub struct ElementNode {
    area: Rect,
    layout_size: Vec2,
    visual_size: Vec2,
    children: Vec<ElementNode>,
}

impl ElementNode {
    pub fn new() -> Self {
        Self {
            area: Rect::NONE,
            layout_size: Vec2::ZERO,
            visual_size: Vec2::ZERO,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: impl Into<Vec<ElementNode>>) -> Self {
        self.children = children.into();
        self
    }

    fn do_layout(&mut self, bounds: Rect) -> Layout {
        self.area = bounds;

        Layout {
            bounds,
            children: self.children.iter_mut().map(|e| e.do_layout(bounds)).collect(),
        }
    }
}

pub struct Layout {
    pub bounds: Rect,
    pub children: Vec<Layout>,
}



trait Element {
    fn layout(&self, bounds: Rect, renderer: &mut Renderer) -> Layout;
}
