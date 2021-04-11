mod buffer;
mod editor;
mod interface;
mod window;

use buffer::Buffer;
use editor::Editor;
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
	let mut editor = Editor::default();

	// create a scratch buffer, there must be at LEAST one buffer
	// that exists for the root window to attach to
	editor.add_buffer(
		String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter."),
		String::from("*scratch*"),
		None
	);

	let mut interface = Interface::new(&mut editor.buffers[0]).unwrap();

	interface.clear().ok();
	interface.draw().ok();

	while interface.running {
		interface.update().ok().expect("oh well 2");
	}
	interface.destroy();
}
