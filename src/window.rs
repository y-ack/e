use tui::{
	layout::Rect,
	style::{Color, Style},
	widgets::{Block, Borders, Paragraph},
};

use crate::buffer::Buffer;

struct Point {
	x: u32,
	y: u32,
}

/// A window/visible buffer
pub struct Window<'a> {
	buffer: &'a Buffer,
	cursor: Point,
	view_offset: Point,
}

impl<'a> Window<'a> {
	pub fn new(buffer: &'a Buffer) -> Window<'a> {
		Window {
			buffer: buffer,
			cursor: Point { x: 0, y: 0 },
			view_offset: Point { x: 0, y: 0 },
		}
	}

	pub fn get_widget(&self, viewport: Rect) -> Paragraph {
		let text = self
			.buffer
			.render_with_viewport(self.view_offset.y, viewport.height);

		Paragraph::new(text)
			.block(
				Block::default()
					.title(self.buffer.name.clone())
					.borders(Borders::ALL),
			)
			.style(Style::default().fg(Color::White).bg(Color::Black))
			.scroll((0, self.view_offset.x as u16))
	}
}
