//
//
//
extern crate libc;	// for isspace
extern crate std;

use std::default::Default;
use std::io::IoResult;

#[deriving(PartialEq)]
#[deriving(Clone)]
enum Token {
	TokInval,
	TokEof,
	TokNumber(u64),
	TokLine(String),
	TokGroup(String),
	TokIdent(String),
	
	TokMetaOp(String),
	TokPreproc(String),
	
	TokNewline,
	TokComma,
	TokColon,
	TokAssign,
	
	TokPlus,
	TokMinus,
	TokStar,
	TokSlash,
	TokBackslash,
	
	TokSqOpen,
	TokSqClose,
	TokParenOpen,
	TokParenClose,
	
	TokComment,
}

struct Lexer<'stream>
{
	instream: &'stream mut Reader,
	filename: String,
	line: u32,
	lastchar: Option<char>,
	saved_tok: Option<Token>,
}

struct Parser<'stream>
{
	lexer: Lexer<'stream>,
}


macro_rules! is_enum( ($val:expr $exp:pat) => (match $val { $exp => true, _ => false }) )
macro_rules! exp_enum( ($val:expr $exp:ident) => (match $val { $exp(x) => x, _ => fail!("Expected enum {}", $exp) }) )
macro_rules! parse_try( ($e:expr, $rv:expr) => (match $e {Ok(v) => v, Err(_) => return $rv}) )
macro_rules! syntax_assert( ($tok:expr, $filter:pat => $val:expr, $msg:expr) => ({
	let tok = $tok;
	match tok {
		$filter => $val,
		_ => fail!("Syntax Assert failure: {}, got {}", $msg, tok)
	}
}) )

impl ::std::fmt::Show for Token
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
		TokInval     => write!(f, "TokInval"),
		TokEof       => write!(f, "TokEof"),
		TokNumber(ref v) => write!(f, "TokNumber({})", v),
		TokLine(ref v)   => write!(f, "TokLine({})", v),
		TokGroup(ref v)  => write!(f, "TokGroup({})", v),
		TokIdent(ref v)  => write!(f, "TokIdent({})", v),
		TokMetaOp(ref v)  => write!(f, "TokMetaOp({})", v),
		TokPreproc(ref v) => write!(f, "TokPreproc({})", v),
		TokNewline   => write!(f, "TokNewline"),
		TokComma     => write!(f, "TokComma"),
		TokColon     => write!(f, "TokColon"),
		TokAssign    => write!(f, "TokAssign"),
		TokPlus      => write!(f, "TokPlus"),
		TokMinus     => write!(f, "TokMinus"),
		TokStar      => write!(f, "TokStar"),
		TokSlash     => write!(f, "TokSlash"),
		TokBackslash => write!(f, "TokBackslash"),
		TokSqOpen    => write!(f, "TokSqOpen"),
		TokSqClose   => write!(f, "TokSqClose"),
		TokParenOpen => write!(f, "TokParenOpen"),
		TokParenClose=> write!(f, "TokParenClose"),
		
		TokComment => write!(f, "TokComment"),
		}
	}
}

impl<'rl> Lexer<'rl>
{
	pub fn new(instream: &'rl mut Reader, root_filename: &str) -> Lexer<'rl> {
		Lexer {
			instream: instream,
			filename: root_filename.to_string(),
			line: 1,
			lastchar: None,
			saved_tok: None,
		}
	}
	
	fn _getc(&mut self) -> IoResult<char> {
		let ret = match self.lastchar {
			Some(ch) => Ok(ch),
			None => Ok( try!(self.instream.read_byte()) as char )
			};
		self.lastchar = None;
		return ret
	}
	fn _putback(&mut self, ch: char) {
		self.lastchar = Some(ch)
	}
	fn eat_spaces(&mut self) -> bool {
		loop
		{
			let ch = parse_try!(self._getc(), true);
			if !isspace(ch) {
				self._putback(ch);
				break;
			}
		}
		return false;
	}
	fn read_to_eol(&mut self) -> String {
		let mut ret = String::new();
		loop
		{
			let ch = parse_try!(self._getc(), ret);
			if ch == '\n' { break; }
			ret.push_char( ch );
		}
		debug!("read_to_eol: ret = '{}'", ret);
		return ret;
	}
	fn read_ident(&mut self) -> String {
		let mut name = String::new();
		let mut ch = parse_try!(self._getc(), name);
		while isalnum(ch) || ch == '_'
		{
			name.push_char( ch );
			ch = parse_try!(self._getc(), name);
		}
		self._putback(ch);
		return name;
	}
	fn read_number(&mut self, base: uint) -> u64 {
		let mut val = 0;
		loop
		{
			let ch = parse_try!(self._getc(), val);
			match ch.to_digit(base) {
			Some(d) => {
				val *= base as u64;
				val += d as u64
				},
			None => {
				self._putback(ch);
				break;
				}
			}
		}
		return val;
	}
	/// @brief Low-level lexer
	fn get_token_int(&mut self) -> Token
	{
		macro_rules! getc( ($err_ret:expr) => ( parse_try!(self._getc(), $err_ret) ) )
		let mut ch = getc!(TokEof);
		while isspace(ch) {
			ch = getc!(TokEof);
		}
		
		//debug!("get_token_int: ch='{}'", ch);
		let ret = match ch
		{
		';' => {
			self.read_to_eol();
			TokComment
			},
		'$' => TokLine( self.read_ident() ),
		'@' => TokGroup( self.read_ident() ),
		'%' => TokPreproc( self.read_ident() ),
		'#' => TokMetaOp( self.read_ident() ),
		
		',' => TokComma,
		':' => TokColon,
		'=' => TokAssign,
		
		'+' => TokPlus,
		'-' => TokMinus,
		'*' => TokStar,
		'/' => {
			ch = getc!( TokSlash );
			match ch {
			'/' => {
				self.read_to_eol();
				TokComment
				},
			_ => TokSlash
			}
			}
		'\\' => TokBackslash,
		'[' => TokSqOpen,
		']' => TokSqClose,
		'(' => TokParenOpen,
		')' => TokParenClose,
		
		'0' => {
			ch = getc!( TokNumber(0) );
			match ch {
			'1' .. '7' => {
				self._putback(ch);
				TokNumber( self.read_number(8) )
				},
			'x' => TokNumber( self.read_number(16) ),
			'b' => TokNumber( self.read_number(2) ),
			_ => TokNumber(0)
			}
			},
		'1' .. '9' => {
			self._putback(ch);
			TokNumber( self.read_number(10) )
			},
		'a'..'z'|'A'..'Z'|'_' => {
			self._putback(ch);
			TokIdent( self.read_ident() )
			}
		_ => {
			debug!("Invalid character '{}'", ch);
			TokInval
			}
		};
		debug!("get_token_int: ret={}", ret);
		return ret;
	}
	/// @brief Wraps low-level lexer to ignore comments and handle preprocessor comments
	pub fn get_token(&mut self) -> Token
	{
		match std::mem::replace(&mut self.saved_tok, None) {
		Some(x) => return x,
		None => ()
		};
		
		loop {
			let tok = self.get_token_int();
			match tok
			{
			TokComment => (),
			TokPreproc(stmt) => {
				match stmt.as_slice()
				{
				"line" => {
					// %line <line>+<unk> <filename>
					let line = syntax_assert!(self.get_token_int(), TokNumber(x) => x, "Expected number in %line");
					syntax_assert!(self.get_token_int(), TokPlus => (), "Expected '+' in %line");
					let unk = syntax_assert!(self.get_token_int(), TokNumber(x) => x, "Expected number in %line");
					self.eat_spaces();
					let file = self.read_to_eol();
					debug!("Set Line: Line {}, Unk {}, Filename: '{}'", line, unk, file);
					},
				_ => {
					warn!("Unknown preprocessor statement '{}'", stmt);
					}
				}
				},
			_ => return tok
			}
		}
	}
	pub fn put_back(&mut self, tok: Token) {
		self.saved_tok = Some(tok)
	}
	pub fn look_ahead(&mut self) -> Token {
		let ret = self.get_token();
		self.put_back( ret.clone() );
		return ret;
	}
}

impl<'rl> Parser<'rl>
{
	fn new(instream: &'rl mut Reader, root_filename: &str) -> Parser<'rl> {
		Parser {
			lexer: Lexer::new(instream, root_filename),
		}
	}
	
	fn get_token(&mut self) -> Token { self.lexer.get_token() }
	fn look_ahead(&mut self) -> Token { self.lexer.look_ahead() }
	fn put_back(&mut self, tok: Token) { self.lexer.put_back(tok) }
	
	fn get_numeric(&mut self) -> u64
	{
		// TODO: Support arithmatic
		
		let tok = self.get_token();
		return syntax_assert!(tok, TokNumber(x) => x, "get_numeric - Expected number");
	}
	
	fn get_value(&mut self, values: &mut ::cct_mesh::LinkList, unit: &mut ::cct_mesh::Unit)
	{
		let tok = self.get_token();
		match tok
		{
		TokLine(name) => {
			let count =
				if self.look_ahead() == TokStar {
					self.get_token();
					self.get_numeric()
				}
				else {
					1
				};
			for i in range(0, count) {
				values.push( unit.get_link(&name) );
			}
			},
		TokGroup(name) => {
			let group = match unit.get_group(&name) {
				Some(x) => x,
				None => fail!("Group @{} is not defined", name)
				};
			if self.look_ahead() == TokSqOpen
			{
				self.get_token();
				loop
				{
					let start = self.get_numeric();
					if start >= group.len() as u64 {
						fail!("Index {} out of range for group @{}", start, name);
					}
					if self.look_ahead() != TokColon
					{
						// Single
						values.push( group.get(start as uint).clone() );
						fail!("TODO: Group single {} #{}", name, start);
					}
					else
					{
						// Range
						self.get_token();
						let end = self.get_numeric();
						if end >= group.len() as u64 {
							fail!("Range end {} out of range for group @{}", start, name);
						}
						fail!("TODO: Group range{} {}--{}", name, start, end);
						for i in range(start, end) {
							debug!("%{}[{}]", name, i);
						}
					}
					let tok = self.get_token();
					if( tok != TokComma ) {
						self.put_back(tok);
						break;
					}
				}
			}
			else
			{
				// Entire range
				fail!("TODO: Group content {}", name);
			}
			},
		_ => fail!("TODO: Syntax errors, get_value")
		}
	}
	
	fn do_line(&mut self, unit: &mut ::cct_mesh::Unit)
	{
		let mut tok = self.get_token();
		let mut outputs = ::cct_mesh::LinkList {..Default::default()};
		if !is_enum!(tok TokIdent(_))
		{
			// Get destination line list
			self.put_back(tok);
			loop
			{
				self.get_value(&mut outputs, unit);
				tok = self.get_token();
				if !is_enum!(tok TokComma) { break }
			}
		}
		let ident = match tok {
			TokIdent(x) => x,
			_ => fail!("Expected TokIdent")
			};
	}
}

fn isspace(ch: char) -> bool {
	unsafe {
		return libc::funcs::c95::ctype::isspace(ch as i32) != 0
	}
}
fn isalnum(ch: char) -> bool {
	unsafe {
		return libc::funcs::c95::ctype::isalnum(ch as i32) != 0
	}
}

pub fn load(filename: &str) -> Option<::cct_mesh::Root>
{
	debug!("load(filename='{}')", filename);
	// 1. Spin up a yasm preprocessor
	let mut subproc = match ::std::io::Command::new("yasm").arg("-e").arg(filename).spawn() {
		Ok(child) => child,
		Err(e) => fail!("Failed to execute yasm to preprocess file. Reason: {}", e),
		};
	let output_pipe = match subproc.stdout {
		Some(ref mut pipe) => pipe,
		None => fail!("BUGCHECK - Stdout was None"),
		};
	// 2. Create a parser object
	let mut parser = Parser::new(output_pipe, filename);
	
	// 3. Create mesh root
	let mut meshroot = ::cct_mesh::Root::new();
	{
		let curunit: Option<&mut ::cct_mesh::Unit> = None;
		
		// 4. Parse!
		loop
		{
			let tok = parser.get_token();
			match tok
			{
			TokNewline => {},
			TokEof => { println!("EOF"); break },
			TokMetaOp(name) => {
				match name.as_slice() {
				"defunit" => {
					let unitname = syntax_assert!(parser.get_token(), TokIdent(v) => v, "Expected TokIdent after #defunit");
					curunit = match meshroot.add_unit(&unitname) {
						Some(x) => Some(x),
						None => fail!("Redefinition of unit {}", unitname)
						};
					},
				_ => fail!("Unknown meta-op '#{}'", name)
				}
				},
			_ => {
				parser.put_back(tok);
				parser.do_line(match curunit { Some(ref x) => *x, None => meshroot.get_root_unit()});
				}
			}
		}
	}
	
	return Some(meshroot);
}

// vim: ft=rust
