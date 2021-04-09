use std::io::{self, Stdout};
use tui::{
    widgets::{Block, Borders},
    Terminal,
};
// FIXME: the problem with this backend is that it only supports linux
// we will probably need to use crosstermback later
use termion::raw::{IntoRawMode, RawTerminal};
use tui::backend::TermionBackend;

pub struct Buffer {
    content: dyn AsRef<u8>,
}

/// A window/visible buffer
pub struct Window {}

/// A configuration for drawing a window for the interface

/// The full interface that will be rendered on the screen
pub struct Interface {
    windows: Vec<Window>,
    /// abstract interface to the terminal
    terminal: Terminal<TermionBackend<RawTerminal<Stdout>>>,
}

impl Interface {
    pub fn draw(&mut self) -> Result<(), io::Error> {
        self.terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Block").borders(Borders::ALL);
            f.render_widget(block, size);
        })
    }

    pub fn clear(&mut self) -> Result<(), io::Error> {
        self.terminal.clear()
    }
}

impl Default for Interface {
    fn default() -> Interface {
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

// we will only support drawing a single window right now as a render test
impl Window {
    fn new() -> Window {
        return Window {};
    }
    fn draw(&self) {}
}
