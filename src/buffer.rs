use ropey::Rope;
use tree_sitter::{Language, Parser, Tree};

pub struct Buffer {
	pub content: Rope,
	pub name: String,
	pub parser: Option<Box<Parser>>,
	pub tree: Option<Box<Tree>>,
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
