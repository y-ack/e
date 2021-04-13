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
	buffer: Arc<Mutex<Buffer>>,
	cursor: Point,
	view_offset: Point,
}

impl Window {
	pub fn new(buffer: Arc<Mutex<Buffer>>) -> Window {
		Window {
			buffer: buffer,
			cursor: Point { column: 5, row: 0 },
			view_offset: Point { column: 0, row: 0 },
		}
	}

	pub fn get_widget<'a>(&self, viewport: Rect) -> Paragraph {
		let name = self.buffer.lock().unwrap().name.clone();

		let buffer = *self.buffer.lock().unwrap();

		let display = buffer
			.content
			.lines_at(self.view_offset.row)
			.take(viewport.height as usize)
			.enumerate()
			.map(move |(i, x)| {
				let start_byte = buffer.content.line_to_byte(i + self.view_offset.row);
				Spans::from(match buffer.tree.as_ref() {
					Some(t) => Spans::from(
						buffer.highlight(
							t.root_node()
								.descendant_for_byte_range(start_byte, start_byte + x.len_bytes())
								.unwrap(),
							start_byte,
							start_byte + x.len_bytes(),
						),
					),
					None => Spans::from(Span::raw(x)),
				})
			})
			.collect::<Vec<Spans>>();

		Paragraph::new(display)
			.block(Block::default().title(name).borders(Borders::ALL))
			.style(Style::default().fg(Color::White).bg(Color::Black))
			.scroll((0, self.view_offset.column as u16))
			.wrap(Wrap { trim: false })
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
