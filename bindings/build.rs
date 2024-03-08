use std::env;
use std::process::Command;

fn main() {
	println!("cargo:rerun-if-changed=example.c");
	let _ = Command::new("cc")
		.arg(env::var("OUT_DIR").unwrap().to_owned() + "/libclue_bindings.a")
		.args(["example.c", "-o", "example"])
		.status();
}
