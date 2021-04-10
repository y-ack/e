use crossterm::event::{read, Event};
use std::io::{self, Stdout};
use tui::backend::CrosstermBackend;
use tui::{
	layout::{Constraint, Direction, Layout},
	Terminal,
};

use crate::window::Window;

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
			if windows.len() != 0 {
				let layout = Layout::default()
					.direction(Direction::Vertical)
					.margin(0)
					.constraints([Constraint::Percentage(100)].as_ref())
					.split(f.size());
				for i in windows.iter().zip(layout.iter()) {
					let (w, c) = i;
					let widget = w.get_widget(*c);
					f.render_widget(widget, *c);
				}
			} else {
				panic!("There are windows to render!");
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
					self.terminal
						.autoresize()
						.ok()
						.expect("Cannot reload the terminal successfully");
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
			Err(_) => panic!("Unable to create an Interface instance. Cannot proceed."),
		}
	}
}
