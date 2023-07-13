#![allow(clippy::blocks_in_if_conditions)]

use clap::{crate_version, Parser};
use clue_core::{
	check,
	compiler::*,
	env::{BitwiseMode, ContinueMode, LuaVersion, Options},
	format_clue,
	parser::*,
	preprocessor::*,
	scanner::*, error::{StringReader, CodeReader, FileReader},
};
use tempfile::Builder;
use std::{env, fs::{self, File}, path::PathBuf, time::Instant, process, io::Write};
use colored::*;

#[derive(Parser)]
#[clap(
	version,
	about = "C/Rust like programming language that compiles into Lua code\nMade by Maiori\nhttps://github.com/ClueLang/Clue",
	long_about = None
)]
struct Cli {
	/// The path to the directory where the *.clue files are located.
	/// Every directory inside the given directory will be checked too.
	/// If the path points to a single *.clue file, only that file will be compiled.
	#[clap(required_unless_present = "license")]
	path: Option<PathBuf>,

	/// The name the output file will have
	/// [default for compiling a directory: main]
	/// [default for compiling a single file: that file's name]
	#[clap(value_name = "OUTPUT FILE NAME")]
	outputname: Option<PathBuf>,

	/// Print license information
	#[clap(short = 'L', long, display_order = 1000)]
	license: bool,

	/// Print list of detected tokens in compiled files
	#[clap(long)]
	tokens: bool,

	/// Print syntax structure of the tokens of the compiled files
	#[clap(long)]
	r#struct: bool,

	/// Print output Lua code in the console
	#[clap(short, long)]
	output: bool,

	/// Print preprocessed file
	#[clap(short = 'E', long)]
	expand: bool,

	/// Use LuaJIT's bit library for bitwise operations
	#[clap(
		short,
		long,
		hide(true),
		default_missing_value = "bit",
		value_name = "VAR NAME"
	)]
	jitbit: Option<String>,

	/// Change the way bitwise operators are compiled
	#[clap(
		short,
		long,
		value_enum,
		ignore_case(true),
		default_value = "Clue",
		value_name = "MODE"
	)]
	bitwise: BitwiseMode,

	/// Change the way continue identifiers are compiled
	#[clap(
		short,
		long,
		value_enum,
		ignore_case(true),
		default_value = "simple",
		value_name = "MODE"
	)]
	r#continue: ContinueMode,

	/// Don't save compiled code
	#[clap(short = 'D', long)]
	dontsave: bool,

	/// Treat PATH not as a path but as Clue code
	#[clap(short, long)]
	pathiscode: bool,

	/// Use rawset to create globals
	#[clap(short, long)]
	rawsetglobals: bool,

	/// Add debug information in output (might slow down runtime)
	#[clap(short, long)]
	debug: bool,

	/// Use a custom Lua file as base for compiling the directory
	#[clap(short = 'B', long, value_name = "FILE NAME")]
	base: Option<String>,

	/// Uses preset configuration based on the targeted Lua version
	#[clap(
		short,
		long,
		value_enum,
		ignore_case(true),
		conflicts_with("bitwise"),
		conflicts_with("jitbit"),
		conflicts_with("continue"),
		value_name = "LUA VERSION"
	)]
	target: Option<LuaVersion>,

	/// Change OS checked by @ifos
	#[clap(long, default_value = std::env::consts::OS, value_name = "TARGET OS")]
	targetos: String,
	/*/// This is not yet supported (Coming out in 4.0)
	#[clap(short, long, value_name = "MODE")]
	types: Option<String>,*/

	/*	/// Enable type checking (might slow down compilation)
		#[clap(
			short,
			long,
			value_enum,
			default_value = "none",
			value_name = "MODE"
		)]
		types: TypesMode,

		/// Use the given Lua version's standard library (--types required)
		#[clap(
			long,
			value_enum,
			default_value = "luajit",
			value_name = "LUA VERSION",
			requires = "types"
		)]
		std: LuaSTD,
	*/
	#[cfg(feature = "mlua")]
	/// Execute the output Lua code once it's compiled
	#[clap(short, long)]
	execute: bool,

	#[cfg(feature = "lsp")]
	/// Print the symbol table of the compiled files
	#[clap(long, hide(true))]
	symbols: bool,
}

fn main() -> Result<(), String>{
    let cli = Cli::parse();
    let mut options = Options {
		env_outputname: cli.outputname.clone(),
		env_tokens: cli.tokens,
		env_struct: cli.r#struct,
		env_expand: cli.expand,
		env_jitbit: {
			if cli.jitbit.is_some() {
				println!("Warning: \"--jitbit was deprecated and replaced by --bitwise\"");
				cli.jitbit
			} else if cli.bitwise == BitwiseMode::Library {
				Some(String::from("bit"))
			} else {
				None
			}
		},
		env_bitwise: cli.bitwise,
		env_continue: cli.r#continue,
		env_rawsetglobals: cli.rawsetglobals,
		env_debug: cli.debug,
		env_output: if cli.pathiscode {
			cli.outputname.is_none()
		} else {
			cli.output
		},
		env_target: cli.target,
		env_targetos: cli.targetos,
		#[cfg(feature = "lsp")]
		env_symbols: cli.symbols,
        #[cfg(not(feature = "lsp"))]
        env_symbols: false,
	};
	options.preset();

    if cli.pathiscode{
        let code = cli.path.unwrap();
        let code = code.to_str().unwrap().to_owned();
        let reader: &dyn CodeReader = &StringReader::new(code);

        let (rawcode, variables) = read_code(reader, &options)?;
        let code  = preprocess_codes(0, rawcode, &variables, reader)?;
        let tokens = scan_code(code, reader)?;
        let (ctokens, statics) = parse_tokens(tokens, reader, &options)?;
        let compiler = Compiler::new(&options,reader);
        let code = compiler.compile_tokens(0, ctokens)?;

        println!("{}{}", statics, code);
    } else if let Some(path) = cli.path {
		if path.is_file(){
			let reader: &dyn CodeReader = &FileReader::new(path.to_string_lossy().to_string());
			let (rawcode, variables) = read_code(reader, &options)?;
			let code  = preprocess_codes(0, rawcode, &variables, reader)?;
			let tokens = scan_code(code, reader)?;
			let (ctokens, statics) = parse_tokens(tokens, reader, &options)?;
			let compiler = Compiler::new(&options,reader);
			let code = compiler.compile_tokens(0, ctokens)?;
			println!("{}{}", statics, code);
		}
	}

    Ok(())
}
