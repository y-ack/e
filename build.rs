use std::path::PathBuf;

fn main() {
  println!("cargo:rustc-link-lib=static=stdc++");
	// get directory of treesitter backends
	let dir: PathBuf = ["lang", "tree-sitter-javascript", "src"].iter().collect();

	// build the treesitter backends
	cc::Build::new()
		.include(&dir)
		.file(dir.join("parser.c"))
		.file(dir.join("scanner.c"))
		.warnings(false)
		.compile("tree-sitter-javascript");
	
	let lua_src_dir: PathBuf = ["lang", "tree-sitter-lua", "src"].iter().collect();
	cc::Build::new()
		.include(&lua_src_dir)
		.file(lua_src_dir.join("parser.c"))
		.file(lua_src_dir.join("scanner.cc"))
		.compile("tree-sitter-lua");
}
