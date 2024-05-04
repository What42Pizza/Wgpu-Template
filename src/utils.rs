use crate::prelude::*;



#[cfg(debug_assertions)]
pub fn get_program_file_path(input: impl AsRef<Path>) -> PathBuf {
	let mut output = std::env::current_exe().expect("Could not retrieve the path for the current exe.");
	output.pop();
	output.pop();
	output.pop();
	output.join("data").join(input)
}

#[cfg(not(debug_assertions))]
pub fn get_program_file_path(input: impl AsRef<Path>) -> PathBuf {
	let mut output = std::env::current_exe().expect("Could not retrieve the path for the current exe.");
	output.pop();
	output.join(input)
}



pub trait IoResultFns<T> {
	fn add_path_to_error(self, path: impl AsRef<Path>) -> Result<T>;
}

impl<T> IoResultFns<T> for std::io::Result<T> {
	fn add_path_to_error(self, path: impl AsRef<Path>) -> Result<T> {
		match self {
			StdResult::Ok(v) => Ok(v),
			StdResult::Err(err) => Err(Error::msg(format!("Error with file {:?}: {err}", path.as_ref()))),
		}
	}
}
