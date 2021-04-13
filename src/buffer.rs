use std::cmp::{self, max};
use std::rc::Rc;
use std::{borrow::Cow, cmp::min};

use mlua::Lua;
use ropey::Rope;
use ropey::RopeSlice;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Query, QueryCursor, Tree};
use tui::{
	style::{Color, Style},
	text::{Span, Spans},
};

/// State that the buffer can undo to
struct Revision<'a> {
	start_byte: usize,
	old_end_byte: usize,
	new_end_byte: usize,
	text: &'a str,
}

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

// TODO: need to actually make a Theme dict lol
fn write_token<'a, T>(text: T, token: &'static str) -> Span<'a>
where
	T: Into<Cow<'a, str>>,
{
	Span::styled(
		text,
		Style::default().fg(match token {
			"function" => Color::Rgb(246, 199, 255),
			"identifier" => Color::Cyan,
			"string" => Color::Yellow,
			"comment" => Color::Green,
			_ => Color::White,
		}),
	)
}

fn clamp(v: usize, x: usize, y: usize) -> usize {
	if v < x {
		x
	} else if v > y {
		y
	} else {
		v
	}
}

impl Buffer {
	pub fn new<'b>(content: String, name: String, language: Option<Language>) -> Buffer {
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

	pub fn highlight<'b>(&self, node: Node, start: usize, end: usize) -> Vec<Span> {
		let cursor = &mut node.walk();
		let mut vector: Vec<Span> = vec![];
		let mut token_end = start;
		//let comment = Query::new(node.language(), "(comment)").unwrap();
		//let qc = QueryCursor::new();
		loop {
			// we select if it is a kind of "string" because the children of
			// the "string" are the symbols surrounding the string and doesn't
			// include the literal between them
			if cursor.node().kind() == "string"
				|| cursor.node().kind() == "comment"
				|| !cursor.goto_first_child()
			{
				let start_byte = cmp::max(cursor.node().start_byte(), start);
				if start_byte - token_end != 0 {
					vector
						.push(Span::raw(self.content.slice(
							clamp(token_end, start, end)..clamp(start_byte, start, end),
						)));
				}
				vector.push(write_token(
					self.content.slice(
						clamp(start_byte, start, end)..clamp(cursor.node().end_byte(), start, end),
					),
					cursor.node().kind(),
				));
				token_end = cursor.node().end_byte();
				while !cursor.goto_next_sibling() {
					if !cursor.goto_parent() {
						return vector;
					}
				}
			}
		}
	}

	pub fn get_tree(&mut self) -> Tree {
		let parser = self.parser.as_mut().unwrap();
		parser
			.parse(
				self.content.clone().to_string(),
				Some(&self.tree.as_ref().unwrap()),
			)
			.unwrap()
	}

	pub fn edit_region<'b>(&mut self, start_byte: usize, end_byte: usize, text: &'b str) -> Point {
		let start_row = self.content.byte_to_line(start_byte);
		let end_row = self.content.byte_to_line(end_byte);
		let (_, start_row_byte_idx, _, _) = self.content.chunk_at_line_break(start_row);
		let (_, end_row_byte_idx, _, _) = self.content.chunk_at_line_break(end_row);
		let lowest = min(start_byte, end_byte);
		let highest = max(start_byte, end_byte);

		match (self.parser.as_ref(), &mut self.tree.as_mut()) {
			(Some(_parser), Some(tree)) => {
				let edit = InputEdit {
					start_byte: lowest,
					old_end_byte: highest,
					new_end_byte: lowest + text.len(),

					start_position: Point {
						row: start_row,
						column: lowest - start_row_byte_idx,
					},
					old_end_position: Point {
						row: end_row,
						column: highest - start_row_byte_idx,
					},
					new_end_position: Point {
						row: self.content.byte_to_line(end_byte + text.len()),
						column: lowest - end_row_byte_idx + text.len(),
					},
				};
				tree.edit(&edit)
			}
			_ => (), // no parse language
		}

		// println!("({},{}), ({},{}) -> ({},{})",
		// 		 start_row,
		// 		 start_byte - start_row_byte_idx,
		// 		 end_row,
		// 		 end_byte - end_row_byte_idx,
		// 		 self.content.byte_to_line(end_byte + text.len()),
		// 		 end_byte - end_row_byte_idx + text.len());

		// edit buffer content
		self.content.remove(lowest..highest);
		self.content.insert(lowest, text.clone());
		match (self.parser.as_ref(), self.tree.as_ref()) {
			(Some(_parser), Some(_tree)) => self.tree = Some(Box::new(self.get_tree())),
			_ => (),
		}

		Point {
			row: self.content.byte_to_line(end_byte),
			column: end_byte,
		}
	}

	pub fn insert_at_point<'b>(&mut self, point: Point, text: &'b str) -> Point {
		let index = self.content.line_to_byte(point.row) + point.column;
		let mut point = self.edit_region(index, index, text);
		point.column += text.len();
		point
	}

	pub fn delete_backwards_at_point(&mut self, point: Point, n: usize) -> Point {
		let index = self.content.line_to_byte(point.row) + point.column;
		self.edit_region(index, index - n, "")
	}

	pub fn delete_forwards_at_point(&mut self, point: Point, n: usize) {
		let index = self.content.line_to_byte(point.row) + point.column;
		self.edit_region(index, index + n, "");
	}
}
