use crossterm::{
	event::{read, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use io::stdout;
use std::{
	borrow::Borrow,
	io::{self, Stdout},
};
use tui::{backend::CrosstermBackend, layout::Rect};
use tui::{
	layout::{Constraint, Direction, Layout},
	terminal::Frame,
	Terminal,
};

use crate::buffer::Buffer;
use crate::window::Window;

pub struct WindowTree<'a> {
	pub window: Box<Window<'a>>,
	pub branch: Option<Box<&'a WindowTree<'a>>>,
	pub orientation: Direction,
}

/// The full interface that will be rendered on the screen
pub struct Interface<'a> {
	// TODO: we should probably have a some high level configuration for the
	// interface that specifies how to order the windows
	/// the visual windows in the interface
	pub root_window: WindowTree<'a>,
	// FIXME: this is temporary and should be moved to the editor class maybe
	// this is just so that we can destroy the interface during this testing
	pub running: bool,
	/// layout for tui-rs
	// layout: Layout,
	/// abstract interface to the terminal
	terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl<'a> Interface<'a> {
	pub fn new(scratch_buffer: &'a mut Buffer) -> Result<Interface<'a>, io::Error> {
		enable_raw_mode().unwrap();
		let stdout = io::stdout();
		let backend = CrosstermBackend::new(stdout);

		let interface = Interface {
			root_window: WindowTree {
				window: Box::new(Window::new(scratch_buffer)),
				branch: None,
				orientation: Direction::Vertical,
			},
			running: true,
			terminal: Terminal::new(backend)?,
		};
		execute!(io::stdout(), EnterAlternateScreen).unwrap();

		Ok(interface)
	}

	pub fn destroy(&self) {
		execute!(stdout(), LeaveAlternateScreen).unwrap();
		disable_raw_mode().unwrap();
	}

	pub fn draw(&mut self) -> Result<(), io::Error> {
		let root_window = Box::new(self.root_window.borrow());
		self.terminal.draw(|f| {
			fn generate_layouts<'b>(
				x: Box<&WindowTree<'b>>,
				layout: Rect,
				f: &mut Frame<CrosstermBackend<Stdout>>,
			) {
				let l = Layout::default()
					.direction(x.orientation.clone())
					.margin(0)
					.constraints(match x.branch {
						Some(_) => {
							[Constraint::Percentage(50), Constraint::Percentage(50)].as_ref()
						}
						None => [Constraint::Percentage(100)].as_ref(),
					})
					.split(layout);
				f.render_widget(x.window.get_widget(l[0]), l[0]);
				match x.branch.clone() {
					Some(b) => generate_layouts(b, l[1], f),
					None => {}
				}
			}
			generate_layouts(root_window, f.size(), f);
		})
	}

	pub fn clear(&mut self) -> Result<(), io::Error> {
		self.terminal.clear()
	}

	pub fn update(&mut self) -> crossterm::Result<()> {
		// `read()` blocks until an `Event` is available
		Ok(match read()? {
			Event::Key(event) => {
				if event == KeyCode::Char('q').into() {
					self.running = false;
				}
				// self.root_window.window.insert_at_cursor(event.code);
				// self.root_window.window.insert_at_cursor(event.code);
				self.draw().ok();
			}
			Event::Mouse(event) => println!("{:?}", event),
			Event::Resize(_, _) => {
				self.terminal
					.autoresize()
					.expect("Cannot reload the terminal successfully");
				self.draw().ok();
			}
		})
	}
}
