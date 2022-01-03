#![allow(non_snake_case)]

use std::io;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

//Version 7 Alpha 0.0

macro_rules! check {
	($tocheck: expr) => {
		match $tocheck {
			Ok(t) => t,
			Err(e) => return Err(e.to_string())
		}
	};
}

fn CompileFile(path: &Path, name: String) -> Result<(), String> {
	let mut code: String = String::new();
	check!(check!(File::open(path)).read_to_string(&mut code));
	let compiledname: String = String::from(path.display()
		.to_string()
		.strip_suffix(".clue")
		.unwrap()) + ".lua";
	let output: File = check!(File::create(compiledname));
	Ok(())
}

fn CompileFolder(path: &Path) -> Result<(), String> {
	for entry in check!(fs::read_dir(path)) {
		let entry = check!(entry);
		let name: String = entry.path().file_name().unwrap().to_string_lossy().into_owned();
		let filePathName: String = path.display().to_string() + "\\" + &name;
		let filepath: &Path = &Path::new(&filePathName);
		if filepath.is_dir() {
			CompileFolder(filepath)?;
		} else if filePathName.ends_with(".clue") {
			CompileFile(filepath, name)?;
		}
	}
	Ok(())
}

fn main() -> Result<(), String> {
	let args: Vec<String> = env::args().collect();
	let mut codepath: String = String::new();
	if args.len() == 1 {
		println!("Please insert path to code directory: ");
		io::stdin()
			.read_line(&mut codepath)
			.expect("Failed to read path!");
	} else {
		codepath = args[1].clone();
	}
	let path: &Path = Path::new(&codepath);
	if !path.is_dir() {
		return Err(String::from("The given path doesn't exist"));
	}
	CompileFolder(path)?;
	Ok(())
}