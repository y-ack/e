use std::{cell::RefCell, collections::HashMap, io::Stdout};

use io::stdout;
use libloading as lib;
use mlua::Lua;
use std::rc::Rc;
use tree_sitter::Language;
use tui::{backend::CrosstermBackend, Terminal};

use crate::{buffer::Buffer, pane::Pane};
use crossterm::{
	event::{read, Event, KeyCode},
	execute,
	terminal::{
		disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen, SetTitle,
	},
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
	pub current_pane: Rc<RefCell<Pane>>,
	pub running: bool,
	pub buffers: Vec<Rc<RefCell<Buffer>>>,
	tree_sitter_backends: HashMap<String, Language>,
	terminal: Terminal<CrosstermBackend<Stdout>>,
	lua: Lua,
}

impl Editor {
	/// Adds a new buffer to the Editor
	pub fn add_buffer<'b>(
		&mut self,
		content: &'b str,
		name: &'b str,
		language: Option<Language>,
	) -> Rc<RefCell<Buffer>> {
		let buffer = Rc::new(RefCell::new(Buffer::new(
			String::from(content),
			String::from(name),
			language,
		)));
		self.buffers.push(buffer.clone());
		buffer.clone()
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

	// TODO: Need to investigate why backend compiled from Lua tree sitter
	// segfaults later I guess
	/// Loads a TreeSitter backend from a file.
	pub fn get_tree_sitter_backend(filename: String, command_name: String) -> Language {
		unsafe {
			let lib = lib::Library::new(filename).unwrap();
			let func: lib::Symbol<unsafe extern "C" fn() -> Language> =
				lib.get(command_name.as_bytes()).unwrap();

			func()
		}
	}

	/// Loads a TreeSitter backend from a file.
	pub fn load_tree_sitter_backend(
		&mut self,
		filename: String,
		language: String,
		command_name: String,
	) {
		self.tree_sitter_backends.insert(
			language,
			Editor::get_tree_sitter_backend(filename, command_name),
		);
	}

	/// Read user input events passed to the Editor as well as update the Lua
	/// interpreter state.
	pub fn update(&mut self) -> crossterm::Result<()> {
		// update the title of the window with the set format
		execute!(
			stdout(),
			SetTitle({
				let current_pane = (*self.current_pane).borrow();
				let current_pane_name: String = (*current_pane.buffer).borrow().name.clone();
				format!("e - {}", current_pane_name).as_str()
			})
		)
		.unwrap();
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

		let mut tree_sitter_backends: HashMap<String, Language> = HashMap::new();
		let language = Editor::get_tree_sitter_backend(
			"./parser.so".to_string(),
			"tree_sitter_lua".to_string(),
		);
		println!("{}", language.version());
		tree_sitter_backends.insert(
			"lua".to_string(),
			Editor::get_tree_sitter_backend(
				"./parser.so".to_string(),
				"tree_sitter_lua".to_string(),
			),
		);

		let buffers: Vec<Rc<RefCell<Buffer>>> = vec![Rc::new(RefCell::new(Buffer::new(
			String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter.\nfunction hello()\n  print('hello, world!')\nend"),
			String::from("*scratch*"),
			Some(*tree_sitter_backends.get("lua").unwrap()),
		)))];
		let buffer = buffers[0].clone();
		let root_pane = Rc::new(RefCell::new(Pane::new(buffer)));

		let mut editor = Editor {
			root_pane: root_pane.clone(),
			current_pane: root_pane.clone(),
			running: true,
			buffers,
			tree_sitter_backends,
			terminal: Terminal::new(backend).unwrap(),
			lua: Lua::new(),
		};

		{
			let mut root_pane = (*editor.root_pane).borrow_mut();
			root_pane.split_window_horizontal();
		}
		{
			let buffer = editor.add_buffer("owo world", "*scratch-2*", None);
			let mut root_pane_mut = (*editor.root_pane).borrow_mut();
			root_pane_mut.branch = Some(Rc::new(RefCell::new(Pane::new(buffer))));
		}

		editor.enter();
		editor
	}
}
