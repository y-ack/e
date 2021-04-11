use std::cmp::{self, max};
use std::{borrow::Cow, cmp::min};

use ropey::Rope;
use ropey::RopeSlice;
use tree_sitter::{InputEdit, Language, Node, Parser, Point, Tree};
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

	pub fn highlight<'b>(&self, node: Node, start: usize, end: usize) -> Vec<Span> {
		let cursor = &mut node.walk();
		let mut vector: Vec<Span> = vec![];
		let mut token_end = start;
		loop {
			// we select if it is a kind of "string" because the children of
			// the "string" are the symbols surrounding the string and doesn't
			// include the literal between them
			if cursor.node().kind() == "string" || !cursor.goto_first_child() {
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

	// OPTIMIZE: we can probably just find the first line and then iterate through
	// each line by searching for the next newline from there instead of asking
	// rope to get each line individually. haven't tested how rope handles
	// carriage returns yet,
	pub fn render_with_viewport(&self, y: u32, h: u16) -> Vec<Spans> {
		struct Line<'a> {
			rope: RopeSlice<'a>,
			start_byte: usize,
		}
		let mut lines: Vec<Line> = vec![];

		for i in y..cmp::min(y + (h as u32), self.content.len_lines() as u32) {
			lines.push(Line {
				rope: self.content.line(i as usize),
				start_byte: self.content.line_to_byte(i as usize),
			});
		}

		lines
			.into_iter()
			.map(|x| {
				Spans::from(match self.tree.as_ref() {
					Some(t) => Spans::from(
						self.highlight(
							t.root_node()
								.descendant_for_byte_range(
									x.start_byte,
									x.start_byte + x.rope.len_bytes(),
								)
								.unwrap(),
							x.start_byte,
							x.start_byte + x.rope.len_bytes(),
						),
					),
					// Some(t) => Span::raw(x.rope),
					None => Spans::from(Span::raw(x.rope)),
				})
			})
			.collect::<Vec<Spans>>()
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
		let lowest = min(start_byte, end_byte);
		let highest = max(start_byte, end_byte);
		let edit = InputEdit {
			start_byte: lowest,
			old_end_byte: highest,
			new_end_byte: lowest + text.len(),
			start_position: Point {
				row: self.content.byte_to_line(lowest),
				column: self.content.start_byte,
			},
			old_end_position: Point {
				row: self.content.byte_to_line(end_byte),
				column: end_byte - start_byte,
			},
			new_end_position: Point {
				row: self.content.byte_to_line(end_byte),
				column: end_byte,
			},
		};
		self.content.remove(lowest..highest);
		self.content.insert(lowest, text);
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
