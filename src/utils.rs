use crate::prelude::*;



pub fn get_program_dir() -> PathBuf {
	let mut output = std::env::current_exe().expect("Could not retrieve the path for the current exe.");
	output.pop();
	output
}

#[cfg(debug_assertions)]
pub fn get_program_file_path(input: impl AsRef<Path>) -> PathBuf {
	let mut output = get_program_dir();
	output.pop();
	output.pop();
	output.join("data").join(input)
}

#[cfg(not(debug_assertions))]
pub fn get_program_file_path(input: impl AsRef<Path>) -> PathBuf {
	get_program_dir().join(input)
}
