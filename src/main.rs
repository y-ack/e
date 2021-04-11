mod buffer;
mod interface;
mod window;

use buffer::Buffer;
use interface::{Interface, WindowTree};
use tree_sitter::Language;
use window::Window;

extern "C" {
	fn tree_sitter_lua() -> Language;
}

fn main() {
	// TODO: we probably need to store all of the available tree sitter
	// configurations somewhere at some point.
	let language = unsafe { tree_sitter_lua() };

	// create a scratch buffer, there must be at LEAST one buffer
	// that exists for the root window to attach to
	let buffer = Buffer::new(
		String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter."),
		String::from("*scratch*"),
		Some(language)
	);

	let mut interface = Interface::new(&buffer).unwrap();
	let tree2 = WindowTree {
		window: Box::new(Window::new(&buffer)),
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
