use std::sync::{Arc, Mutex};

use tree_sitter::Point;
use tui::{
	layout::Rect,
	style::{Color, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::buffer::Buffer;

/// A window/visible buffer
pub struct Window {
	pub buffer: Arc<Mutex<Buffer>>,
	pub cursor: Point,
	pub view_offset: Point,
}

impl Window {
	pub fn new(buffer: Arc<Mutex<Buffer>>) -> Window {
		Window {
			buffer: buffer,
			cursor: Point { column: 5, row: 0 },
			view_offset: Point { column: 0, row: 0 },
		}
	}

	pub fn insert_at_cursor<'b>(&mut self, text: &'b str) {
		self.cursor = self.buffer.lock().unwrap().insert_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			text,
		);
	}

	pub fn delete_backwards_at_cursor(&mut self, n: usize) {
		self.cursor = self.buffer.lock().unwrap().delete_backwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}

	pub fn delete_forwards_at_cursor(&mut self, n: usize) {
		self.buffer.lock().unwrap().delete_forwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}
}
