


use bog::prelude::*;



fn main() {
}



// --- Core types



pub struct UserInterface {
    root: Element,
    mouse_pos: Vec2,
}

impl UserInterface {
    pub fn new(root: Element) -> Self {
        Self {
            root,
            mouse_pos: Vec2::ZERO,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> Vec<Input> {
        match event {
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
}

pub enum Input {
    MouseMovement {
        /// The change in mouse position since the last `MouseMovement` input.
        delta: Vec2,
    },
}

pub struct Element {
    area: Rect,
}

impl Element {
    pub fn new(area: Rect) -> Self {
        Self {
            area,
        }
    }
}



// --- Implementation
