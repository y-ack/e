mod buffer;

use std::io;
use termion::raw::IntoRawMode;
use tui::Terminal;
use tui::backend::CrosstermBackend;

/// A window/visible buffer
pub struct Window<'a> {
    buffer: &'a Buffer,
}

/// A configuration for drawing a window for the interface


/// The full interface that will be rendered on the screen
pub struct Interface {
    windows: Vec::<Window>
}

impl Interface {
    fn new() -> Interface {
        let stdout = io::stdout().into_raw_mode();
        let backend = ;
    }

    fn draw(&self) {

    }
}

// we will only support drawing a single window right now as a render test
impl Window {
    fn new() -> Window {

    }
    fn draw(&self) {

    }
}