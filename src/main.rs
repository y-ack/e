mod buffer;
mod interface;
mod window;

use buffer::Buffer;
use interface::{Interface, WindowTree};
use tree_sitter::Language;
use window::Window;

extern "C" {
	fn tree_sitter_javascript() -> Language;
}

fn main() {
	// TODO: we probably need to store all of the available tree sitter
	// configurations somewhere at some point.
	let language = unsafe { tree_sitter_javascript() };

	// create a scratch buffer, there must be at LEAST one buffer
	// that exists for the root window to attach to
	let buffer = Buffer::new(
		String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter."),
		String::from("*scratch*"),
		None
	);
	let buffer2 = Buffer::new(
		String::from("// this is an example JavaScript file\nfunction hello() {\n    console.log('hello, world!');\n}"),
		String::from("*scratch*"),
		Some(language)
	);

	let mut interface = Interface::new(&buffer).unwrap();
	let tree2 = WindowTree {
		window: Box::new(Window::new(&buffer2)),
		branch: None,
		orientation: tui::layout::Direction::Vertical,
	};
	let tree = WindowTree {
		window: Box::new(Window::new(&buffer)),
		branch: Some(Box::new(&tree2)),
		orientation: tui::layout::Direction::Horizontal,
	};
	interface.root_window.branch = Some(Box::new(&tree));

	interface.clear().ok();
	interface.draw().ok();

	loop {
		interface.update().ok().expect("oh well 2");
	}
}
