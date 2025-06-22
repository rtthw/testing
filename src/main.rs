


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

    if let EventResult::Consumed(Some(cb)) = editor.event(Event::Click) {
        (cb.0)(&mut state)
    }

    assert_eq!(state, State {
        clicks: 1,
        keys: 0,
    });

    if let EventResult::Consumed(Some(cb)) = editor.event(Event::Key) {
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

pub trait Element {
    fn render(&self, renderer: &mut u8);
    fn event(&mut self, event: Event) -> EventResult;
}

#[derive(Debug, PartialEq)]
pub enum Event {
    Click,
    Key,
}

pub enum EventResult {
    Ignored,
    Consumed(Option<Callback>),
}

impl EventResult {
    pub fn and(self, other: Self) -> Self {
        match (self, other) {
            (EventResult::Ignored, result)
            | (result, EventResult::Ignored) => result,
            (EventResult::Consumed(None), EventResult::Consumed(cb))
            | (EventResult::Consumed(cb), EventResult::Consumed(None)) => EventResult::Consumed(cb),
            (EventResult::Consumed(Some(cb1)), EventResult::Consumed(Some(cb2))) => {
                EventResult::Consumed(Some(Callback::new(move |state| {
                    (cb1.0)(state);
                    (cb2.0)(state);
                })))
            }
        }
    }
}



// --- Utility types



pub struct OnEvent<E> {
    element: E,
    callbacks: Vec<(Event, OnEventCallback<E>)>,
}

type OnEventCallback<T> = Arc<Box<dyn Fn(&mut T, &Event) -> Option<EventResult> + Send + Sync>>;

impl<E: Element> Element for OnEvent<E> {
    fn render(&self, renderer: &mut u8) {
        self.element.render(renderer);
    }

    fn event(&mut self, event: Event) -> EventResult {
        let element = &mut self.element;
        let callbacks = &self.callbacks;

        callbacks.iter()
            .filter(|&(ev, _)| ev == &event)
            .filter_map(|(_, cb)| (*cb)(element, &event))
            .fold(None, |s, r| match s {
                None => Some(r),
                Some(c) => Some(c.and(r)),
            })
            .unwrap_or_else(|| {
                EventResult::Ignored
            })
    }
}



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

    fn event(&mut self, event: Event) -> EventResult {
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

    fn event(&mut self, event: Event) -> EventResult {
        EventResult::Consumed(Some(match event {
            Event::Click => self.on_click.clone(),
            Event::Key => self.on_key.clone(),
        }))
    }
}
