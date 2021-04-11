use tree_sitter::Language;

use crate::buffer::Buffer;

pub struct Editor {
	pub buffers: Vec<Buffer>,
}

impl Editor {
	pub fn add_buffer(&mut self, content: String, name: String, language: Option<Language>) {
		self.buffers.push(Buffer::new(content, name, language));
	}
}

impl Default for Editor {
	fn default() -> Self {
		Editor { buffers: vec![] }
	}
}
