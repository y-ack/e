use std::{borrow::Borrow, io::Stdout};

use io::stdout;
use mlua::Lua;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use tree_sitter::Language;
use tui::layout::Rect;
use tui::{backend::CrosstermBackend, layout::Direction, Terminal};
use tui::{
	layout::{Constraint, Layout},
	terminal::Frame,
};

use crate::{buffer::Buffer, window::Window};
use crossterm::{
	event::{read, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use std::io::{self};

extern "C" {
	fn tree_sitter_javascript() -> Language;
	fn tree_sitter_lua() -> Language;
}

pub struct WindowTree<'a> {
	pub window: Box<Window<'a>>,
	pub branch: Option<Box<&'a WindowTree<'a>>>,
	pub orientation: Direction,
}

pub struct Editor<'a> {
	pub root_window: WindowTree<'a>,
	pub running: bool,
	pub buffers: Vec<Arc<Mutex<Buffer<'a>>>>,
	terminal: Terminal<CrosstermBackend<Stdout>>,
	lua: Rc<Lua>,
}

impl<'a> Editor<'a> {
	pub fn destroy(&self) {
		execute!(stdout(), LeaveAlternateScreen).unwrap();
		disable_raw_mode().unwrap();
	}

	pub fn add_buffer(&mut self, content: String, name: String, language: Option<Language>) {
		let buffer = Arc::new(Mutex::new(Buffer::new(content, name, language, self.lua)));
		self.buffers.push(buffer);
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
				x.window.draw_widget(l[0], f);
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
				} else if event == KeyCode::Backspace.into() {
					self.root_window.window.delete_backwards_at_cursor(1);
				} else if event == KeyCode::Delete.into() {
					self.root_window.window.delete_forwards_at_cursor(1);
				} else {
					self.root_window.window.insert_at_cursor(match event.code {
						KeyCode::Char('a') => "a",
						KeyCode::Char('b') => "b",
						KeyCode::Char('c') => "c",
						KeyCode::Char('d') => "d",
						KeyCode::Char('-') => "-",
						KeyCode::Char(' ') => " ",
						KeyCode::Char('(') => "(",
						KeyCode::Char(')') => ")",
						_ => "hello! ",
					});
				}
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

impl<'a> Default for Editor<'a> {
	fn default() -> Self {
		enable_raw_mode().unwrap();
		let stdout = io::stdout();
		let backend = CrosstermBackend::new(stdout);
		// TODO: WE NEED TO MOVE BACKENDS SOMEWHERE ELSE
		let language_lua = unsafe { tree_sitter_lua() };
		let lua = Rc::new(Lua::new());

		let buffers: Vec<Arc<Mutex<Buffer>>> = vec![Arc::new(Mutex::new(Buffer::new(
			String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter.\nfunction hello()\n  print('hello, world!')\nend"),
			String::from("*scratch*"),
			Some(language_lua),
			lua
		)))];
		let buffer = buffers[0].clone();

		let editor = Editor {
			buffers: buffers,
			root_window: WindowTree {
				window: Box::new(Window::new(buffer)),
				branch: None,
				orientation: Direction::Vertical,
			},
			lua: lua,
			running: true,
			terminal: Terminal::new(backend).unwrap(),
		};
		execute!(io::stdout(), EnterAlternateScreen).unwrap();

		editor
	}
}
