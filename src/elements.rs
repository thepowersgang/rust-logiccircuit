//
//
//
use std::default::Default;
use simulator::read_uint;

/*
pub enum Errcode
{
	UnknownEle,
	ParamCount(uint, uint, &'static str),
	ParamRange(uint, uint, uint),
	InputCount(uint, uint),
}
*/

pub trait Element
{
	fn name(&self) -> String;
	fn get_outputs(&self, n_inputs: uint) -> uint;
	fn dup(&self) -> Box<Element>;
	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>);
}

fn write_uint(outlines: &mut Vec<bool>, base: uint, count: uint, val: u64) {
	for i in range(0,count)
	{
		if (val & 1u64 << i) != 0
		{
			*(outlines.get_mut(base+i)) = true;
		}
	}
}

macro_rules! get_or( ($vec:expr, $idx:expr, $def:expr) => ({let _i=$idx; let _v=$vec; (if _i < _v.len(){_v[_i]}else{$def})}) )

pub fn create(name: &String, params: &[u64], n_inputs: uint) -> Result<Box<Element+'static>,String>
{
	match name.as_slice()
	{
	// Meta-gates
	"DELAY" => ElementDELAY::new(params, n_inputs),
	"PULSE" => ElementPULSE::new(params, n_inputs),
	"HOLD"  => ElementHOLD::new(params, n_inputs),
	"ENABLE" => ElementENABLE::new(params, n_inputs),
	
	// Builtin Units
	"LATCH" => ElementLATCH::new(params, n_inputs),
	"MUX" => ElementMUX::new(params, n_inputs),
	"DEMUX" => ElementDEMUX::new(params, n_inputs),
	"SEQUENCER" => ElementSEQUENCER::new(params, n_inputs),
	"MEMORY_DRAM" => ElementMEMORY_DRAM::new(params, n_inputs),
	
	// Logic Gates
	"AND" => ElementAND::new(params, n_inputs),
	"OR"  => ElementOR::new(params, n_inputs),
	"XOR" => ElementXOR::new(params, n_inputs),
	"NOR" => ElementNOR::new(params, n_inputs),
	"NXOR" => ElementNXOR::new(params, n_inputs),
	"XNOR" => ElementNXOR::new(params, n_inputs),	// < same
	"NOT" => ElementNOT::new(params, n_inputs),
	_ => return Err("Unknown element".to_string())
	}
}

#[deriving(Clone)]
struct ElementDELAY
{
	count: uint,
	idx: uint,
	vals: Vec<bool>,
}
impl ElementDELAY
{
	pub fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element+'static>,String>
	{
		let count = get_or!(params, 0, 1u64) as uint - 1;
		Ok( box ElementDELAY { count: count, idx: 0, vals: Vec::<bool>::from_fn(count*n_inputs, |_| false),} as Box<Element> )
	}
}
impl Element for ElementDELAY
{
	fn name(&self) -> String {
		return format!("ElementDELAY{{{}}}", self.count+1);
	}
	fn get_outputs(&self, n_inputs: uint) -> uint
	{
		return n_inputs;
	}
	
	fn dup(&self) -> Box<Element> {
		(box self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		assert!( inlines.len() == outlines.len() );
		if self.count == 0
		{
			for i in range(0, inlines.len())
			{
				*(outlines.get_mut(i)) |= inlines[i];
			}
		}
		else
		{
			let baseidx = self.idx * inlines.len();
			for (i,line) in inlines.iter().enumerate()
			{
				*(outlines.get_mut(i)) = self.vals[ baseidx + i ];
				*(self.vals.get_mut(baseidx + i)) = *line;
			}
			
			self.idx += 1;
			if self.idx == self.count {
				self.idx = 0;
			}
		}
	}
}


struct ElementENABLE;
impl ElementENABLE
{
	fn new(_/*params*/: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		if n_inputs < 2 {
			return Err(format!("Incorrect input count, expected at least two"));
		}
		
		return Ok( box ElementENABLE as Box<Element> );
	}
}
impl Element for ElementENABLE
{
	fn name(&self) -> String {
		return format!("ElementENABLE");
	}
	fn get_outputs(&self, n_inputs: uint) -> uint {
		return n_inputs - 1;
	}
	
	fn dup(&self) -> Box<Element> {
		box ElementENABLE as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		if inlines[0]
		{
				for (i,line) in outlines.mut_iter().enumerate()
				{
					*line = inlines[1+i];
				}
		}
	}
}

#[deriving(Clone)]
#[deriving(Default)]
struct ElementPULSE
{
	dir_is_falling: bool,
	last_value: bool,
}
impl ElementPULSE
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		let dir = match params.len() {
			0 => 0,
			1 => params[0] as uint,
			_ => return Err(format!("Too many parameter, expected only one")),
			};
		if n_inputs != 1 {
			return Err(format!("Incorrect input count, expected one"));
		}
		
		return Ok( box ElementPULSE { dir_is_falling: dir != 0, last_value: false } as Box<Element> );
	}
}
impl Element for ElementPULSE
{
	fn name(&self) -> String {
		return format!("ElementPULSE{{{}}}", self.dir_is_falling);
	}
	fn get_outputs(&self, _/*n_inputs*/: uint) -> uint {
		return 1;
	}
	
	fn dup(&self) -> Box<Element> {
		box self.clone() as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let curval = inlines[0];
		
		// Transition?
		// Pulse on the relevant transition
		if curval != self.last_value && self.last_value == self.dir_is_falling
		{
			*(outlines.get_mut(0)) = true;
		}
		self.last_value = curval;
	}
}

#[deriving(Clone)]
struct ElementHOLD
{
	hold_time: uint,
	times: Vec<uint>,
}
impl ElementHOLD
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		let time = match params.len() {
			0 => 1,
			1 => params[0] as uint,
			_ => return Err(format!("Too many parameters, expected only one")),
			};
		
		return Ok( box ElementHOLD { hold_time: time, times: Vec::from_elem(n_inputs, 0) } as Box<Element> );
	}
}
impl Element for ElementHOLD
{
	fn name(&self) -> String {
		return format!("ElementHOLD{{{}}}", self.hold_time);
	}
	fn get_outputs(&self, n_inputs: uint) -> uint {
		return n_inputs;
	}
	
	fn dup(&self) -> Box<Element> {
		box self.clone() as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		for (i,line) in inlines.iter().enumerate()
		{
			if *line {
				*(self.times.get_mut(i)) = self.hold_time;
			}
			if self.times[i] > 0 {
				*(outlines.get_mut(i)) |= true;
				*(self.times.get_mut(i)) -= 1;
			}
		}
	}
}

macro_rules! def_logic( ($name:ident, $init:expr, $op:expr, $finish:expr) => (
struct $name
{
	bussize: uint,
	buscount: uint,
}
impl $name
{
	pub fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		let bussize  = get_or!(params, 0, 1u64) as uint;
		let buscount = get_or!(params, 1, 1u64) as uint;
		let min_inputs = bussize * buscount;
		if n_inputs < min_inputs {
			Err( format!("Too few inputs, need at least {}, got {}", min_inputs, n_inputs) )
		}
		else {
			Ok( box $name { bussize: bussize, buscount: buscount } as Box<Element> )
		}
	}
}
impl Element for $name
{
	fn name(&self) -> String {
		return format!("{}{{{},{}}}", stringify!($name), self.bussize, self.buscount);
	}
	fn get_outputs(&self, _n_inputs: uint) -> uint {
		return self.bussize;
	}
	
	fn dup(&self) -> Box<Element> {
		box $name { bussize:self.bussize,buscount:self.buscount} as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let fixed_lines = inlines.len()-self.bussize*self.buscount;
		let mut val = $init;
		for i in range(0, fixed_lines)
		{
			let inval = inlines[i];
			val = $op(val,inval);
		}
		let baseval = val;
		
		for i in range(0, self.bussize)
		{
			let ofs = fixed_lines + i;
			val = baseval;
			for j in range(0, self.buscount)
			{
				let inval = inlines[ofs + j*self.bussize];
				val = $op(val, inval);
			}
			val = $finish(val);
			*(outlines.get_mut( i )) |= val;
		}
	}
}
) )

def_logic!( ElementNXOR, false, |v:bool,i:bool| v^i, |v:bool| !v )
def_logic!( ElementNOR,  false, |v:bool,i:bool| v|i, |v:bool| !v )
def_logic!( ElementXOR,  false, |v:bool,i:bool| v^i, |v| v )
def_logic!( ElementAND,  true,  |v:bool,i:bool| v&i, |v| v )
def_logic!( ElementOR,   false, |v:bool,i:bool| v|i, |v| v )

struct ElementNOT;
impl ElementNOT
{
	fn new(_/*params*/: &[u64], _/*n_inputs*/: uint) -> Result<Box<Element>,String>
	{
		return Ok( box ElementNOT as Box<Element> );
	}
}
impl Element for ElementNOT
{
	fn name(&self) -> String {
		return format!("ElementNOT");
	}
	fn get_outputs(&self, n_inputs: uint) -> uint {
		return n_inputs;
	}
	
	fn dup(&self) -> Box<Element> {
		box ElementNOT as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		for (i,line) in outlines.mut_iter().enumerate()
		{
			*line = !inlines[i];
		}
	}
}

//
//
//
#[deriving(Clone)]
#[deriving(Default)]
struct ElementLATCH
{
	vals: Vec<bool>,
}
impl ElementLATCH
{
	pub fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		let size = get_or!(params, 0, 1u64) as uint;
		if size == 0 {
			return Err( format!("Size invalid, must be non-zero") );
		}
		if n_inputs != 2 + size {
			return Err( format!("Invalid input count, expected {}, got {}", n_inputs, 2+size) );
		}
		Ok( box ElementLATCH { vals: Vec::from_elem(size, false), ..Default::default() } as Box<Element> )
	}
}
impl Element for ElementLATCH
{
	fn name(&self) -> String {
		return format!("ElementLATCH{{{}}}", self.vals.len());
	}
	fn get_outputs(&self, _: uint) -> uint {
		return 1+self.vals.len();
	}
	
	fn dup(&self) -> Box<Element> {
		(box self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let enable = inlines[0];
		let reset = inlines[1];
		let in_ofs = 2u;
		
		if enable
		{
			*(outlines.get_mut(0)) = true;
			
			if reset {
				for v in self.vals.mut_iter() {
					*v = false;
				}
			}
			else {
				for (i,v) in self.vals.mut_iter().enumerate() {
					if inlines[in_ofs+i]
					{
						*v = true;
					}
				}
			}
			
			for (i,v) in self.vals.iter().enumerate() {
				*(outlines.get_mut(1+i)) = *v;
			}
		}
	}
}
struct ElementMUX
{
	bits: uint,
	bussize: uint,
}
impl ElementMUX
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		if params.len() > 2 { return Err(format!("Too many parameters, expected at most 2")); }
		let bits    = if params.len() >= 1 { params[0] as uint } else { 1 };
		let bussize = if params.len() >= 2 { params[1] as uint } else { 1 };

		if bits == 0 || bits > 10 {
			return Err(format!("Bit count sanity check failure, must be 1--10 inclusive (got {})", bits));
		}
		if bussize == 0 {
			return Err(format!("Bus size sanity check failure, must >0 (got {})", bussize));
		}
		
		let exp_inputs: uint =  1 + bits + (1 << bits)*bussize;
		if n_inputs != exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		return Ok( box ElementMUX{bits: bits, bussize: bussize} as Box<Element> );
	}
}
impl Element for ElementMUX
{
	fn name(&self) -> String {
		return format!("ElementMUX{{{},{}}}", self.bits,self.bussize);
	}
	fn get_outputs(&self, _/*n_inputs*/: uint) -> uint {
		return self.bussize;
	}
	
	fn dup(&self) -> Box<Element> {
		box ElementMUX { bits:self.bits, bussize:self.bussize } as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let enable = inlines[0];
		let index = read_uint(inlines, 1, self.bits) as uint;
		let ofs = 1 + self.bits;
		if enable
		{
			for i in range(0,self.bussize)
			{
				*(outlines.get_mut(i)) |= inlines[ofs+index*self.bussize+i];
			}
			
		}
	}
}

struct ElementDEMUX
{
	bits: uint
}
impl ElementDEMUX
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		let (bits, bussize) = match params.len()
				{
				0 => return Err(format!("Not enough parameters, need at least 1")),
				1 => (params[0] as uint, 1),
				2 => (params[0] as uint, params[1] as uint),
				_ => return Err(format!("Too many parameters, at most 2")),
				};
		if bits == 0 || bits > 10 {
			return Err(format!("Bit count sanity check failure, must be 1--10 inclusive (got {})", bits));
		}
		let exp_inputs: uint = 1 + bits + bussize;
		if n_inputs < exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		return Ok( box ElementDEMUX{bits: bits} as Box<Element> );
	}
}
impl Element for ElementDEMUX
{
	fn name(&self) -> String {
		return format!("ElementDEMUX{{{}}}", self.bits);
	}
	fn get_outputs(&self, n_inputs: uint) -> uint {
		let bussize = n_inputs - 1 - self.bits;
		return bussize << self.bits as uint;
	}
	
	fn dup(&self) -> Box<Element> {
		box ElementDEMUX { bits:self.bits } as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let enable = inlines[0];
		let index = read_uint(inlines, 1, self.bits) as uint;
		let ofs = 1 + self.bits;
		let bussize = inlines.len() - ofs;
		if enable
		{
			for i in range(0,bussize)
			{
				*(outlines.get_mut(index*bussize+i)) |= inlines[ofs+i];
			}
		}
	}
}

#[deriving(Clone)]
struct ElementSEQUENCER
{
	count: uint,
	position: uint,
}
impl ElementSEQUENCER
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		if params.len() != 1 {
			return Err(format!("Invalid parameter count, expected 1, got {}", params.len()));
		}
		let count = params[0] as uint;
		let exp_inputs: uint = 3;
		if n_inputs != exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		return Ok( box ElementSEQUENCER { count: count, position: 0 } as Box<Element> );
	}
}
impl Element for ElementSEQUENCER
{
	fn name(&self) -> String {
		return format!("ElementSEQUENCER{{{}}}", self.count);
	}
	fn get_outputs(&self, _/*n_inputs*/: uint) -> uint {
		return self.count;
	}
	
	fn dup(&self) -> Box<Element> {
		box self.clone() as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		let enable = inlines[0];
		let reset  = inlines[1];
		let next   = inlines[2];
		
		if enable
		{
			if reset
			{
				self.position = 0
			}
			else if next
			{
				self.position = (self.position + 1) % self.count;
			}
			else
			{
				// keep
			}
			
			*(outlines.get_mut(self.position)) = true;
		}
	}
}

#[allow(non_camel_case_types)]
#[deriving(Default)]
#[deriving(Clone)]
struct ElementMEMORY_DRAM
{
	wordsize: uint,
	addrbits: uint,
	data: Vec<u32>,
}
impl ElementMEMORY_DRAM
{
	fn new(params: &[u64], n_inputs: uint) -> Result<Box<Element>,String>
	{
		if params.len() != 2 {
			return Err(format!("Invalid parameter count, expected 2, got {}", params.len()));
		}
		let wordsize = params[0] as uint;
		if wordsize == 0 || wordsize > 64 {
			return Err(format!("Word size sanity check failure, must be 0--64 (got {})", wordsize));
		}
		let addrbits = params[1] as uint;
		if addrbits == 0 || addrbits > 20 {	// allows up to 8MiB
			return Err(format!("Address bit sanity check failure, must be 1--14 (got {})", addrbits));
		}
		let exp_inputs: uint = 1 + addrbits + 1 + 2*wordsize;
		if n_inputs != exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		return Ok( box ElementMEMORY_DRAM {wordsize: wordsize, addrbits: addrbits, ..Default::default()} as Box<Element> );
	}
}
impl Element for ElementMEMORY_DRAM
{
	fn name(&self) -> String {
		return format!("ElementMEMORY_DRAM{{{},{}}}", self.wordsize, self.addrbits);
	}
	fn get_outputs(&self, _/*n_inputs*/: uint) -> uint {
		return 1 + 1 << self.wordsize as uint;
	}
	
	fn dup(&self) -> Box<Element> {
		box self.clone() as Box<Element>
	}

	fn update(&mut self, outlines: &mut Vec<bool>, inlines: &Vec<bool>)
	{
		if self.data.len() == 0 {
			self.data = Vec::from_elem( 1u << self.addrbits * self.wordsize / 32, 0 );
		}
		let enable = inlines[0];
		let wordnum = read_uint(inlines, 1, self.addrbits) as uint;
		let write = inlines[1+self.addrbits];
		let writemask = read_uint(inlines, 1+self.addrbits+1, self.wordsize);
		let writeval = read_uint(inlines, 1+self.addrbits+1+self.wordsize, self.wordsize);
		
		assert!( self.wordsize == 32 );
		if enable
		{
			if write
			{
				let val = self.data.get_mut(wordnum);
				*val &= !writemask as u32;
				*val |= writeval as u32;
			}
			write_uint(outlines, 1, self.wordsize, self.data[wordnum] as u64);
		}
	}
}


// vim: ft=rust
