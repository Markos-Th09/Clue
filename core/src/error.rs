use std::{ops::Range, fs};
use colored::{ColoredString, Colorize};


pub trait CodeReader {
	fn get_code(&self) -> Result<String, String>;
	fn get_filename(&self) -> String;
}

pub struct FileReader{
	filename: String,
}

impl FileReader{
	pub fn new(filename: String) -> Self{
		FileReader{
			filename,
		}
	}
}

impl CodeReader for FileReader{
	fn get_code(&self) -> Result<String, String>{
		fs::read_to_string(&self.filename).map_err(|e| e.to_string())
	}
	fn get_filename(&self) -> String {
		self.filename.clone()
	}
}

pub struct StringReader {
	code: String,
}

impl StringReader {
	pub fn new(code: String) -> Self{
		StringReader{
			code,
		}
	}
}

impl CodeReader for StringReader {
	fn get_code(&self) -> Result<String, String>{
		Ok(self.code.to_owned())
	}

	fn get_filename(&self) -> String {
		"<code>".to_owned()
	}
}



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClueErrorKind {
	Error,
	Warning,
}

impl ClueErrorKind {
	fn to_colored_string(self) -> ColoredString {
		match self {
			ClueErrorKind::Error => "Error".red().bold(),
			ClueErrorKind::Warning => "Warning".yellow().bold(),
		}
	}
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClueError {
	kind: ClueErrorKind,
	message: String,
	line: usize,
	column: usize,
	range: Range<usize>,
	help: Option<String>,
}

impl ClueError {
	pub fn new(
		kind: ClueErrorKind,
		message: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) -> Self {
		ClueError {
			kind,
			message: message.into(),
			line,
			column,
			range,
			help,
		}
	}

	pub fn error(
		message: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) -> Self {
		ClueError::new(ClueErrorKind::Error, message, line, column, range, help)
	}

	pub fn warning(
		message: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) -> Self {
		ClueError::new(ClueErrorKind::Warning, message, line, column, range, help)
	}

	pub fn expected(
		expected: impl Into<String>,
		got: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) -> Self {
		ClueError::error(format!("Expected '{}', got '{}'", expected.into(), got.into()), line, column, range, help)
	}

	pub fn expected_before(
		expected: impl Into<String>,
		before: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) -> Self {
		ClueError::error(format!("Expected '{}' before '{}'", expected.into(), before.into()), line, column, range, help)
	}
}
pub trait ErrorMessaging {
	fn send(
		&mut self,
		ClueError { kind, message, line, column, range, help }: ClueError,
	) {
		let is_first = self.is_first(kind == ClueErrorKind::Error);
		let filename = self.get_filename();
		let kind = kind.to_colored_string();

		let header = format!(
			"{}{} in {}:{}:{}!",
			if is_first {
				""
			} else {
				"\n----------------------------------\n\n"
			},
			kind,
			filename,
			line,
			column
		);
		let full_message = format!(
			"{}: {}{}",
			kind,
			message.replace('\n', "<new line>").replace('\t', "<tab>"),
			if let Some(help) = help {
				format!("\n{}: {}", "Help".cyan().bold(), help)
			} else {
				String::from("")
			}
		);

		if let Ok(code) = self.reader().get_code() {
			let before_err = get_errored_edges(&code[..range.start], str::rsplit);
			let after_err = get_errored_edges(&code[range.end..], str::split);
			let errored = &code[range];
			eprintln!(
				"{}\n\n{}{}{}\n\n{}",
				header,
				before_err.trim_start(),
				errored.red().underline(),
				after_err.trim_end(),
				full_message
			)
		} else {
			eprintln!("{}\n{}", header, full_message)
		}
	}

	fn error(
		&mut self,
		message: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) {
		self.send(ClueError::error(message, line, column, range, help))
	}

	fn warning(
		&mut self,
		message: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) {
		self.send(ClueError::warning(message, line, column, range, help))
	}

	fn expected(
		&mut self,
		expected: impl Into<String>,
		got: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) {
		self.send(ClueError::expected(expected, got, line, column, range, help))
	}

	fn expected_before(
		&mut self,
		expected: impl Into<String>,
		before: impl Into<String>,
		line: usize,
		column: usize,
		range: Range<usize>,
		help: Option<String>,
	) {
		self.send(ClueError::expected_before(expected, before, line, column, range, help))
	}

	fn get_filename(&self) -> &str;

	fn is_first(&mut self, error: bool) -> bool;

	fn reader(&self) -> &dyn CodeReader;
}

#[macro_export]
macro_rules! impl_errormessaging {
	($struct:ty) => {
		impl ErrorMessaging for $struct {
			#[inline]
			fn get_filename(&self) -> &str {
				&self.filename
			}

			#[inline]
			fn is_first(&mut self, error: bool) -> bool {
				if error {
					self.errors += 1;
				}
				self.errors == 1
			}

			#[inline]
			fn reader(&self) -> &dyn $crate::error::CodeReader {
				self.reader
			}
		}
	};
}

fn get_errored_edges<'a, T: Iterator<Item = &'a str>>(
    code: &'a str,
    splitter: impl FnOnce(&'a str, char) -> T,
) -> &str {
    splitter(code, '\n')
        .next()
        .unwrap_or_default()
}
