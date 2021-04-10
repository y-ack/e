use ropey::Rope;
use ropey::RopeSlice;
use std::cmp;
use tree_sitter::{Language, Node, Parser, Tree};
use tui::{
	style::{Color, Style},
	text::{Span, Spans},
};

pub struct Buffer {
	pub content: Rope,
	pub name: String,
	pub parser: Option<Box<Parser>>,
	pub tree: Option<Box<Tree>>,
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

impl Buffer {
	pub fn new(content: String, name: String, language: Option<Language>) -> Buffer {
		match language {
			Some(v) => {
				let mut parser = Parser::new();
				parser.set_language(v).unwrap();
				let tree = parser.parse(content.clone(), None).unwrap();
				Buffer {
					content: Rope::from_str(&content),
					name: name,
					parser: Some(Box::new(parser)),
					tree: Some(Box::new(tree)),
				}
			}
			None => Buffer {
				content: Rope::from_str(&content),
				name: name,
				parser: None,
				tree: None,
			},
		}
	}

	pub fn highlight<'b>(&self, node: Node) -> Vec<Span> {
		let cursor = &mut node.walk();
		let mut vector: Vec<Span> = vec![];
		let mut token_end = 0;
		loop {
			if cursor.node().kind() == "string" || !cursor.goto_first_child() {
				let start_byte = cursor.node().start_byte();
				if start_byte - token_end != 0 {
					vector.push(Span::raw(
						self.content.slice(token_end..start_byte).as_str().unwrap(),
					));
				}
				vector.push(write_token(
					self.content
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
	pub fn render_with_viewport(&self, y: u32, h: u16) -> Vec<Spans> {
		let mut lines: Vec<RopeSlice> = vec![];

		for i in y..cmp::min(y + (h as u32), self.content.len_lines() as u32) {
			lines.push(self.content.line(i as usize));
		}

		lines
			.into_iter()
			.map(|x| {
				Spans::from(match self.parser {
					// Some(_) => self.highlight(self.tree.),
					Some(_) => Span::raw(x.as_str().unwrap()),
					None => Span::raw(x.as_str().unwrap()),
				})
			})
			.collect::<Vec<Spans>>()
	}

	pub fn get_tree(&mut self) -> Tree {
		let parser = self.parser.as_mut().unwrap();
		parser
			.parse(
				self.content.clone().to_string(),
				Some(&self.tree.as_ref().unwrap()),
			)
			.unwrap()
	}
}
