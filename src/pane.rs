use std::{
	io::Stdout,
	sync::{Arc, Mutex},
};

use tree_sitter::Point;
use tui::{
	backend::CrosstermBackend,
	layout::Rect,
	style::{Color, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Paragraph, Wrap},
	Frame,
};

use crate::buffer::Buffer;

/// A window/visible buffer
pub struct Pane<'a> {
	pub buffer: Arc<Mutex<Buffer<'a>>>,
	pub cursor: Point,
	pub view_offset: Point,
}

impl<'a> Pane<'a> {
	pub fn new(buffer: Arc<Mutex<Buffer<'a>>>) -> Pane<'a> {
		Pane {
			buffer: buffer,
			cursor: Point { column: 5, row: 0 },
			view_offset: Point { column: 0, row: 0 },
		}
	}

	pub fn draw_widget(&self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) {
		let buffer = self.buffer.lock().unwrap();
		let name = buffer.name.to_str().unwrap();

		let display: Vec<Spans> = buffer
			.content
			.lines_at(self.view_offset.row)
			.take(area.height as usize)
			.enumerate()
			.map(|(i, r)| {
				let start_byte = buffer
					.content
					.line_to_byte(i + self.view_offset.row)
					.clone();
				Spans::from(match buffer.tree.as_ref() {
					Some(t) => Spans::from(
						buffer
							.highlight(
								t.root_node()
									.descendant_for_byte_range(
										start_byte,
										start_byte + r.len_bytes(),
									)
									.unwrap(),
								start_byte,
								start_byte + r.len_bytes(),
							)
							.clone(),
					),
					None => Spans::from(Span::raw(r)),
				})
			})
			.collect();

		f.render_widget(
			Paragraph::new(display)
				.block(Block::default().title(name).borders(Borders::ALL))
				.style(Style::default().fg(Color::White).bg(Color::Black))
				.scroll((0, self.view_offset.column as u16))
				.wrap(Wrap { trim: false }),
			area,
		);
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
