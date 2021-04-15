use std::{cell::RefCell, io::Stdout};

use io::stdout;
use mlua::Lua;
use std::rc::Rc;
use tree_sitter::Language;
use tui::{backend::CrosstermBackend, Terminal};

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

/// A state of the editor that is used to manage the relationship
/// between components and Lua.
pub struct Editor {
	pub root_pane: Rc<RefCell<Pane>>,
	pub running: bool,
	pub buffers: Vec<Rc<RefCell<Buffer>>>,
	terminal: Terminal<CrosstermBackend<Stdout>>,
	lua: Lua,
}

impl Editor {
	/// Adds a new buffer to the Editor
	pub fn add_buffer(&mut self, content: String, name: String, language: Option<Language>) {
		let buffer = Rc::new(RefCell::new(Buffer::new(content, name, language)));
		self.buffers.push(buffer);
	}

	/// Draws the screen
	pub fn draw(&mut self) -> Result<(), io::Error> {
		let root_pane = (*self.root_pane).borrow();
		self.terminal.draw(|f| root_pane.draw_widgets(f.size(), f))
	}

	/// Clears the screen
	pub fn clear(&mut self) -> Result<(), io::Error> {
		self.terminal.clear()
	}

	/// Enters the Alternate Screen
	pub fn enter(&self) {
		execute!(io::stdout(), EnterAlternateScreen).unwrap();
		enable_raw_mode().unwrap();
	}

	/// Leaves the Alternate Screen
	pub fn leave(&self) {
		disable_raw_mode().unwrap();
		execute!(stdout(), LeaveAlternateScreen).unwrap();
	}

	/// Read user input events passed to the Editor as well as update the Lua
	/// interpreter state.
	pub fn update(&mut self) -> crossterm::Result<()> {
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
						.insert_at_cursor(match event.code {
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

impl Drop for Editor {
	fn drop(&mut self) {
		self.leave();
	}
}

impl Default for Editor {
	fn default() -> Self {
		let stdout = io::stdout();
		let backend = CrosstermBackend::new(stdout);
		// TODO: WE NEED TO MOVE BACKENDS SOMEWHERE ELSE
		let language_lua = unsafe { tree_sitter_lua() };

		let buffers: Vec<Rc<RefCell<Buffer>>> = vec![Rc::new(RefCell::new(Buffer::new(
			String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter.\nfunction hello()\n  print('hello, world!')\nend"),
			String::from("*scratch*"),
			Some(language_lua),
		)))];
		let buffer = buffers[0].clone();

		let editor = Editor {
			root_pane: Rc::new(RefCell::new(Pane::new(buffer))),
			running: true,
			buffers: buffers,
			terminal: Terminal::new(backend).unwrap(),
			lua: Lua::new(),
		};

		(*editor.root_pane).borrow_mut().split_window_horizontal();
		editor.enter();
		editor
	}
}
