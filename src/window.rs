use ropey::RopeSlice;
use std::cmp;
use tui::{
	layout::Rect,
	style::{Color, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Paragraph},
};

use crate::buffer::Buffer;

// https://docs.rs/ropey/1.2.0/ropey/struct.Rope.html?search=#method.byte_to_line
//

/// A window/visible buffer
pub struct Window<'a> {
	buffer: &'a Buffer,
	// TODO: need to have some sort of Point object that defines both the
	// of the screen as well as the current location of the cursor in the
	// buffer
	// cursor: Point,
}

// TODO: need to actually make a Theme dict lol
fn write_token<'a>(text: &'a str, token: &'static str) -> Span<'a> {
	Span::styled(
		text,
		Style::default().fg(match token {
			"function" => Color::Rgb(246, 199, 255),
			"identifier" => Color::Cyan,
			"string" => Color::Yellow,
			_ => Color::White,
		}),
	)
}

impl<'a> Window<'a> {
	pub fn new(buffer: &'a Buffer) -> Window<'a> {
		Window { buffer: buffer }
	}

	pub fn highlight<'b>(&self) -> Vec<Span> {
		let cursor = &mut self.buffer.tree.as_ref().unwrap().walk();
		let mut vector: Vec<Span> = vec![];
		let mut token_end = 0;
		loop {
			if cursor.node().kind() == "string" || !cursor.goto_first_child() {
				let start_byte = cursor.node().start_byte();
				if start_byte - token_end != 0 {
					vector.push(Span::raw(
						self.buffer
							.content
							.slice(token_end..start_byte)
							.as_str()
							.unwrap(),
					));
				}
				vector.push(write_token(
					self.buffer
						.content
						.slice(start_byte..cursor.node().end_byte())
						.as_str()
						.unwrap(),
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
	fn render_with_viewport(&self, x: u32, y: u32, w: u16, h: u16) -> Vec<Spans> {
		let mut lines: Vec<RopeSlice> = vec![];

		for i in y..cmp::min(y + (h as u32), self.buffer.content.len_lines() as u32) {
			lines.push(self.buffer.content.line(i as usize));
			eprintln!("{}", self.buffer.content.line(i as usize));
			eprintln!("{}", self.buffer.content.line(i as usize).len_bytes());
		}

		// these are offsets expressed in usize
		let offx = x as usize;
		let offw = offx + (w as usize);
		eprintln!("{} {}", offx, offw);
		lines
			.into_iter()
			.map(|x| {
				Spans::from(Span::raw({
					let end = cmp::min(offw, x.len_bytes());
					if end >= offx {
						x.slice(offx..cmp::min(offw, x.len_bytes()))
							.as_str()
							.unwrap()
					} else {
						""
					}
				}))
			})
			.collect::<Vec<Spans>>()
	}

	pub fn get_widget(&self, viewport: Rect) -> Paragraph {
		let text = match self.buffer.tree {
			Some(_) => vec![Spans::from(self.highlight())],
			None => self.render_with_viewport(0, 0, viewport.width, viewport.height),
		};

		Paragraph::new(text)
			.block(
				Block::default()
					.title(self.buffer.name.clone())
					.borders(Borders::ALL),
			)
			.style(Style::default().fg(Color::White).bg(Color::Black))
	}
}
