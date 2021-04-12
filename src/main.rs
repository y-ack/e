mod buffer;
mod editor;
mod interface;
mod window;

use editor::Editor;
use interface::{Interface, WindowTree};
use tree_sitter::Language;
use window::Window;

extern "C" {
	fn tree_sitter_javascript() -> Language;
	fn tree_sitter_lua() -> Language;
}

fn main() {
	// TODO: we probably need to store all of the available tree sitter
	// configurations somewhere at some point.
	let language_js = unsafe { tree_sitter_javascript() };
	let language_lua = unsafe { tree_sitter_lua() };
	let mut editor = Editor::default();

	// create a scratch buffer, there must be at LEAST one buffer
	// that exists for the root window to attach to
	editor.add_buffer(
		String::from("-- This buffer is for text that is not saved, and for Lua evaluation\n-- Use this to interact with the built-in Lua interpreter.\nfunction hello() {\n  console.log('hello, world!')\n}"),
		String::from("*scratch*"),
		Some(language_lua)
	);
	editor.add_buffer(
		String::from(""),
		String::from("*debug*"),
		None
	);

	let (buffer1, buffer2) = editor.buffers.split_at_mut(1);
	let mut interface = Interface::new(&mut buffer1[0]).unwrap();
	let tree = WindowTree {
		window: Box::new(Window::new(&mut buffer2[0])),
		branch: None,
		orientation: tui::layout::Direction::Vertical,
	};
	interface.root_window.branch = Some(Box::new(&tree));
	
	interface.clear().ok();
	interface.draw().ok();

	while interface.running {
		interface.update().ok().expect("oh well 2");
	}
	interface.destroy();
}
