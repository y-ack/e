use std::{ops::DerefMut, rc::Rc};

use tree_sitter::Point;
use tui::{
	layout::Rect,
	style::{Color, Style},
	widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::buffer::Buffer;

/// A window/visible buffer
pub struct Window<'a> {
	buffer: Rc<&'a mut Buffer>,
	cursor: Point,
	view_offset: Point,
}

impl<'a> Window<'a> {
	pub fn new(buffer: Rc<&'a mut Buffer>) -> Window<'a> {
		Window {
			buffer: buffer,
			cursor: Point { column: 5, row: 0 },
			view_offset: Point { column: 0, row: 0 },
		}
	}

	pub fn get_widget(&self, viewport: Rect) -> Paragraph {
		let text = self
			.buffer
			.render_with_viewport(self.view_offset.row as u32, viewport.height);

		Paragraph::new(text)
			.block(
				Block::default()
					.title(self.buffer.name.clone())
					.borders(Borders::ALL),
			)
			.style(Style::default().fg(Color::White).bg(Color::Black))
			.scroll((0, self.view_offset.column as u16))
			.wrap(Wrap { trim: false })
	}

	pub fn insert_at_cursor(&mut self, text: &'a str) {
		self.cursor = self.buffer.deref_mut().insert_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			text,
		);
	}

	pub fn delete_backwards_at_cursor(&mut self, n: usize) {
		self.cursor = self.buffer.delete_backwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}

	pub fn delete_forwards_at_cursor(&mut self, n: usize) {
		self.buffer.delete_forwards_at_point(
			Point {
				row: self.cursor.row,
				column: self.cursor.column,
			},
			n,
		);
	}
}
