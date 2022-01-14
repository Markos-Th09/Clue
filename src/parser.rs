use crate::scanner::Token;
use crate::scanner::TokenType;
use crate::scanner::TokenType::*;
use self::ComplexToken::*;
use std::cmp;

#[derive(Debug)]
struct Expression {
	tokens: Vec<ComplexToken>,
	line: u32
}

#[derive(Debug)]
pub enum ComplexToken {
	VALUE {
		value: String,
		kind: TokenType,
		line: u32
	},

	CHAR {
		kind: TokenType,
		line: u32,
	},

	CALL {
		name: Expression,
		args: Vec<Expression>,
		line: u32
	},
}

struct ParserInfo {
	current: usize,
	size: usize,
	tokens: Vec<Token>,
	filename: String,
	ctokens: Vec<ComplexToken>
}

impl ParserInfo {
	fn new(tokens: Vec<Token>, filename: String) -> ParserInfo {
		ParserInfo {
			current: 0,
			size: tokens.len(),
			tokens: tokens,
			filename: filename,
			ctokens: Vec::new()
		}
	}

	fn getLine(&self) -> u32 {
		self.at(self.current - 1).line
	}

	fn error(&self, msg: String) -> String {
		println!("Error in file \"{}\" at line [{}]!", self.filename, self.getLine());
		msg
	}

	fn expected(&self, expected: &str, before: &str) -> String {
		self.error(format!("Expected token '{}' before '{}'.", expected, before))
	}

	fn unexpected(&self, str: &str) -> String {
		self.error(format!("Unexpected token '{}'.", str))
	}

	fn ended(&self) -> bool {
		self.current >= self.size
	}

	fn at(&self, pos: usize) -> Token {
		self.tokens[cmp::min(pos, self.size - 1)].clone()
	}

	fn advance(&mut self) -> Token {
		let prev: Token = self.at(self.current);
		self.current += 1;
		prev
	}

	fn peekFar(&self, pos: usize) -> Token {
		let pos: usize = self.current + pos;
		self.at(pos)
	}

	fn lookBack(&self, pos: usize) -> Token {
		let pos: usize = self.current - pos - 1;
		self.at(pos)
	}

	fn peek(&self) -> Token {
		self.peekFar(0)
	}

	fn compare(&mut self, expected: TokenType) -> bool {
		if self.ended() {return false;}
		if self.at(self.current).kind != expected {return false;}
		true
	}

	fn buildCall(&mut self, name: Expression) -> Result<ComplexToken, String> {
		let mut args: Vec<Expression> = Vec::new();
		let mut start: usize = self.current + 2;
		let mut pscope: u8 = 0;
		let mut ended: bool = false;
		loop {
			println!("{:?}", name);
			loop {
				let t: Token = self.advance();
				println!("{:?}", t.kind);
				match t.kind {
					ROUND_BRACKET_OPEN => {pscope += 1}
					ROUND_BRACKET_CLOSED => {
						pscope -= 1;
						println!("{}", pscope);
						if pscope == 0 {
							ended = true;
							self.current -= 1;
							break;
						}
					}
					COMMA => {
						if pscope > 1 {continue}
						self.current -= 1;
						break;
					}
					EOF => {return Err(self.expected(")", "<eof>"))}
					_ => {}
				}
			}
			println!("{} {}", start, self.current);
			if start == self.current {
				if args.len() > 0 {
					return Err(self.error(String::from("Invalid empty function argument found.")))
				}
				break
			}
			args.push(self.buildExpression(start, self.current)?);
			if ended {
				break;
			}
			self.current += 1;
			start = self.current;
		}
		println!("Finished func");
		Ok(CALL {
			name: name,
			args: args,
			line: self.getLine()
		})
	}

	fn buildExpression(&mut self, start: usize, end: usize) -> Result<Expression, String> {
		let mut expr = Expression {
			tokens: Vec::new(),
			line: self.at(start).line
		};
		self.current = start;
		while self.current < end {
			let t: Token = self.advance();
			println!("{:?}", t.kind);
			match t.kind {
				IDENTIFIER => {
					let line: u32 = self.getLine();
					if self.compare(ROUND_BRACKET_OPEN) {
						self.current -= 1;
						expr.tokens.push(self.buildCall(Expression {
							tokens: vec![VALUE {
								value: t.lexeme,
								kind: IDENTIFIER,
								line: line
							}],
							line: line
						})?);
						self.current += 1;
					} else if self.current == end || self.at(self.current).isOp() {
						expr.tokens.push(VALUE {
							value: t.lexeme,
							kind: IDENTIFIER,
							line: line
						})
					} else {
						return Err(self.unexpected(t.lexeme.as_str()))
					}
				}
				PLUS | MINUS | STAR | SLASH | PERCENTUAL | CARET | TWODOTS => {
					if self.current - 1 == start {
						return Err(self.error(format!("Operator '{}' not expected at the start of expression.", t.lexeme)))
					}
					if self.current == end {
						return Err(self.error(format!("Operator '{}' not expected at the end of expression.", t.lexeme)))
					}
					let pt: TokenType = self.lookBack(1).kind;
					let nt: TokenType = self.peek().kind ;
					if pt == TRUE || nt == TRUE || pt == FALSE || nt == FALSE {
						return Err(self.error(format!("Operator '{}' cannot operate with booleans.", t.lexeme)))
					}
					if pt != NUMBER && pt != IDENTIFIER && pt != STRING && pt != CURLY_BRACKET_CLOSED && pt != SQUARE_BRACKET_CLOSED && pt != NEW {
						return Err(self.error(format!("Operator '{}' has invalid left hand token.", t.lexeme)))
					}
					if nt != NUMBER && nt != IDENTIFIER && pt != STRING && nt != CURLY_BRACKET_OPEN && pt != CURLY_BRACKET_CLOSED {
						return Err(self.error(format!("Operator '{}' has invalid right hand token.", t.lexeme)))
					}
					expr.tokens.push(CHAR {
						kind: t.kind,
						line: self.getLine()
					})
				}
				NUMBER | STRING | TRUE | FALSE => {
					if self.current - 1 == start || self.current == end || self.at(self.current).isOp() {
						expr.tokens.push(VALUE {
							value: t.lexeme,
							kind: t.kind,
							line: self.getLine()
						})
					} else {
						return Err(self.unexpected(t.lexeme.as_str()))
					}
				}
				_ => {return Err(self.unexpected(t.lexeme.as_str()))}
			}
		}
		if expr.tokens.len() == 0 {
			return Err(self.unexpected(self.at(end).lexeme.as_str()))
		}
		Ok(expr)
	}
}

pub fn ParseTokens(tokens: Vec<Token>, filename: String) -> Result<Vec<ComplexToken>, String> {
	let mut i: ParserInfo = ParserInfo::new(tokens, filename);
	/*while !i.ended() {
		let t: Token = i.advance();
		match t.kind {
			IF => {

			},
			_ => {}
		}
	}*/
	let call = i.buildCall(Expression {
		tokens: vec![VALUE {
			value: i.at(i.current).lexeme,
			kind: IDENTIFIER,
			line: 1,
		}],
		line: 1,
	})?;
	i.ctokens.push(call);
	Ok(i.ctokens)
}