use std::io::{self, Stdout};
use tui::{
    widgets::{Block, Borders, Widget},
    Frame, Terminal,
};
// FIXME: the problem with this backend is that it only supports linux
// we will probably need to use crosstermback later
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;

pub struct Buffer {
    content: String,
    name: String,
}

/// A window/visible buffer
pub struct Window<'a> {
    buffer: &'a Buffer,
    block_widget: Block<'a>,
}

impl<'a> Window<'a> {
    fn new(buffer: &'a Buffer) -> Window<'a> {
        Window {
            buffer: buffer,
            block_widget: Block::default()
                .title(buffer.name.clone())
                .borders(Borders::ALL),
        }
    }
}

/// A configuration for drawing a window for the interface

/// The full interface that will be rendered on the screen
pub struct Interface<'a> {
    // TODO: we should probably have a some high level configuration for the
    // interface that specifies how to order the windows
    /// the visual windows in the interface
    windows: Vec<Window<'a>>,
    /// abstract interface to the terminal
    terminal: Terminal<TermionBackend<RawTerminal<Stdout>>>,
}

impl<'a> Interface<'a> {
    pub fn draw(&mut self) -> Result<(), io::Error> {
        let widgets = self.windows[..].iter().map(|x| Box::new(&x.block_widget));
        self.terminal.draw(|f| {
            let size = f.size();
            for w in widgets {
                f.render_widget(*w, size);
            }
        })
    }

    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.terminal.clear()
    }
}

impl<'a> Default for Interface<'a> {
    fn default() -> Interface<'a> {
        let result: Result<Interface, io::Error> = (|| {
            let stdout = io::stdout().into_raw_mode()?;
            let backend = TermionBackend::new(stdout);

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
