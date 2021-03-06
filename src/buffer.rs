use std::cmp::max;
use std::cmp::min;

use mlua::{prelude::LuaError, Lua, MetaMethod, ToLua, UserData, UserDataMethods};
use ropey::Rope;
use tree_sitter::{InputEdit, Language, Parser, Point, Tree};

/// State that the buffer can undo to
struct Revision<'a> {
	start_char: usize,
	old_end_char: usize,
	new_end_char: usize,
	text: &'a str,
}

/// An allocated area for text in the editor.
pub struct Buffer {
	pub content: Rope,
	pub name: String,
	filename: String,
	directory: String,
	pub parser: Option<Box<Parser>>,
	pub tree: Option<Box<Tree>>,
	// TODO: we need to support custom tab width rendering
	tabwidth: u8,
}

impl Buffer {
	/// Creates a new Buffer struct
	///
	/// * content - The initial content of the buffer
	/// * name - The name of the buffer
	/// * language - The [`Language`] used by the parser. If none is provided,
	///   then the buffer will render without syntax highlighting.
	pub fn new(content: String, name: String, language: Option<Language>) -> Buffer {
		match language {
			Some(v) => {
				let mut parser = Parser::new();
				parser.set_language(v).unwrap();
				let tree = parser.parse(content.clone(), None).unwrap();
				Buffer {
					content: Rope::from_str(&content),
					name: name,
					parser: Some(Box::new(parser)),
					tree: Some(Box::new(tree)),
					filename: String::from(""),
					directory: String::from(""),
					tabwidth: 4,
				}
			}
			None => Buffer {
				content: Rope::from_str(&content),
				name: name,
				parser: None,
				tree: None,
				filename: String::from(""),
				directory: String::from(""),
				tabwidth: 4,
			},
		}
	}

	/// A wrapper to converting a character index in the buffer content to
	/// a point
	pub fn point_to_char(&self, point: Point) -> usize {
		let index = self.content.line_to_char(point.row);
		index + point.column
	}

	/// Get an updated version of the parser tree
	///
	/// # Panics
	/// * Trying to access the tree when there is not already a tree
	///   in the buffer will cause a panic
	pub fn get_tree(&mut self) -> Tree {
		let parser = self.parser.as_mut().unwrap();

		parser
			.parse(
				self.content.clone().to_string(),
				Some(&self.tree.as_ref().unwrap()),
			)
			.unwrap()
	}

	/// A wrapper to converting a character index in the buffer content to
	/// a point
	pub fn char_to_point(&self, char: usize) -> Point {
		let row = self.content.char_to_line(char);
		let column = char - self.content.line_to_char(row);
		Point { row, column }
	}

	/// Replaces the region between the start and end byte in the buffer
	///
	/// # Examples
	/// ## Inserting text
	/// ```rust
	/// buffer.edit_region(0, 0, "Hello!");
	/// ```
	/// ## Replacing text
	/// ```rust
	/// // assume that we have "Hello!" in the buffer
	/// buffer.edit_region(1, 5, "i!")
	/// // the text will now be "Hi!"
	/// ```
	/// ## Deleting text
	/// ```rust
	/// // assume that we have "Hi! Hello!" in the buffer
	///	buffer.edit_region(0, 3, "");
	/// // the text will now be "Hello!"
	/// ```
	// FIXME: known bug that it is possible to edit outside of the region,
	// causing the program to crash
	pub fn edit_region<'b>(&mut self, start_char: usize, end_char: usize, text: &'b str) -> Point {
		let lowest_char = min(start_char, end_char);
		let highest_char = max(start_char, end_char);
		let lowest_byte = self.content.char_to_byte(lowest_char);
		let highest_byte = self.content.char_to_byte(highest_char);
		let lowest = self.char_to_point(lowest_char);
		let highest = self.char_to_point(highest_char);
		let (_, _, lowest_row_char_idx, _) = self.content.chunk_at_line_break(lowest.row);
		let (_, highest_row_byte_idx, _, _) = self.content.chunk_at_line_break(highest.row);

		match (self.parser.as_ref(), &mut self.tree.as_mut()) {
			(Some(_parser), Some(tree)) => {
				let edit = InputEdit {
					start_byte: lowest_byte,
					old_end_byte: highest_byte,
					new_end_byte: lowest_byte + text.len(),

					start_position: Point {
						row: lowest.row,
						column: lowest_char - lowest_row_char_idx,
					},
					old_end_position: Point {
						row: highest.row,
						column: highest_char - lowest_row_char_idx,
					},
					new_end_position: Point {
						row: self.content.byte_to_line(highest_byte + text.len()),
						column: self
							.content
							.byte_to_char(lowest_byte - highest_row_byte_idx + text.len()),
					},
				};
				tree.edit(&edit)
			}
			_ => (), // no parse language
		}

		// edit buffer content
		self.content.remove(lowest_char..highest_char);
		self.content.insert(lowest_char, text.clone());
		match (self.parser.as_ref(), self.tree.as_ref()) {
			(Some(_parser), Some(_tree)) => self.tree = Some(Box::new(self.get_tree())),
			_ => (),
		}

		self.char_to_point(end_char)
	}

	/// Inserts text in the buffer at the provided point
	pub fn insert_at_point<'b>(&mut self, point: Point, text: &'b str) -> Point {
		let index = self.content.line_to_char(point.row) + point.column;
		let mut point = self.edit_region(index, index, text);
		// ensure that we get the correct text position after the insert
		// TODO: this does not work for newlines lol
		point.column += text.len();
		point
	}

	/// Deletes n-length backwards in the text of the buffer at the provided
	/// point
	pub fn delete_backwards_at_point(&mut self, point: Point, n: usize) -> Point {
		let index = self.content.line_to_char(point.row) + point.column;
		self.edit_region(index, index - n, "")
	}

	/// Deletes n-length forwards in the text of the buffer at the provided
	/// point
	pub fn delete_forwards_at_point(&mut self, point: Point, n: usize) {
		let index = self.content.line_to_char(point.row) + point.column;
		self.edit_region(index, index + n, "");
	}
}

impl UserData for Buffer {
	fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
		methods.add_meta_method(
			MetaMethod::Index,
			|lua: &Lua, this: &Buffer, key: String| match key.as_ref() {
				"name" => Ok(this.name.as_str().to_lua(lua).unwrap()),
				_ => Err(LuaError::RuntimeError(String::from(":("))),
			},
		);
	}
}
