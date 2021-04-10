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

	let mut buffer = Buffer::new(
		String::from("function hello_world() {\n  console.log('hello, world!');\n}"),
		String::from("test.js"),
		Some(language),
	);
	let buffer2 = Buffer::new(
		String::from(buffer.get_tree().root_node().to_sexp()),
		String::from("test.txt"),
		None,
	);

	interface.windows.push(Window::new(&buffer));
	interface.windows.push(Window::new(&buffer2));

	interface.clear().ok();
	interface.draw().ok();

	loop {
		interface.update().ok().expect("oh well 2");
	}
}
