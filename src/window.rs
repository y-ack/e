use tui::{
	layout::Rect,
	style::{Color, Style},
	widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{buffer::Buffer, editor::Editor};

struct Point {
	x: usize,
	y: usize,
}

/// A window/visible buffer
pub struct Window<'a> {
	buffer: &'a mut Buffer<'a>,
	cursor: Point,
	view_offset: Point,
	editor: Box<&'a Editor<'a>>,
}

impl<'a> Window<'a> {
	pub fn new(buffer: &'a mut Buffer<'a>, editor: Box<&'a Editor>) -> Window<'a> {
		Window {
			buffer: buffer,
			cursor: Point { x: 0, y: 0 },
			view_offset: Point { x: 0, y: 0 },
			editor: editor,
		}
	}

	pub fn get_widget(&self, viewport: Rect) -> Paragraph {
		let text = self
			.buffer
			.render_with_viewport(self.view_offset.y as u32, viewport.height);

		Paragraph::new(text)
			.block(
				Block::default()
					.title(self.buffer.name.clone())
					.borders(Borders::ALL),
			)
			.style(Style::default().fg(Color::White).bg(Color::Black))
			.scroll((0, self.view_offset.x as u16))
			.wrap(Wrap { trim: false })
	}

	pub fn insert_at_cursor(&mut self, text: &'a str) {
		self.buffer
			.insert_at_point(self.cursor.y, self.cursor.x, text);
	}
}
