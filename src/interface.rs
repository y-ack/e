use crossterm::event::{read, Event};
use ropey::Rope;
use std::io::{self, Stdout};
use tree_sitter::{Language, Parser, Tree};
use tui::{backend::CrosstermBackend, text::Spans};
use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Paragraph, Wrap},
    Terminal,
};

extern "C" {
    fn tree_sitter_javascript() -> Language;
}

pub struct Buffer {
    pub content: Rope,
    pub name: String,
    pub parser: Parser,
    pub tree: Tree,
}

impl Buffer {
    pub fn new(content: String, name: String) -> Buffer {
        let language = unsafe { tree_sitter_javascript() };
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();
        let tree = parser.parse(content.clone(), None).unwrap();

        Buffer {
            content: Rope::from_str(&content),
            name: name,
            parser: parser,
            tree: tree,
        }
    }

    pub fn get_tree(&mut self) -> Tree {
        self.parser
            .parse(self.content.clone().to_string(), Some(&self.tree))
            .unwrap()
    }
}

/// A window/visible buffer
pub struct Window<'a> {
    buffer: &'a Buffer,
    // TODO: should have Rect that defines viewport for the window
}

fn write_token<'a>(text: &'a str, token: &'static str) -> Span<'a> {
    Span::styled(
        text,
        Style::default().fg(match token {
            "function" => Color::Rgb(246, 199, 255),
            "identifier" => Color::Cyan,
            "string" => Color::Yellow,
            _ => Color::White,
        }),
    )
}

impl<'a> Window<'a> {
    pub fn new(buffer: &'a Buffer) -> Window<'a> {
        Window { buffer: buffer }
    }

    pub fn highlight(&self) -> Vec<Span> {
        let cursor = &mut self.buffer.tree.walk();
        let mut vector: Vec<Span> = vec![];
        let mut token_end = 0;
        loop {
            if cursor.node().kind() == "string" || !cursor.goto_first_child() {
                let start_byte = cursor.node().start_byte();
                if start_byte - token_end != 0 {
                    // eprintln!("end token: {} start_byte: {}", token_end, start_byte);
                    vector.push(Span::raw(
                        self.buffer
                            .content
                            .slice(token_end..start_byte)
                            .as_str()
                            .unwrap(),
                    ));
                }
                vector.push(write_token(
                    self.buffer
                        .content
                        .slice(start_byte..cursor.node().end_byte())
                        .as_str()
                        .unwrap(),
                    cursor.node().kind(),
                ));
                token_end = cursor.node().end_byte();
                while !cursor.goto_next_sibling() {
                    if !cursor.goto_parent() {
                        return vector;
                    }
                }
            }
        }
    }

    fn get_widget(&self) -> Paragraph {
        // let text = Span::raw(self.buffer.content.clone());
        let text = Spans::from(self.highlight());
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(self.buffer.name.clone())
                    .borders(Borders::ALL),
            )
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .wrap(Wrap { trim: true })
    }
}

/// A configuration for drawing a window for the interface

/// The full interface that will be rendered on the screen
pub struct Interface<'a> {
    // TODO: we should probably have a some high level configuration for the
    // interface that specifies how to order the windows
    /// the visual windows in the interface
    pub windows: Vec<Window<'a>>,
    /// layout for tui-rs
    // layout: Layout,
    /// abstract interface to the terminal
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl<'a> Interface<'a> {
    pub fn draw(&mut self) -> Result<(), io::Error> {
        let windows = &self.windows;
        self.terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(f.size());
            for i in windows.iter().zip(layout.iter()) {
                let (w, c) = i;
                let widget = w.get_widget();
                f.render_widget(widget, *c);
            }
        })
    }

    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.terminal.clear()
    }

    pub fn update(&mut self) -> crossterm::Result<()> {
        loop {
            // `read()` blocks until an `Event` is available
            match read()? {
                Event::Key(event) => println!("{:?}", event),
                Event::Mouse(event) => println!("{:?}", event),
                Event::Resize(_, _) => {
                    self.terminal.autoresize().ok().expect("oh well");
                    self.clear().ok();
                    self.draw().ok();
                }
            }
        }
    }
}

impl<'a> Default for Interface<'a> {
    fn default() -> Interface<'a> {
        let result: Result<Interface, io::Error> = (|| {
            let stdout = io::stdout();
            let backend = CrosstermBackend::new(stdout);

            let interface = Interface {
                windows: vec![],
                terminal: Terminal::new(backend)?,
            };

            Ok(interface)
        })();
        match result {
            Ok(v) => v,
            // FIXME: we should probably find a better way to handle errors
            // than just panic lol
            Err(_) => panic!(",,,,,,,,,,,,,,"),
        }
    }
}
