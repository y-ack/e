use std::{cell::RefCell, io::stdout, io::Stdout, rc::Rc};

use crossterm::{
	cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition},
	execute, ExecutableCommand,
};
use tree_sitter::Point;
use tui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout, Rect},
	style::{Color, Style},
	text::{Span, Spans},
	widgets::{Block, Borders, Paragraph, Wrap},
	Frame,
};

use crate::buffer::{self, Buffer};

/// A visible representation of a [`Buffer`]
pub struct Pane {
	pub buffer: Rc<RefCell<Buffer>>,
	pub branch: Option<Rc<RefCell<Pane>>>,
	pub orientation: Direction,
	pub cursor: Point,
	pub view_offset: Point,
	pub view_area: Rect,
}

impl Pane {
	/// Creates a new window from a given buffer
	pub fn new(buffer: Rc<RefCell<buffer::Buffer>>) -> Pane {
		Pane {
			buffer: buffer,
			cursor: Point { column: 5, row: 0 },
			view_offset: Point { column: 0, row: 0 },
			branch: None,
			view_area: Rect {
				x: 0,
				y: 0,
				width: 0,
				height: 0,
			},
			orientation: Direction::Vertical,
		}
	}

	// TODO: this should be handled by something else and NOT the window
	/// Given a [`Rect`], it will render itself and all subwindows within the
	/// given region.
	pub fn draw_widgets(&mut self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>) {
		let buffer = (*self.buffer).borrow();
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

		self.view_area = l[0];

		f.render_widget(
			Paragraph::new(display)
				.block(Block::default().title(name).borders(Borders::ALL))
				.style(Style::default().fg(Color::White).bg(Color::Black))
				.scroll((0, self.view_offset.column as u16))
				.wrap(Wrap { trim: false }),
			l[0],
		);

		match self.branch {
			Some(_) => {
				let branch = self.branch.as_deref().unwrap();
				(*branch).borrow_mut().draw_widgets(l[1], f)
			}
			None => {}
		};
	}

	pub fn draw_cursor(&self) -> crossterm::Result<()> {
		stdout()
			.execute(MoveTo(
				self.view_area.x + self.cursor.column as u16,
				self.view_area.y + self.cursor.row as u16,
			))?
			.execute(RestorePosition)?;

		Ok(())
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

	/// Splits the window horizontally and copies the current buffer state into
	/// it
	pub fn split_window_vertical(&mut self) {
		self.orientation = Direction::Vertical;
		self.branch = Some(Rc::new(RefCell::new(Pane::new(self.buffer.clone()))));
	}

	/// Splits the window horizontally and copies the current buffer state into
	/// it
	pub fn split_window_horizontal(&mut self) {
		self.orientation = Direction::Horizontal;
		self.branch = Some(Rc::new(RefCell::new(Pane::new(self.buffer.clone()))));
	}
}
