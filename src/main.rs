


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
        let mut inputs = vec![];
        self.root.compute_layout(area, &mut inputs);

        inputs
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
    Resized {
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

    fn compute_layout(&mut self, area: Rect, inputs: &mut Vec<Input>) {
        if area == self.area {
            return;
        }
        self.area = area;
        inputs.push(Input::Resized { element: self.name });

        if !self.children.is_empty() {
            let rects = match self.style.orientation {
                Orientation::Horizontal => area.columns(self.children.len()),
                Orientation::Vertical => area.rows(self.children.len()),
            };

            assert_eq!(rects.len(), self.children.len());

            for (child, rect) in self.children.iter_mut().zip(rects.into_iter()) {
                child.compute_layout(rect, inputs);
            }
        }
    }
}

pub struct Style {
    pub size_request: Option<Vec2>,
    pub visual_size: Vec2,
    pub hoverable: bool,
    pub orientation: Orientation,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            size_request: None,
            visual_size: Vec2::ZERO,
            hoverable: true,
            orientation: Orientation::Vertical,
        }
    }
}

impl Style {
    pub fn horizontal(mut self) -> Self {
        self.orientation = Orientation::Horizontal;
        self
    }

    pub fn vertical(mut self) -> Self {
        self.orientation = Orientation::Vertical;
        self
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}



#[cfg(test)]
#[test]
fn works() {
    let root = Node::new("root")
        .with_style(Style::default().horizontal())
        .with_children(vec![
            Node::new("left"),
            Node::new("right"),
        ]);

    let root_area = Rect::new(Vec2::ZERO, vec2(10.0, 2.0));
    let (left_area, right_area) = root_area.split_len_h(5.0);

    let mut ui = UserInterface::new(root);
    ui.handle_resize(root_area);

    assert_eq!(
        ui.root.children.iter()
            .map(|n| (n.name, n.area))
            .collect::<Vec<_>>(),
        vec![
            ("left", left_area),
            ("right", right_area),
        ],
    );
}
