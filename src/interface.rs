use crossterm::event::{read, Event};
use ropey::Rope;
use std::io::{self, Stdout};
use tree_sitter::{Language, Parser, Tree};
use tree_sitter_highlight::Highlighter;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent};
use tui::backend::CrosstermBackend;
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
    pub highlight_config: HighlightConfiguration,
}

impl Buffer {
    pub fn new(content: String, name: String) -> Buffer {
        let language = unsafe { tree_sitter_javascript() };
        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        let mut js_highlight_config = HighlightConfiguration::new(
            tree_sitter_javascript::language(),
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTION_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )
        .unwrap();

        let highlight_names: Vec<String> = ["function", "keyword"]
            .iter()
            .cloned()
            .map(String::from)
            .collect();

        js_highlight_config.configure(&highlight_names);

        Buffer {
            content: Rope::from_str(&content),
            name: name,
            parser: parser,
            highlight_config: js_highlight_config,
        }
    }

    pub fn get_tree(&mut self) -> Tree {
        self.parser
            .parse(self.content.clone().to_string(), None)
            .unwrap()
    }
}

/// A window/visible buffer
pub struct Window<'a> {
    buffer: &'a Buffer,
}

impl<'a> Window<'a> {
    pub fn new(buffer: &'a Buffer) -> Window<'a> {
        Window { buffer: buffer }
    }

    fn get_widget(&self) -> Paragraph {
        let text = Span::raw(self.buffer.content.clone());
        let mut highlighter = Highlighter::new();
        let highlights = highlighter
            .highlight(
                &self.buffer.highlight_config,
                b"const x = new Y();",
                None,
                |_| None,
            )
            .unwrap();
        for event in highlights {
            match event.unwrap() {
                HighlightEvent::Source { start, end } => {
                    eprintln!("source: {}-{}", start, end);
                }
                HighlightEvent::HighlightStart(s) => {
                    eprintln!("highlight style started: {:?}", s);
                }
                HighlightEvent::HighlightEnd => {
                    eprintln!("highlight style ended");
                }
            }
        }
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
