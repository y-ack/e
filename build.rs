use std::path::PathBuf;

fn main() {
	// get directory of treesitter backends
	let dir: PathBuf = ["lang", "tree-sitter-javascript", "src"].iter().collect();
	// let lua_dir: PathBuf = ["lang", "tree-sitter-lua", "src"].iter().collect();
	// build the treesitter backends
	cc::Build::new()
		.include(&dir)
		.file(dir.join("parser.c"))
		.file(dir.join("scanner.c"))
		.warnings(false)
		.compile("tree-sitter-javascript");
	// cc::Build::new()
	// 	.include(&lua_dir)
	// 	.file(lua_dir.join("parser.c"))
	// 	.file(lua_dir.join("scanner.cc"))
	// 	// .file(lua_dir.join("binding.cc"))
	// 	.warnings(false)
	// 	// .cpp(true)
	// 	.compile("tree-sitter-lua");
}
