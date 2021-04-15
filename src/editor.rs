use std::{
	borrow::Borrow,
	cell::{Ref, RefCell},
	io::Stdout,
};

use io::stdout;
use mlua::Lua;
use std::rc::Rc;
use tree_sitter::Language;
use tui::layout::Rect;
use tui::{backend::CrosstermBackend, Terminal};
use tui::{
	layout::{Constraint, Layout},
	terminal::Frame,
};

use crate::{buffer::Buffer, pane::Pane};
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

pub struct Editor {
	pub root_pane: Rc<RefCell<Pane>>,
	pub running: bool,
	pub buffers: Vec<Rc<RefCell<Buffer>>>,
	terminal: Terminal<CrosstermBackend<Stdout>>,
	lua: Lua,
}

impl Editor {
	pub fn destroy(&self) {
		execute!(stdout(), LeaveAlternateScreen).unwrap();
		disable_raw_mode().unwrap();
	}

	pub fn add_buffer(&mut self, content: String, name: String, language: Option<Language>) {
		let buffer = Rc::new(RefCell::new(Buffer::new(content, name, language)));
		self.buffers.push(buffer);
	}

	pub fn draw(&mut self) -> Result<(), io::Error> {
		let root_pane = (*self.root_pane).borrow();
		self.terminal.draw(|f| root_pane.draw_widgets(f.size(), f))
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
					(*self.root_pane).borrow_mut().delete_backwards_at_cursor(1);
				} else if event == KeyCode::Delete.into() {
					(*self.root_pane).borrow_mut().delete_forwards_at_cursor(1);
				} else {
					(*self.root_pane)
						.borrow_mut()
						.insert_at_cursor(String::from(match event.code {
							KeyCode::Char('a') => "a",
							KeyCode::Char('b') => "b",
							KeyCode::Char('c') => "c",
							KeyCode::Char('d') => "d",
							KeyCode::Char('-') => "-",
							KeyCode::Char(' ') => " ",
							KeyCode::Char('(') => "(",
							KeyCode::Char(')') => ")",
							_ => "hello! ",
						}));
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

impl Default for Editor {
	fn default() -> Self {
		enable_raw_mode().unwrap();
		let stdout = io::stdout();
		let backend = CrosstermBackend::new(stdout);
		// TODO: WE NEED TO MOVE BACKENDS SOMEWHERE ELSE
		let language_lua = unsafe { tree_sitter_lua() };
		let lua = Lua::new();

		let buffers: Vec<Rc<RefCell<Buffer>>> = vec![Rc::new(RefCell::new(Buffer::new(
			String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter.\nfunction hello()\n  print('hello, world!')\nend"),
			String::from("*scratch*"),
			Some(language_lua),
		)))];
		let buffer = buffers[0].clone();

		let editor = Editor {
			buffers: buffers,
			root_pane: Rc::new(RefCell::new(Pane::new(buffer))),
			lua: lua,
			running: true,
			terminal: Terminal::new(backend).unwrap(),
		};
		(*editor.root_pane).borrow_mut().split_window_vertical();
		execute!(io::stdout(), EnterAlternateScreen).unwrap();

		editor
	}
}
