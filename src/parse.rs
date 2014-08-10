//
//
//
extern crate libc;	// for isspace

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
macro_rules! parse_try( ($e:expr, $rv:expr) => (match $e {Ok(v) => v, Err(e) => {error!("Read error: {}", e); return $rv}}) )
macro_rules! syntax_warn( ($lexer:expr, $($arg:tt)*) => ({
	let p = &$lexer;
	println!("{}:{}:warning: {}", p.filename, p.line, format!($($arg)*));
}) )
macro_rules! syntax_error( ($lexer:expr, $($arg:tt)*) => ({
	let p = &$lexer;
	fail!("Syntax Error: {}:{}: {}", p.filename, p.line, format!($($arg)*));
}) )
macro_rules! syntax_assert_raw( ($parser:expr, $tok:expr, $filter:pat => $val:expr, $msg:expr) => ({
	let tok = $tok;
	match tok {
		$filter => $val,
		_ => syntax_error!($parser, "{}, got {}", $msg, tok)
	}
}) )
macro_rules! syntax_assert_get_int( ($parser:expr, $filter:pat => $val:expr, $msg:expr) => ({
	syntax_assert_raw!($parser, ($parser).get_token_int(), $filter => $val, $msg)
}) )
macro_rules! syntax_assert_get( ($parser:expr, $filter:pat => $val:expr, $msg:expr) => ({
	syntax_assert_raw!($parser.lexer, ($parser).get_token(), $filter => $val, $msg)
}) )

impl ::std::fmt::Show for Token
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
			if !isspace(ch) || ch == '\n' {
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
		self._putback( '\n' );
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
				'\\' => ret.push_char('\\'),
				'"' => ret.push_char('"'),
				'n' => ret.push_char('\n'),
				'\n' => (),
				_ => fail!("Unexpected escape code in string '\\{}'", codechar)
				}
			}
			ret.push_char( ch );
		}
		return Some(ret);
	}
	/// @brief Low-level lexer
	fn get_token_int(&mut self) -> Token
	{
		macro_rules! getc( ($err_ret:expr) => ( parse_try!(self._getc(), $err_ret) ) )
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
			'1' .. '7' => {
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
		match ::std::mem::replace(&mut self.saved_tok, None) {
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
			TokBackslash => {
				let tok2 = self.get_token_int();
				match tok2 {
				TokNewline => {},
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

fn range_inc(first: uint, last: uint) -> ::std::iter::RangeStep<uint> {
	if first <= last {
		return ::std::iter::range_step(first, last+1, 1);
	}
	else {
		return ::std::iter::range_step(first, last-1, -1);
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
	
	fn get_numeric_3(&mut self) -> u64 {
		return syntax_assert_get!(self, TokNumber(x) => x, "Expected numeric value");
	}
	fn get_numeric_2(&mut self) -> u64 {
		if self.look_ahead() == TokParenOpen {
			self.get_token();
			let val = self.get_numeric_0();
			syntax_assert_get!(self, TokParenClose => (), "Expecting TokParenClose in numeric");
			val
		}
		else {
			self.get_numeric_3()
		}
	}
	fn get_numeric_1(&mut self) -> u64 {
		let mut val = self.get_numeric_2();
		loop {
			let tok = self.get_token();
			match tok {
			TokStar => {
				val *= self.get_numeric_2();
				},
			TokSlash => {
				val /= self.get_numeric_2();
				},
			_ => {
				self.put_back(tok);
				break;
				}
			}
		}
		return val;
	}
	fn get_numeric_0(&mut self) -> u64 {
		let mut val = self.get_numeric_1();
		loop {
			let tok = self.get_token();
			match tok {
			TokPlus => {
				val += self.get_numeric_1();
				},
			TokMinus => {
				val -= self.get_numeric_1();
				},
			_ => {
				self.put_back(tok);
				break;
				}
			}
		}
		return val;
	}
	fn get_numeric(&mut self) -> u64
	{
		// TODO: Support arithmatic
		return self.get_numeric_0();
	}
	
	/// Read a single value (link, group, constant, or an embedded element)
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
						values.push( group[start as uint].clone() );
						debug!("Group single {} #{}", name, start);
					}
					else
					{
						// Range
						self.get_token();
						let end = self.get_numeric();
						if end >= group.len() as u64 {
							syntax_error!(self.lexer, "Range end {} out of range for group @{} (len={})",
								end, name, group.len());
						}
						for i in range_inc(start as uint, end as uint) {
							debug!("Group item @{}[{}]", name, i);
							values.push( group.get(i).clone() );
						}
					}
					let tok = self.get_token();
					if tok != TokComma {
						self.put_back(tok);
						break;
					}
				}
				syntax_assert_get!(self, TokSqClose => (), "Expected TokSqClose after range specifiers");
			}
			else
			{
				// Entire range
				for item in group.iter() {
					values.push( item.clone() );
				}
			}
			},
		TokNumber(val) => {
			let mut start = 0u;
			let mut end = 0u;
			if self.look_ahead() == TokSqOpen {
				// Extract a range of bits from the number
				self.get_token();
				start = self.get_numeric() as uint;
				syntax_assert_get!(self, TokColon => (), "Expected TokColon in literal");
				end = self.get_numeric() as uint;
				syntax_assert_get!(self, TokSqClose => (), "Expected TokSqClose after literal range");
			}
			
			let count = if self.look_ahead() == TokStar {
				self.get_token();
				self.get_numeric() as uint
				}
				else { 0 };
			
			if start >= 64 || end >= 64 {
				syntax_warn!(self.lexer, "Start or end are greater than 63 (start={}, end={})", start, end);
			}
			
			if val >> ::std::cmp::max(start,end)+1 != 0 {
				syntax_warn!(self.lexer, "Value exceeds extracted range (0x{:x} > 1<<{})", val, ::std::cmp::max(start,end)+1);
			}
			
			for _ in range(0,count) {
				for i in range_inc(start,end) {
					values.push( unit.get_constant( (val >> i) & 1 == 1 ) );
				}
			}
			
			},
		TokParenOpen => {
			let mut subele = self.get_element(unit);
			syntax_assert_get!(self, TokParenClose => (), "Expected TokParenClose after sub-element");
			values.push_all( subele.anon_outputs(unit).as_slice() );
			unit.append_element( subele );
			},
		_ => syntax_error!(self.lexer, "Expected TokLine or TokGroup when parsing value, got {}", tok)
		}
	}
	/// Read a comma-separated list of link names (does not handle constant values)
	/// \note Used for inputs and outputs (defines groups it finds)
	fn get_connections(&mut self, unit: &mut ::cct_mesh::Unit) -> ::cct_mesh::LinkList
	{
		let mut ret = ::cct_mesh::LinkList {..Default::default()};
		loop
		{
			let tok = self.get_token();
			match tok
			{
			TokLine(name) => {
				// TODO: Ensure that name does not already exist?
				ret.push( unit.get_link(&name) );
				},
			TokGroup(name) => {
				if unit.get_group(&name) != None {
					fail!("Group @{} is already defined", name)
				}
				syntax_assert_get!(self, TokSqOpen => (), "Expected TokSqOpen after group in connection list");
				let size = self.get_numeric();
				syntax_assert_get!(self, TokSqClose => (), "Expected TokSqClose after group in connection list");
				
				unit.make_group(&name, size as uint);
				},
			_ => fail!("Syntax error - Expected TokLine or TokGroup in connection list, got {}", tok)
			}
			
			let comma = self.get_token();
			if comma != TokComma
			{
				self.put_back(comma);
				break;
			}
		}
		return ret;
	}
	
	/// Read an element (<ELEMENT> <INPUTS>), leaving the inputs unbound
	fn get_element(&mut self, unit: &mut ::cct_mesh::Unit) -> ::cct_mesh::Element
	{
		let ident = syntax_assert_get!(self, TokIdent(x) => x, "Expected TokIdent");
		let params = if self.look_ahead() == TokBraceOpen
			{
				let mut params = Vec::new();
				self.get_token();
				loop
				{
					params.push( self.get_numeric() );
					if self.look_ahead() != TokComma {
						break;
					}
					self.get_token();
				}
				syntax_assert_get!(self, TokBraceClose => (), "Expected brace close after parameters");
				params
			}
			else
			{
				Vec::new()
			};
		
		let inputs = self.get_value_list(unit);
		
		let ele = ::cct_mesh::Element::new( ident, params, inputs );
		
		return ele;
	}
	
	/// Read a comma-separated list of values
	fn get_value_list(&mut self, unit: &mut ::cct_mesh::Unit) -> ::cct_mesh::LinkList
	{
		let mut values = ::cct_mesh::LinkList {..Default::default()};
		
		loop
		{
			self.get_value(&mut values, unit);
			let tok = self.get_token();
			if !is_enum!(tok TokComma) {
				self.put_back(tok);
				break
			}
		}
		return values;
	}
	
	/// Handle a descriptor line (<outputs> = ELEMENT <inputs>)
	fn do_line(&mut self, unit: &mut ::cct_mesh::Unit)
	{
		let outputs = if is_enum!(self.look_ahead() TokIdent(_)) {
				::cct_mesh::LinkList {..Default::default()}
			}
			else {
				// Get destination line list
				let v = self.get_value_list(unit);
				syntax_assert_get!(self, TokAssign => (), "Expected TokAssign");
				v
			};
		
		// If the next token is an identifier, then it's a typical descriptor
		if is_enum!(self.look_ahead() TokIdent(_))
		{
			let mut ele = self.get_element(unit);
			syntax_assert_get!(self, TokNewline => (), "Expected newline after element descriptor");
			ele.set_outputs( outputs );
			unit.append_element(ele);
		}
		// If it's not, then it's a binding operation
		else
		{
			let inputs = self.get_value_list(unit);
			syntax_assert_get!(self, TokNewline => (), "Expected newline after rename descriptor");
			if outputs.len() != inputs.len() {
				syntax_error!(self.lexer, "Left and right counts don't match when binding ({} != {})",
					outputs.len(), inputs.len());
			}
			// Call .bind on all output lines, to set their value to that of the input
			// TODO: .bind should check that the lefthand side has not yet been rebound
			// outputs,inputs: Vec<Rc<Link>>
			for i in range(0, outputs.len()) {
				outputs.get(i).borrow_mut().bind( inputs.get(i).borrow().deref() );
			}
		}
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

/// @brief Wraps 'curunit' as a reassignable reference
/// Wrapper for curunit due to rust #6393 - Borrow checker doesn't expire borrows on re-assignment
struct RootState {
	curunit: *mut ::cct_mesh::Unit,
	curtest: Option<*mut ::cct_mesh::Test>,
}
impl RootState {
	pub fn new(initunit: &mut ::cct_mesh::Unit) -> RootState {
		RootState {
			curunit: initunit as *mut _,
			curtest: None,
		}
	}
	pub fn set_curunit(&mut self, unit: &mut ::cct_mesh::Unit) {
		self.curunit = unit as *mut _;
		self.curtest = None;
	}
	pub fn set_curtest(&mut self, test: &mut ::cct_mesh::Test) {
		self.curtest = Some(test as *mut _);
		self.curunit = test.get_unit() as *mut _;
	}
	pub fn get_curunit(&self) -> &mut ::cct_mesh::Unit {
		unsafe { &mut *self.curunit }
	}
	pub fn get_curtest(&self) -> Option<&mut ::cct_mesh::Test> {
		unsafe {
			match self.curtest {
			Some(x) => Some(&mut *x),
			None => None
			}
		}
	}
}

fn handle_meta(parser: &mut Parser, meshroot: &mut ::cct_mesh::Root, state: &mut RootState, name: String)
{
	match name.as_slice()
	{
	"defunit" => {
		let unitname = syntax_assert_get!(parser, TokIdent(v) => v, "Expected TokIdent after #defunit");
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after #defunit");
		
		//if state.get_curunit() != meshroot.get_root_unit() {
		//	syntax_error!(parser.lexer, "#defunit outside of root");
		//}
		
		match meshroot.add_unit(&unitname) {
			Some(x) => state.set_curunit(x),
			None => fail!("Redefinition of unit {}", unitname)
			};
		},
	"input" => {
		// Parse a list of lines into a vector
		let conns = parser.get_connections(state.get_curunit());
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after input list");
		
		if state.get_curunit().set_input( conns ) {
			fail!("Redefinition of unit inputs");
		}
		},
	"output" => {
		// Parse a list of lines into a vector
		let conns = parser.get_connections(state.get_curunit());
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after output list");
		
		if state.get_curunit().set_output( conns ) {
			fail!("Redefinition of unit outputs");
		}
		},
	"array" => {
		let name = syntax_assert_get!(parser, TokIdent(x) => x, "Expected group name after #array");
		let size = parser.get_numeric() as uint;
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after group definition");
		
		state.get_curunit().make_group(&name, size as uint);
		},
	"endunit" => {
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after #endunit");

		state.set_curunit( meshroot.get_root_unit() );
		},
	"testcase" => {
		let limit = syntax_assert_get!(parser, TokNumber(v) => (v), "Expected number after #testcase");
		let name = syntax_assert_get!(parser, TokString(v) => (v), "Expected test name after execution limit");
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after test case definition");
		
		//if state.get_curunit() != meshroot.get_root_unit() {
		//	syntax_error!(parser.lexer, "#testcase outside of root");
		//}
		
		match meshroot.add_test(&name, limit as uint) {
			Some(x) => state.set_curtest( x ),
			None => fail!("Redefinition of test \"{}\"", name)
			};
		},
	"testcomplete" => {
		let conditions = parser.get_value_list( state.get_curunit() );
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after test completion condition");
		
		match state.get_curtest() {
			Some(x) => x,
			None => syntax_error!(parser.lexer, "#testcomplete outside of a test")
			}.set_completion(conditions);
		},
	"testassert" => {
		let conditions = parser.get_value_list( state.get_curunit() );
		let values = parser.get_value_list( state.get_curunit() );
		let expected = parser.get_value_list( state.get_curunit() );
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after test case definition");
		
		match state.get_curtest() {
			Some(x) => x,
			None => syntax_error!(parser.lexer, "#testassert outside of a test")
			}.add_assert(conditions, values, expected);
		},
	"endtestcase" => {
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after #endtestcase");
		state.set_curunit( meshroot.get_root_unit() );
		},
	"display" => {
		let conditions = parser.get_value_list( state.get_curunit() );
		let text = syntax_assert_get!(parser, TokString(x) => x, "Expected string after condtions in #display");
		let values = parser.get_value_list( state.get_curunit() );
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after values in #display");
		
		state.get_curunit().append_display(conditions, text, values);
		},
	"block" => {
		let name = syntax_assert_get!(parser, TokString(x) => x, "Expected block name after #block");
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after name in #block");
		
		warn!("TODO: Display blocks (#block \"{}\")", name);
		},
	"breakpoint" => {
		let conditions = parser.get_value_list( state.get_curunit() );
		let name = syntax_assert_get!(parser, TokString(x) => x, "Expected string after conditions in #breakpoint");
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after name in #breakpoint");
		
		state.get_curunit().append_breakpoint(name, conditions);
		},
	"endblock" => {
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after #endblock");
		},
	_ => fail!("Unknown meta-op '#{}'", name)
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
		let mut state = RootState::new(meshroot.get_root_unit());
		
		// 4. Parse!
		loop
		{
			let tok = parser.get_token();
			match tok
			{
			TokNewline => {},	// ignore newlines
			TokEof => { println!("EOF"); break },
			TokMetaOp(name) => handle_meta(&mut parser, &mut meshroot, &mut state,  name),
			_ => {
				parser.put_back(tok);
				parser.do_line( state.get_curunit() );
				}
			}
		}
	}
	
	return Some(meshroot);
}

// vim: ft=rust
