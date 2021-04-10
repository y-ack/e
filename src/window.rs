use tui::{
	layout::Rect,
	style::{Color, Style},
	widgets::{Block, Borders, Paragraph},
};

use crate::buffer::Buffer;

/// A window/visible buffer
pub struct Window<'a> {
	buffer: &'a Buffer,
	// TODO: need to have some sort of Point object that defines both the
	// of the screen as well as the current location of the cursor in the
	// buffer
	// cursor: Point,
}

impl<'a> Window<'a> {
	pub fn new(buffer: &'a Buffer) -> Window<'a> {
		Window { buffer: buffer }
	}

	pub fn get_widget(&self, viewport: Rect) -> Paragraph {
		let text = self.buffer.render_with_viewport(0, viewport.height);

		Paragraph::new(text)
			.block(
				Block::default()
					.title(self.buffer.name.clone())
					.borders(Borders::ALL),
			)
			.style(Style::default().fg(Color::White).bg(Color::Black))
			.scroll((0, 3))
	}
}
