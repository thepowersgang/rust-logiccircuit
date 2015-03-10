//
//
//
use self::Token::*;

#[derive(PartialEq,Clone)]
pub enum Token {
	TokInval,
	TokEof,
	TokNumber(u64),
	TokLine(String),
	TokGroup(String),
	TokIdent(String),
	TokString(String),
	
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
	TokBraceOpen,
	TokBraceClose,
	
	TokComment,
}

pub type InStream<'a> = &'a mut (::std::iter::Iterator<Item=char> + 'a);

pub struct Lexer<'stream>
{
	instream: InStream<'stream>,
	filename: String,
	line: u32,
	lastchar: Option<char>,
	saved_tok: Option<Token>,
}

macro_rules! parse_try{
	($e:expr, $rv:expr) => (match $e {Some(v) => v, None => {return $rv}})
}
macro_rules! syntax_error{ ($lexer:expr, $($arg:tt)*) => ({
	let p = &$lexer;
	panic!("Syntax Error: {}:{}: {}", p.filename, p.line, format!($($arg)*));
}) }
macro_rules! syntax_assert_raw{ ($parser:expr, $tok:expr, $filter:pat => $val:expr, $msg:expr) => ({
	let tok = $tok;
	match tok {
		$filter => $val,
		_ => syntax_error!($parser, "{}, got {}", $msg, tok)
	}
}) }
macro_rules! syntax_assert_get_int{ ($parser:expr, $filter:pat => $val:expr, $msg:expr) => ({
	syntax_assert_raw!($parser, ($parser).get_token_int(), $filter => $val, $msg)
}) }

impl ::std::fmt::Display for Token
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self {
		TokInval     => write!(f, "TokInval"),
		TokEof       => write!(f, "TokEof"),
		TokNumber(ref v) => write!(f, "TokNumber(0x{:x})", *v),
		TokLine(ref v)   => write!(f, "TokLine({})", v),
		TokGroup(ref v)  => write!(f, "TokGroup({})", v),
		TokIdent(ref v)  => write!(f, "TokIdent({})", v),
		TokString(ref v)  => write!(f, "String(\"{}\")", v),
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
		TokBraceOpen => write!(f, "TokBraceOpen"),
		TokBraceClose=> write!(f, "TokBraceClose"),
		
		TokComment => write!(f, "TokComment"),
		}
	}
}

impl<'rl> Lexer<'rl>
{
	pub fn new<'a>(instream: InStream<'a>, root_filename: &str) -> Lexer<'a> {
		Lexer {
			instream: instream,
			filename: root_filename.to_string(),
			line: 1,
			lastchar: None,
			saved_tok: None,
		}
	}
	pub fn curline(&self) -> uint { self.line as uint }
	
	fn _getc(&mut self) -> Option<char>
	{
		let ret = match self.lastchar {
			Some(ch) => Some(ch),
			None => self.instream.next(),
			};
		self.lastchar = None;
		return ret
	}
	fn _putback(&mut self, ch: char) {
		self.lastchar = Some(ch)
	}
	// Eat as many spaces as possible, returns 'true' on any error (EOF incl)
	fn eat_spaces(&mut self) -> bool {
		loop
		{
			let ch = parse_try!(self._getc(), true);
			if !ch.is_whitespace() || ch == '\n' {
				self._putback(ch);
				break;
			}
		}
		return false;
	}
	// Read and return the rest of the line
	fn read_to_eol(&mut self) -> String
	{
		let mut ret = String::new();
		loop
		{
			let ch = parse_try!(self._getc(), ret);
			if ch == '\n' { break; }
			ret.push( ch );
		}
		self._putback( '\n' );
		debug!("read_to_eol: ret = '{}'", ret);
		return ret;
	}
	// Read and return a sequence of "identifier" characters
	fn read_ident(&mut self) -> String
	{
		let mut name = String::new();
		let mut ch = parse_try!(self._getc(), name);
		while ch.is_alphanumeric() || ch == '_'
		{
			name.push( ch );
			ch = parse_try!(self._getc(), name);
		}
		self._putback(ch);
		return name;
	}
	// Read a number from the input stream
	fn read_number(&mut self, base: u32) -> u64
	{
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
	fn read_string(&mut self) -> Option<String> {
		let mut ret = String::new();
		loop
		{
			let ch = parse_try!(self._getc(), None);
			if ch == '\"' {
				break;
			}
			if ch == '\\' {
				let codechar = parse_try!(self._getc(), None);
				match codechar {
				'\\' => ret.push('\\'),
				'"' => ret.push('"'),
				'n' => ret.push('\n'),
				'\n' => (),
				_ => panic!("Unexpected escape code in string '\\{}'", codechar)
				}
			}
			ret.push( ch );
		}
		return Some(ret);
	}
	/// @brief Low-level lexer
	fn get_token_int(&mut self) -> Token
	{
		macro_rules! getc{ ($err_ret:expr) => ( parse_try!(self._getc(), $err_ret) ) }
		if self.eat_spaces() {
			return TokEof;
		}
		
		//debug!("get_token_int: ch='{}'", ch);
		let mut ch = getc!(TokEof);
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
		
		'\n' => {
			self.line += 1;
			TokNewline
			},
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
			_ => {
				self._putback(ch);
				TokSlash
				}
			}
			}
		'\\' => TokBackslash,
		'[' => TokSqOpen,
		']' => TokSqClose,
		'(' => TokParenOpen,
		')' => TokParenClose,
		'{' => TokBraceOpen,
		'}' => TokBraceClose,
		
		'"' => TokString( self.read_string().unwrap() ),	// TODO: Convert None into TokInval
		
		'0' => {
			ch = getc!( TokNumber(0) );
			match ch {
			'1' ... '7' => {
				self._putback(ch);
				TokNumber( self.read_number(8) )
				},
			'x' => TokNumber( self.read_number(16) ),
			'b' => TokNumber( self.read_number(2) ),
			_ => {
				self._putback(ch);
				TokNumber(0)
				}
			}
			},
		'1' ... '9' => {
			self._putback(ch);
			TokNumber( self.read_number(10) )
			},
		'a'...'z'|'A'...'Z'|'_' => {
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
		match ::std::mem::replace(&mut self.saved_tok, None) {
		Some(x) => return x,
		None => ()
		};
		
		loop
		{
			let tok = self.get_token_int();
			match tok
			{
			// Comments: Ignore
			TokComment => (),
			// NASM Preprocessor statements
			TokPreproc(stmt) => {
				match &*stmt
				{
				"line" => {
					// %line <line>+<unk> <filename>
					let line = syntax_assert_get_int!(self, TokNumber(x) => x, "Expected number in %line");
					syntax_assert_get_int!(self, TokPlus => (), "Expected '+' in %line");
					let unk = syntax_assert_get_int!(self, TokNumber(x) => x, "Expected number in %line");
					self.eat_spaces();
					let file = self.read_to_eol();
					syntax_assert_get_int!(self, TokNewline => (), "");
					debug!("Set Line: Line {}, Unk {}, Filename: '{}'", line, unk, file);
					self.line = line as u32 - 1;	// -1 to counter coming newline
					self.filename = file;
					},
				_ => {
					warn!("Unknown preprocessor statement '{}'", stmt);
					}
				}
				},
			// Backslash - Escape the meaning of another character
			TokBackslash => {
				let tok2 = self.get_token_int();
				match tok2 {
				// Ignore a newline
				TokNewline => {},
				// Explicit newline
				TokBackslash => return TokNewline,
				_ => syntax_error!(self, "Expected newline or backslash after backslash, got {}", tok2)
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
impl<'rl> ::std::fmt::Debug for Lexer<'rl>
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}:{}", self.filename, self.line)
	}
}

// vim: ft=rust

