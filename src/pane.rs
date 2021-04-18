use std::{cell::RefCell, io::Stdout, rc::Rc};

use tree_sitter::Point;
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Paragraph, Wrap},
	Frame,
};

use crate::{buffer::Buffer, bufferdisplay::BufferDisplay};

/// A visible representation of a [`Buffer`]
pub struct Pane {
	pub buffer_displays: Vec<Rc<RefCell<BufferDisplay>>>,
	pub current_buffer: Rc<RefCell<BufferDisplay>>,
	pub branch: Option<Rc<RefCell<Pane>>>,
	pub orientation: Direction,
}

impl Pane {
	/// Creates a new window from a given buffer
	pub fn new(buffer: Rc<RefCell<Buffer>>) -> Pane {
		let buffer_display = Rc::new(RefCell::new(BufferDisplay::new(buffer)));
		Pane {
			current_buffer: buffer_display.clone(),
			buffer_displays: vec![buffer_display],
			branch: None,
			orientation: Direction::Vertical,
		}
	}

	// TODO: this should be handled by something else and NOT the window
	/// Given a [`Rect`], it will render itself and all subwindows within the
	/// given region.
	pub fn draw_widgets(&self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) {
		let buffer_display = (*self.current_buffer).borrow();
		let buffer = buffer_display.buffer.borrow();
		let name = buffer.name.as_str();

		let l = Layout::default()
			.direction(self.orientation.clone())
			.margin(0)
			.constraints(match self.branch {
				Some(_) => [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
				None => [Constraint::Percentage(100)].as_ref(),
			})
			.split(area);

		let display: Vec<Spans> = buffer
			.content
			.lines_at(buffer_display.view_offset.row)
			.take(area.height as usize)
			.enumerate()
			.map(|(i, r)| {
				Spans::from(match buffer.tree.as_ref() {
					Some(_) => Spans::from(buffer_display.highlight_line(
						i,
						buffer_display.view_offset.column,
						l[0].width as usize,
					)),
					None => Spans::from(Span::raw(r)),
				})
			})
			.collect();

		f.render_widget(
			Paragraph::new(display)
				.block(Block::default().title(name).borders(Borders::ALL))
				.style(Style::default().fg(Color::White).bg(Color::Black))
				.scroll((0, self.current_buffer.borrow().view_offset.column as u16))
				.wrap(Wrap { trim: false }),
			l[0],
		);

		match self.branch {
			Some(_) => {
				let branch = self.branch.as_deref().unwrap();
				(*branch).borrow().draw_widgets(l[1], f)
			}
			None => {}
		};
	}

	/// Splits the window horizontally and copies the current buffer state into
	/// it
	pub fn split_window_vertical(&mut self) {
		self.orientation = Direction::Vertical;
		self.branch = Some(Rc::new(RefCell::new(Pane::new(
			self.current_buffer.borrow().buffer.clone(),
		))));
	}

	/// Splits the window horizontally and copies the current buffer state into
	/// it
	pub fn split_window_horizontal(&mut self) {
		self.orientation = Direction::Horizontal;
		self.branch = Some(Rc::new(RefCell::new(Pane::new(
			self.current_buffer.borrow().buffer.clone(),
		))));
	}
}
