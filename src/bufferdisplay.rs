use std::{borrow::Cow, cell::RefCell, cmp::max, cmp::min, rc::Rc};
use tui::{
	style::{Color, Style},
	text::Span,
};

use tree_sitter::{Node, Point};

use crate::buffer::Buffer;

pub struct BufferDisplay {
	pub buffer: Rc<RefCell<Buffer>>,
	pub cursor: Point,
	pub view_offset: Point,
}

// TODO: need to actually make a Theme dict lol
/// Styles a string accordingly to the string and returns a Span
///
/// * text - The text that will be styled into a [`Span`]
/// * token - The token name, expected to be returned from [`Node`].kind()
fn write_token<'a, T>(text: T, token: &'static str) -> Span<'a>
where
	T: Into<Cow<'a, str>>,
{
	Span::styled(
		text,
		Style::default().fg(
			// TODO: we really need to make the theme into a dict of some sort
			match token {
				"function" => Color::Rgb(246, 199, 255),
				"identifier" => Color::Cyan,
				"string" => Color::Yellow,
				"comment" => Color::Green,
				_ => Color::White,
			},
		),
	)
}

impl BufferDisplay {
	/// Creates a new BufferDisplay from a given buffer
	pub fn new(buffer: Rc<RefCell<Buffer>>) -> BufferDisplay {
		BufferDisplay {
			buffer: buffer,
			cursor: Point { column: 0, row: 0 },
			view_offset: Point { column: 0, row: 0 },
		}
	}

	/// Applies syntax highlighting to a line in the buffer
	///
	/// * line - The line of the content to highlight
	/// * offset - The X offset of the view area to start rendering
	/// * width - The width of the view area that will be rendered
	pub fn highlight_line<'b>(&self, line: usize, offset: usize, width: usize) -> Vec<Span> {
		let buffer = self.buffer.borrow();
		let tree = buffer.tree.as_ref().unwrap();
		let node = tree
			.root_node()
			.descendant_for_point_range(
				Point {
					column: 0,
					row: line,
				},
				Point {
					column: buffer.content.line(line).len_chars(),
					row: line,
				},
			)
			.unwrap();
		let start = buffer.content.line_to_char(line) + offset;
		let end = min(width + start, buffer.content.line_to_char(line + 1));
		self.highlight(node, start, end)
	}

	/// Applies syntax highlighting to a region in the buffer
	///
	/// * node - The [`Node`] data that that spans the region of start and end.
	/// * start - The start character point of the region
	/// * end - The end character point of the region
	///
	/// # Example
	///
	/// ```rust
	/// // this assumes that the buffer has the following content
	/// // "function hello() { console.log('Hello, World!') }"
	/// // let's grab the region of "function"
	/// let start = 0;
	/// let end = 8;
	/// // get the node that spans the region of start and end
	/// let node = buffer.tree.unwrap().root_node().descendent_for_byte_range(
	///		start, end
	/// );
	/// let spans = Spans::from(buffer.highlight(node, start, end));
	/// ```
	pub fn highlight<'b>(&self, node: Node, start: usize, end: usize) -> Vec<Span> {
		let buffer = self.buffer.borrow();
		let cursor = &mut node.walk();
		let mut vector: Vec<Span> = vec![];
		let mut token_end = start;
		loop {
			// we select if it is a kind of "string" because the children of
			// the "string" are the symbols surrounding the string and doesn't
			// include the literal between them
			if cursor.node().kind() == "string"
				|| cursor.node().kind() == "comment"
				|| !cursor.goto_first_child()
			{
				let start_char = max(buffer.point_to_char(cursor.node().start_position()), start);
				if start_char - token_end != 0 {
					vector.push(Span::raw(
						buffer
							.content
							.slice(token_end.clamp(start, end)..start_char.clamp(start, end))
							.to_string(),
					));
				}
				vector.push(write_token(
					buffer
						.content
						.slice(
							start_char.clamp(start, end)
								..buffer
									.point_to_char(cursor.node().end_position())
									.clamp(start, end),
						)
						.to_string(),
					cursor.node().kind(),
				));
				token_end = buffer.point_to_char(cursor.node().end_position());
				while !cursor.goto_next_sibling() {
					if !cursor.goto_parent() {
						return vector;
					}
				}
			}
		}
	}

	/// Inserts text at the cursor
	pub fn insert_at_cursor<'b>(&mut self, text: &'b str) {
		self.cursor = self.buffer.borrow_mut().insert_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			text,
		);
	}

	/// Deletes backwards for n-bytes at the cursor
	pub fn delete_backwards_at_cursor(&mut self, n: usize) {
		self.cursor = self.buffer.borrow_mut().delete_backwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}

	/// Deletes forwards for n-bytes at the cursor
	pub fn delete_forwards_at_cursor(&mut self, n: usize) {
		self.buffer.borrow_mut().delete_forwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}
}
