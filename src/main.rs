


use std::sync::Arc;



fn main() {
    let mut state = State {
        clicks: 0,
        keys: 0,
    };
    let mut editor = Editor {
        doc: Document {
            buffer: "Something".to_string(),
            on_click: Callback::new(|state| {
                state.clicks += 1;
            }),
            on_key: Callback::new(|state| {
                state.keys += 1;
            }),
        }
    };
    let mut renderer = 0;

    editor.render(&mut renderer);

    if let Some(cb) = editor.event(Event::Click) {
        (cb.0)(&mut state)
    }

    assert_eq!(state, State {
        clicks: 1,
        keys: 0,
    });

    if let Some(cb) = editor.event(Event::Key) {
        (cb.0)(&mut state)
    }

    assert_eq!(state, State {
        clicks: 1,
        keys: 1,
    });
}



// --- Core types



#[derive(Debug, PartialEq)]
pub struct State {
    clicks: u8,
    keys: u8,
}

#[derive(Clone)]
pub struct Callback(Arc<dyn Fn(&mut State) + Send + Sync>);

impl Callback {
    pub fn new<F>(f: F) -> Self
    where
        F: Fn(&mut State) + Send + Sync + 'static,
    {
        Callback(Arc::new(move |state| {
            f(state);
        }))
    }
}

trait Element {
    fn render(&self, renderer: &mut u8);
    fn event(&mut self, event: Event) -> Option<Callback>;
}

enum Event {
    Click,
    Key,
}



// --- Utility types







// --- Implementation



struct Editor {
    doc: Document,
}

impl Element for Editor {
    fn render(&self, renderer: &mut u8) {
        *renderer = renderer.wrapping_add(1);
        if *renderer < 3 {
            self.doc.render(renderer);
        }
    }

    fn event(&mut self, event: Event) -> Option<Callback> {
        self.doc.event(event)
    }
}

struct Document {
    buffer: String,

    on_click: Callback,
    on_key: Callback,
}

impl Element for Document {
    fn render(&self, renderer: &mut u8) {
        *renderer = renderer.wrapping_add(2);
    }

    fn event(&mut self, event: Event) -> Option<Callback> {
        Some(match event {
            Event::Click => self.on_click.clone(),
            Event::Key => self.on_key.clone(),
        })
    }
}
