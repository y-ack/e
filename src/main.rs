mod buffer;
mod editor;
mod pane;

use editor::Editor;

fn main() {
	// TODO: we probably need to store all of the available tree sitter
	// configurations somewhere at some point.
	let mut editor = Editor::default();

	editor.clear().ok();
	editor.draw();

	while editor.running {
		editor.update().ok().expect("oh well 2");
	}
}
