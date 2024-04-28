use crate::prelude::*;
use std::{fs::File, sync::Mutex};



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



lazy_static::lazy_static! {
	pub static ref LOG_WRITER: Mutex<File> = {
		let file = File::create("log.txt").expect("Failed to create log file.");
		Mutex::new(file)
	};
}

#[macro_export]
macro_rules! log {
	(raw: $string:expr) => {{
		use std::io::Write;
		
		let mut log_writer = utils::LOG_WRITER.lock().expect("Could not lock LOG_WRITER.");
		println!("{}", $string);
		log_writer.write_all($string.as_bytes()).expect("Could not write to log.");
		log_writer.write_all(&[b'\n']).expect("Could not write to log.");
		
	}};
	(>flush) => {{
		use std::io::Write;
		let mut log_writer = utils::LOG_WRITER.lock().expect("Could not lock LOG_WRITER.");
		log_writer.write_all(&[]).expect("Could not flush LOG_WRITER.");
	}};
	($($arg:tt)*) => {
		log!(raw: format!($($arg)*))
	};
}

#[macro_export]
macro_rules! warn {
	($($arg:tt)*) => {
		log!("WARNING: {}", format!($($arg)*))
	};
}
