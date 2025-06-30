


use bog::prelude::*;



fn main() {
}



pub struct UserInterface {
    root: Node,
    mouse_pos: Vec2,
    hovered: Vec<&'static str>,
}

impl UserInterface {
    pub fn new(root: Node) -> Self {
        Self {
            root,
            mouse_pos: Vec2::ZERO,
            hovered: Vec::new(),
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
        let mut inputs = vec![Input::MouseMovement { delta }];

        let new_hovered = self.root.list_under(pos, &|node| node.style.hoverable);
        if self.hovered != new_hovered {
            for element in &self.hovered {
                if !new_hovered.contains(element) {
                    inputs.push(Input::MouseLeave { element });
                }
            }
            for element in &new_hovered {
                if !self.hovered.contains(element) {
                    inputs.push(Input::MouseEnter { element });
                }
            }
            self.hovered = new_hovered;
        }

        inputs
    }

    pub fn handle_resize(&mut self, area: Rect) -> Vec<Input> {
        Vec::new()
    }
}

pub enum Input {
    MouseMovement {
        /// The change in mouse position since the last `MouseMovement` input.
        delta: Vec2,
    },
    MouseEnter {
        element: &'static str,
    },
    MouseLeave {
        element: &'static str,
    },
}

pub struct Node {
    name: &'static str,
    area: Rect,
    style: Style,
    children: Vec<Node>,
}

impl Node {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            area: Rect::NONE,
            style: Style::default(),
            children: Vec::new(),
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

    fn list_under(&self, point: Vec2, check_fn: &impl Fn(&Node) -> bool) -> Vec<&'static str> {
        if !self.area.contains(point) {
            return vec![];
        }

        fn inner(
            current: &Node,
            list: &mut Vec<&'static str>,
            point: Vec2,
            check_fn: &impl Fn(&Node) -> bool,
        ) {
            for child_area in current.children.iter() {
                if !child_area.area.contains(point) {
                    continue;
                }
                if check_fn(current) {
                    list.push(child_area.name);
                }
                inner(child_area, list, point, check_fn);
            }
        }

        let mut list = if check_fn(self) {
            vec![self.name]
        } else {
            vec![]
        };
        inner(self, &mut list, point, check_fn);

        list
    }
}

pub struct Style {
    pub size_request: Option<Vec2>,
    pub visual_size: Vec2,
    pub hoverable: bool,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size_request: None,
            visual_size: Vec2::ZERO,
            hoverable: true,
        }
    }
}
