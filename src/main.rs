mod buffer;
mod interface;
mod window;

use buffer::Buffer;
use interface::Interface;
use tree_sitter::Language;
use window::Window;
use rlua::{Lua};

extern "C" {
	fn tree_sitter_javascript() -> Language;
}

fn main() {
	let lua = Lua::new();
	lua.context(|lua_context| {
		// set a global var s 
		lua_context.load(r#"
			s = "function hello_world() {\n  console.log('hello,	world!');cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc\n}"
		"#).exec().unwrap();

		//handle error
	});

	let mut interface = Interface::default();
	let language = unsafe { tree_sitter_javascript() };
	
	let buffer = lua.context(|lua_context| {
		let globals = lua_context.globals();
		// use variable s from lua as the buffer content
		let s2: rlua::String = globals.get("s").unwrap();
		Buffer::new(
			String::from(s2.to_str().unwrap()),
			String::from("test.js"),
			Some(language),
		)
		//handle error
	});
				
	interface.windows.push(Window::new(&buffer));

	interface.clear().ok();
	interface.draw().ok();

	loop {
		interface.update().ok().expect("oh well 2");
	}
}
