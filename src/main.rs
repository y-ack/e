mod buffer;
mod interface;
mod window;

use buffer::Buffer;
use interface::Interface;
use tree_sitter::Language;
use window::Window;

extern "C" {
	fn tree_sitter_javascript() -> Language;
}

fn main() {
	let mut interface = Interface::default();
	let language = unsafe { tree_sitter_javascript() };

	let buffer = Buffer::new(
		String::from("function hello_world() {\n  console.log('hello, world!');\n}"),
		String::from("test.js"),
		Some(language),
	);

	interface.windows.push(Window::new(&buffer));

	interface.clear().ok();
	interface.draw().ok();

	loop {
		interface.update().ok().expect("oh well 2");
	}
}
