use ropey::Rope;
use tree_sitter::{Language, Parser, Tree};

extern "C" {
	fn tree_sitter_javascript() -> Language;
}

pub struct Buffer {
	pub content: Rope,
	pub name: String,
	pub parser: Parser,
	pub tree: Tree,
}

impl Buffer {
	pub fn new(content: String, name: String) -> Buffer {
		let language = unsafe { tree_sitter_javascript() };
		let mut parser = Parser::new();
		parser.set_language(language).unwrap();
		let tree = parser.parse(content.clone(), None).unwrap();

		Buffer {
			content: Rope::from_str(&content),
			name: name,
			parser: parser,
			tree: tree,
		}
	}

	pub fn get_tree(&mut self) -> Tree {
		self.parser
			.parse(self.content.clone().to_string(), Some(&self.tree))
			.unwrap()
	}
}
