//
//
//
use std::default::Default;
use parse::lex::*;

mod lex;

struct Parser<'stream>
{
	lexer: lex::Lexer<'stream>,
}

macro_rules! is_enum( ($val:expr $exp:pat) => (match $val { $exp => true, _ => false }) )
macro_rules! exp_enum( ($val:expr $exp:ident) => (match $val { $exp(x) => x, _ => fail!("Expected enum {}", $exp) }) )
macro_rules! parse_try( ($e:expr, $rv:expr) => (match $e {Ok(v) => v, Err(e) => {error!("Read error: {}", e); return $rv}}) )
macro_rules! syntax_error( ($lexer:expr, $($arg:tt)*) => ({
	let p = &$lexer;
	fail!("Syntax Error: {} {}", p, format!($($arg)*));
}) )
macro_rules! syntax_assert_raw( ($parser:expr, $tok:expr, $filter:pat => $val:expr, $msg:expr) => ({
	let tok = $tok;
	match tok {
		$filter => $val,
		_ => syntax_error!($parser, "{}, got {}", $msg, tok)
	}
}) )
macro_rules! syntax_warn( ($lexer:expr, $($arg:tt)*) => ({
	let p = &$lexer;
	println!("{}:warning: {}", p, format!($($arg)*));
}) )
macro_rules! syntax_assert_get( ($parser:expr, $filter:pat => $val:expr, $msg:expr) => ({
	syntax_assert_raw!($parser.lexer, ($parser).get_token(), $filter => $val, $msg)
}) )

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
	#[allow(deprecated)] // < for .get
	fn get_value(&mut self, values: &mut ::cct_mesh::LinkList, meshroot: &::cct_mesh::Root, unit: &mut ::cct_mesh::Unit)
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
			debug!("get_value: Line '{}' * {}", name, count);
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
						for i in range_inc(start as int, end as int) {
							debug!("Group item @{}[{}]", name, i);
							values.push( group.get(i as uint).clone() );
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
				else { 1 };
			
			if start >= 64 || end >= 64 {
				syntax_warn!(self.lexer, "Start or end are greater than 63 (start={}, end={})", start, end);
			}
			
			if val >> ::std::cmp::max(start,end)+1 != 0 {
				syntax_warn!(self.lexer, "Value exceeds extracted range (0x{:x} > 1<<{})", val, ::std::cmp::max(start,end)+1);
			}
			
			debug!("get_value: Constant {}[{}:{}] * {}", val, start, end, count);
			for _ in range(0,count) {
				for i in range_inc(start as int, end as int) {
					values.push( unit.get_constant( (val >> i as uint) & 1 == 1 ) );
				}
			}
			
			},
		TokParenOpen => {
			let (elename, params, inputs) = self.get_element(meshroot, unit);
			syntax_assert_get!(self, TokParenClose => (), "Expected TokParenClose after sub-element");
			values.extend( unit.append_element( meshroot, elename, params, inputs, None ).move_iter() );
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
				for line in unit.get_group(&name).unwrap().iter() {
					ret.push( line.clone() );
				}
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
	fn get_element(&mut self, meshroot: &::cct_mesh::Root, unit: &mut ::cct_mesh::Unit) -> (String, Vec<u64>, ::cct_mesh::LinkList)
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
		
		let inputs = self.get_value_list(meshroot, unit);
		
		return ( ident, params, inputs );
	}
	
	/// Read a comma-separated list of values
	fn get_value_list(&mut self, meshroot: &::cct_mesh::Root, unit: &mut ::cct_mesh::Unit) -> ::cct_mesh::LinkList
	{
		let mut values = ::cct_mesh::LinkList {..Default::default()};
		
		loop
		{
			self.get_value(&mut values, meshroot, unit);
			let tok = self.get_token();
			if !is_enum!(tok TokComma) {
				self.put_back(tok);
				break
			}
		}
		return values;
	}
	
	/// Handle a descriptor line (<outputs> = ELEMENT <inputs>)
	#[allow(deprecated)]	// < For .get(), as indexing is buggy
	fn do_line(&mut self, meshroot: &::cct_mesh::Root, unit: &mut ::cct_mesh::Unit)
	{
		let outputs = if is_enum!(self.look_ahead() TokIdent(_)) {
				::cct_mesh::LinkList {..Default::default()}
			}
			else {
				// Get destination line list
				let v = self.get_value_list(meshroot, unit);
				syntax_assert_get!(self, TokAssign => (), "Expected TokAssign");
				v
			};
		
		// If the next token is an identifier, then it's a typical descriptor
		if is_enum!(self.look_ahead() TokIdent(_))
		{
			let (name,params,inputs) = self.get_element(meshroot, unit);
			syntax_assert_get!(self, TokNewline => (), "Expected newline after element descriptor");
			unit.append_element(meshroot, name, params, inputs, Some(outputs));
		}
		// If it's not, then it's a binding operation
		else
		{
			let inputs = self.get_value_list(meshroot, unit);
			syntax_assert_get!(self, TokNewline => (), "Expected newline after rename descriptor");
			if outputs.len() != inputs.len() {
				syntax_error!(self.lexer, "Left and right counts don't match when binding ({} != {})",
					outputs.len(), inputs.len());
			}
			// Call .bind on all output lines, to set their value to that of the input
			// TODO: .bind should check that the lefthand side has not yet been rebound
			// outputs,inputs: Vec<Rc<Link>>
			for i in range(0, outputs.len()) {
				outputs.get(i).borrow_mut().bind( inputs.get(i) );
			}
		}
	}
}

fn range_inc(first: int, last: int) -> ::std::iter::RangeStep<int> {
	if first <= last {
		return ::std::iter::range_step(first, last+1, 1);
	}
	else {
		debug!("range_step({}, {}, {})", first, last-1, -1i);
		return ::std::iter::range_step(first, last-1, -1);
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
		let conditions = parser.get_value_list( meshroot, state.get_curunit() );
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after test completion condition");
		
		match state.get_curtest() {
			Some(x) => x,
			None => syntax_error!(parser.lexer, "#testcomplete outside of a test")
			}.set_completion(conditions);
		},
	"testassert" => {
		let conditions = parser.get_value_list( meshroot, state.get_curunit() );
		let values = parser.get_value_list( meshroot, state.get_curunit() );
		let expected = parser.get_value_list( meshroot, state.get_curunit() );
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
		let conditions = parser.get_value_list( meshroot, state.get_curunit() );
		let text = syntax_assert_get!(parser, TokString(x) => x, "Expected string after condtions in #display");
		let values = parser.get_value_list( meshroot, state.get_curunit() );
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after values in #display");
		
		state.get_curunit().append_display(conditions, text, values);
		},
	"block" => {
		let name = syntax_assert_get!(parser, TokString(x) => x, "Expected block name after #block");
		syntax_assert_get!(parser, TokNewline => (), "Expected newline after name in #block");
		
		warn!("TODO: Display blocks (#block \"{}\")", name);
		},
	"breakpoint" => {
		let conditions = parser.get_value_list( meshroot, state.get_curunit() );
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
				parser.do_line( &meshroot, state.get_curunit() );
				}
			}
		}
	}
	
	return Some(meshroot);
}

// vim: ft=rust
