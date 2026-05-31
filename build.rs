fn main() {
	// for some reason this is needed to fix intel_tex_2
	println!("cargo:rustc-link-lib=dylib=stdc++");
}
