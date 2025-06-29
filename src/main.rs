


use bog::prelude::*;



fn main() {
}



pub struct UserInterface<I = &'static str>
where
    I: Copy,
{
    root: Element<I>,
    mouse_pos: Vec2,
}

impl<I: Copy> UserInterface<I> {
    pub fn new(root: Element<I>) -> Self {
        Self {
            root,
            mouse_pos: Vec2::ZERO,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Vec<Input<I>> {
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

    pub fn handle_mouse_move(&mut self, pos: Vec2) -> Vec<Input<I>> {
        if self.mouse_pos == pos {
            return Vec::new();
        }

        let delta = pos - self.mouse_pos;

        vec![Input::MouseMovement { delta }]
    }

    pub fn handle_resize(&mut self, area: Rect) -> Vec<Input<I>> {
        self.root.do_layout(area);
        Vec::new()
    }
}

pub enum Input<I = &'static str>
where
    I: Copy,
{
    MouseMovement {
        /// The change in mouse position since the last `MouseMovement` input.
        delta: Vec2,
    },
    MouseEnter {
        element: I,
    },
}

pub struct Element<I = &'static str>
where
    I: Copy,
{
    id: I,
    area: Rect,
    layout_size: Vec2,
    visual_size: Vec2,
    children: Vec<Element<I>>,
}

impl<I: Copy> Element<I> {
    pub fn new(id: I) -> Self {
        Self {
            id,
            area: Rect::NONE,
            layout_size: Vec2::ZERO,
            visual_size: Vec2::ZERO,
            children: Vec::new(),
        }
    }

    pub fn with_children(mut self, children: impl Into<Vec<Element<I>>>) -> Self {
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
