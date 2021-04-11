mod buffer;
mod editor;
mod interface;
mod window;

use editor::Editor;
use interface::Interface;
use tree_sitter::Language;

extern "C" {
	fn tree_sitter_javascript() -> Language;
	fn tree_sitter_lua() -> Language;
}

fn main() {
	// TODO: we probably need to store all of the available tree sitter
	// configurations somewhere at some point.
	let language_js = unsafe { tree_sitter_javascript() };
	// let language_lua = unsafe { tree_sitter_lua() };
	let mut editor = Editor::default();

	// create a scratch buffer, there must be at LEAST one buffer
	// that exists for the root window to attach to
	editor.add_buffer(
		String::from("// This buffer is for text that is not saved, and for Lua evaluation\n// Use this to interact with the built-in Lua interpreter.\nfunction hello() {\n  console.log('hello, world!')\n}"),
		String::from("*scratch*"),
		Some(language_js)
	);

	let mut interface = Interface::new(&mut editor.buffers[0]).unwrap();

	interface.clear().ok();
	interface.draw().ok();

	while interface.running {
		interface.update().ok().expect("oh well 2");
	}
	interface.destroy();
}
