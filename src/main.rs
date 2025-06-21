


fn main() {
    run()
}



// --- Core types



trait Process<I> {
    fn process(self, input: I) -> Self;
}



// --- Utility types



struct Renderer;

fn render<T: Process<Renderer>>(data: T) -> T {
    data.process(Renderer)
}

struct Edit;

fn edit<T: Process<Edit>>(data: T) -> T {
    data.process(Edit)
}



// --- Implementation



#[derive(Debug)]
struct Editor {
    doc: Document,
    lsp: Client,
}

impl Process<Renderer> for Editor {
    fn process(mut self, _input: Renderer) -> Self {
        self.doc.needs_render = false;
        self
    }
}

impl Process<Edit> for Editor {
    fn process(mut self, input: Edit) -> Self {
        self.lsp.edited = true;
        self.doc.buffer = self.doc.buffer.process(input);
        self
    }
}

#[derive(Debug)]
struct Document {
    buffer: Buffer,
    needs_render: bool,
}

#[derive(Debug)]
struct Buffer {
    content: String,
}

impl Process<Edit> for Buffer {
    fn process(mut self, _input: Edit) -> Self {
        self.content = "Rendered".to_string();
        self
    }
}

#[derive(Debug)]
struct Client {
    edited: bool,
}



fn run() {
    let mut editor = Editor {
        doc: Document {
            buffer: Buffer {
                content: "Something".to_string(),
            },
            needs_render: true,
        },
        lsp: Client {
            edited: false,
        },
    };

    println!("\nBEFORE: {:#?}", editor);
    editor = render(edit(editor));
    println!("\nAFTER: {:#?}", editor);
}
