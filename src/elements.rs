//
//
//
use std::default::Default;
use std::rc::Rc;
use simulator::read_uint;

//pub enum Error
//{
//	UnknownElement,
//	ParamCount {
//		expected: usize,
//		have: usize,
//		reason: &'static str
//		},
//	ParamRange {
//		param_index: usize,
//		min: usize,
//		max: usize
//		},
//	InputCount {
//		min: usize,
//		max: usize
//		},
//}

pub trait Element //: ::std::fmt::Display
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult where Self: Sized;
	fn finalise(&mut self, unit: &::cct_mesh::Unit) {}
	fn name(&self) -> String;
	fn get_outputs(&self, n_inputs: usize) -> usize;
	fn dup(&self) -> Box<Element+'static>;
	fn update(&mut self, outlines: &mut [bool], inlines: &[bool]);
}

pub type NewEleResult = Result<Box<Element+'static>,String>;

fn write_uint(outlines: &mut [bool], base: usize, count: u8, val: u64)
{
	for i in 0 .. count as usize
	{
		if (val & 1u64 << i) != 0
		{
			outlines[base+i] = true;
		}
	}
}

macro_rules! get_or{
	($vec:expr, $idx:expr, $def:expr) => ({let _i=$idx; let _v=$vec; (if _i < _v.len(){_v[_i]}else{$def})}) 
}

pub fn create(name: &str, params: &[u64], n_inputs: usize) -> NewEleResult
{
	match name
	{
	// Meta-gates
	"DELAY" => ElementDELAY::new(params, n_inputs),
	"PULSE" => ElementPULSE::new(params, n_inputs),
	"HOLD"  => ElementHOLD::new(params, n_inputs),
	"ENABLE" => ElementENABLE::new(params, n_inputs),
	
	// Builtin Units
	"CLOCK" => ElementClock::new(params, n_inputs),
	"JKFLIPFLOP" => ElementJkFlipFlop::new(params, n_inputs),
	"LATCH" => ElementLATCH::new(params, n_inputs),
	"MUX" => ElementMUX::new(params, n_inputs),
	"DEMUX" => ElementDEMUX::new(params, n_inputs),
	"SEQUENCER" => ElementSEQUENCER::new(params, n_inputs),
	"MEMORY_DRAM" => ElementMEMORY_DRAM::new(params, n_inputs),

	"ROM" => ElementROM::new(params, n_inputs),
	
	// Logic Gates
	"AND" => ElementAND::new(params, n_inputs),
	"OR"  => ElementOR::new(params, n_inputs),
	"XOR" => ElementXOR::new(params, n_inputs),
	"NAND" => ElementNAND::new(params, n_inputs),
	"NOR" => ElementNOR::new(params, n_inputs),
	"NXOR" => ElementNXOR::new(params, n_inputs),
	"XNOR" => ElementNXOR::new(params, n_inputs),	// < same
	"NOT" => ElementNOT::new(params, n_inputs),
	_ => return Err("Unknown element".to_string())
	}
}

#[derive(Clone)]
struct ElementDELAY
{
	count: usize,
	idx: usize,
	vals: Vec<bool>,
}
impl ElementDELAY
{
}
impl Element for ElementDELAY
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let count = get_or!(params, 0, 1u64) as usize - 1;
		Ok( Box::new(ElementDELAY {
			count: count,
			idx: 0,
			vals: ::from_elem(count*n_inputs, false),
			} ) )
	}
	fn name(&self) -> String {
		return format!("ElementDELAY{{{}}}", self.count+1);
	}
	fn get_outputs(&self, n_inputs: usize) -> usize
	{
		return n_inputs;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		assert!( inlines.len() == outlines.len() );
		if self.count == 0
		{
			for i in 0 .. inlines.len()
			{
				outlines[i] |= inlines[i];
			}
		}
		else
		{
			let baseidx = self.idx * inlines.len();
			for (i,line) in inlines.iter().enumerate()
			{
				outlines[i] = self.vals[ baseidx + i ];
				self.vals[baseidx + i] = *line;
			}
			
			self.idx += 1;
			if self.idx == self.count {
				self.idx = 0;
			}
		}
	}
}


struct ElementENABLE;
impl Element for ElementENABLE
{
	fn new(_/*params*/: &[u64], n_inputs: usize) -> NewEleResult
	{
		if n_inputs < 2 {
			return Err(format!("Incorrect input count, expected at least two"));
		}
		
		return Ok( Box::new(ElementENABLE) as Box<Element> );
	}
	fn name(&self) -> String {
		return format!("ElementENABLE");
	}
	fn get_outputs(&self, n_inputs: usize) -> usize {
		return n_inputs - 1;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(ElementENABLE) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		if inlines[0]
		{
			for (i,line) in outlines.iter_mut().enumerate()
			{
				*line = inlines[1+i];
			}
		}
	}
}

#[derive(Clone,Default)]
struct ElementPULSE
{
	dir_is_falling: bool,
	last_value: bool,
}
impl Element for ElementPULSE
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let dir = match params.len() {
			0 => false,
			1 => params[0] != 0,
			_ => return Err(format!("Too many parameter, expected only one")),
			};
		if n_inputs != 1 {
			return Err(format!("Incorrect input count, expected one"));
		}
		
		return Ok( Box::new(ElementPULSE { dir_is_falling: dir, last_value: false }) as Box<Element> );
	}
	fn name(&self) -> String {
		return format!("ElementPULSE{{{}}}", self.dir_is_falling);
	}
	fn get_outputs(&self, _/*n_inputs*/: usize) -> usize {
		return 1;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let curval = inlines[0];
		
		// Transition?
		// Pulse on the relevant transition
		if curval != self.last_value && self.last_value == self.dir_is_falling
		{
			outlines[0] = true;
		}
		self.last_value = curval;
	}
}

#[derive(Clone)]
struct ElementHOLD
{
	hold_time: usize,
	times: Vec<usize>,
}
impl Element for ElementHOLD
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let time = match params.len() {
			0 => 1,
			1 => params[0] as usize,
			_ => return Err(format!("Too many parameters, expected only one")),
			};
		
		return Ok( Box::new(ElementHOLD { hold_time: time, times: ::from_elem(n_inputs, 0) }) as Box<Element> );
	}
	fn name(&self) -> String {
		return format!("ElementHOLD{{{}}}", self.hold_time);
	}
	fn get_outputs(&self, n_inputs: usize) -> usize {
		return n_inputs;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		for (i,line) in inlines.iter().enumerate()
		{
			if *line {
				self.times[i] = self.hold_time;
			}
			if self.times[i] > 0 {
				outlines[i] |= true;
				self.times[i] -= 1;
			}
		}
	}
}

#[derive(Clone)]
struct ElementClock
{
	period: usize,
	duty: usize,

	counter: usize,
}
impl ElementClock
{
}
impl Element for ElementClock
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let period = get_or!(params, 0, 1u64) as usize - 1;
		let duty = get_or!(params, 1, 1u64) as usize;
		Ok( Box::new(ElementClock {
			period,
			duty,
			counter: 0,
			} ) )
	}
	fn name(&self) -> String {
		format!("ElementClock{{{},{}}}", self.period,self.duty)
	}
	fn get_outputs(&self, n_inputs: usize) -> usize {
		1
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		if inlines[0]
		{
			self.counter += 1;
			if self.counter >= self.period {
				self.counter = 0;
			}
			if self.counter < self.duty {
				outlines[0] = true;
			}
		}
	}
}

macro_rules! def_logic{ ($name:ident, $init:expr, $op:expr, $finish:expr) => (
#[derive(Clone)]
struct $name
{
	bussize: u8,
	buscount: u8,
}
impl Element for $name
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let bussize  = get_or!(params, 0, 1u64) as u8;
		let buscount = get_or!(params, 1, 1u64) as u8;
		let min_inputs = (bussize as usize) * (buscount as usize);
		if n_inputs < min_inputs {
			Err( format!("Too few inputs, need at least {}, got {}", min_inputs, n_inputs) )
		}
		else {
			Ok( Box::new($name { bussize: bussize, buscount: buscount }) as Box<Element> )
		}
	}
	fn name(&self) -> String {
		return format!("{}{{{},{}}}", stringify!($name), self.bussize, self.buscount);
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		return self.bussize as usize;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let fixed_lines = inlines.len() - (self.bussize as usize)*(self.buscount as usize);
		let mut val = $init;
		for i in 0 .. fixed_lines
		{
			let inval = inlines[i];
			val = $op(val,inval);
		}
		let baseval = val;
		
		for i in 0 .. self.bussize as usize
		{
			let ofs = fixed_lines + i;
			val = baseval;
			for j in 0 .. self.buscount as usize
			{
				let inval = inlines[ofs + j * (self.bussize as usize)];
				val = $op(val, inval);
			}
			val = $finish(val);
			outlines[i] |= val;
		}
	}
}
) }

def_logic!{ ElementNXOR, false, |v:bool,i:bool| v^i, |v:bool| !v }
def_logic!{ ElementNAND, true,  |v:bool,i:bool| v&i, |v:bool| !v }
def_logic!{ ElementNOR,  false, |v:bool,i:bool| v|i, |v:bool| !v }
def_logic!{ ElementXOR,  false, |v:bool,i:bool| v^i, |v| v }
def_logic!{ ElementAND,  true,  |v:bool,i:bool| v&i, |v| v }
def_logic!{ ElementOR,   false, |v:bool,i:bool| v|i, |v| v }

struct ElementNOT;
impl Element for ElementNOT
{
	fn new(_/*params*/: &[u64], _/*n_inputs*/: usize) -> NewEleResult
	{
		Ok( Box::new(ElementNOT) as Box<Element> )
	}
	fn name(&self) -> String {
		format!("ElementNOT")
	}
	fn get_outputs(&self, n_inputs: usize) -> usize {
		n_inputs
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(ElementNOT) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		for (i,line) in outlines.iter_mut().enumerate()
		{
			*line = !inlines[i];
		}
	}
}

//
//
//
#[derive(Clone)]
#[derive(Default)]
struct ElementLATCH
{
	vals: Vec<bool>,
}
impl Element for ElementLATCH
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let size = get_or!(params, 0, 1u64) as usize;
		if size == 0 {
			return Err( format!("Size invalid, must be non-zero") );
		}
		if n_inputs != 2 + size {
			return Err( format!("Invalid input count, expected {}, got {}", n_inputs, 2+size) );
		}
		Ok( Box::new(ElementLATCH { vals: ::from_elem(size, false), ..Default::default() }) as Box<Element> )
	}
	fn name(&self) -> String {
		format!("ElementLATCH{{{}}}", self.vals.len())
	}
	fn get_outputs(&self, _: usize) -> usize {
		1+self.vals.len()
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let enable = inlines[0];
		let reset = inlines[1];
		let in_ofs = 2;
		
		if enable
		{
			outlines[0] = true;
			
			if reset {
				for v in self.vals.iter_mut() {
					*v = false;
				}
			}
			else {
				for (i,v) in self.vals.iter_mut().enumerate()
				{
					if inlines[in_ofs+i]
					{
						*v = true;
					}
				}
			}
			
			for (i,v) in self.vals.iter().enumerate() {
				outlines[1+i] = *v;
			}
		}
	}
}

#[derive(Clone,Default)]
struct ElementJkFlipFlop
{
	last_clk: bool,
	state: bool,
}
impl Element for ElementJkFlipFlop
{
	fn new(_params: &[u64], n_inputs: usize) -> NewEleResult
	{
		if n_inputs != 3 {
			return Err("".into());
		}
		Ok(Box::new(Self::default()))
	}
	fn name(&self) -> String {
		"JkFlipFlop".into()
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		2
	}

	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let clk = inlines[0];
		let j = inlines[1];
		let k = inlines[2];
		//println!("Jk: C={} J={} K={} S={}", clk, j, k, clk);
		if clk != self.last_clk && !clk
		{
			//println!("JkFalling: {:p} {} J={} K={}", self, self.state, j, k);
			// Falling edge: Update state
			self.state = if j && k {
					!self.state
				}
				else if j {
					true
				}
				else if k {
					false
				}
				else {
					self.state
				};
		}
		self.last_clk = clk;

		outlines[0] = self.state;
		outlines[1] = !self.state;
	}
}

#[derive(Clone)]
struct ElementMUX
{
	bits: u8,
	bussize: u8,
}
impl Element for ElementMUX
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		if params.len() > 2 { return Err(format!("Too many parameters, expected at most 2")); }
		let bits    = if params.len() >= 1 { params[0] as u8 } else { 1 };
		let bussize = if params.len() >= 2 { params[1] as u8 } else { 1 };

		if bits == 0 || bits > 10 {
			return Err(format!("Bit count sanity check failure, must be 1--10 inclusive (got {})", bits));
		}
		if bussize == 0 {
			return Err(format!("Bus size sanity check failure, must >0 (got {})", bussize));
		}
		
		let exp_inputs: usize =  1 + bits as usize + (1 << bits as usize)*(bussize as usize);
		if n_inputs != exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		Ok( Box::new(ElementMUX{bits: bits, bussize: bussize}) as Box<Element> )
	}
	fn name(&self) -> String {
		format!("ElementMUX{{{},{}}}", self.bits,self.bussize)
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		self.bussize as usize
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let enable = inlines[0];
		let index = read_uint(inlines, 1, self.bits) as usize;
		let ofs = 1 + (self.bits as usize) + index * (self.bussize as usize);;
		if enable
		{
			for i in 0 .. self.bussize as usize
			{
				outlines[i] |= inlines[ofs + i];
			}
		}
	}
}

#[derive(Clone)]
struct ElementDEMUX
{
	bits: u8
}
impl Element for ElementDEMUX
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		let (bits, bussize) = match params.len()
				{
				0 => return Err(format!("Not enough parameters, need at least 1")),
				1 => (params[0] as u8, 1),
				2 => (params[0] as u8, params[1] as u8),
				_ => return Err(format!("Too many parameters, at most 2")),
				};
		if bits == 0 || bits > 10 {
			return Err(format!("Bit count sanity check failure, must be 1--10 inclusive (got {})", bits));
		}
		let exp_inputs = 1 + bits + bussize;
		if n_inputs < exp_inputs as usize {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		Ok( Box::new(ElementDEMUX{bits: bits}) as Box<Element> )
	}
	fn name(&self) -> String {
		format!("ElementDEMUX{{{}}}", self.bits)
	}
	fn get_outputs(&self, n_inputs: usize) -> usize {
		let bussize = n_inputs - 1 - self.bits as usize;
		bussize << self.bits as usize
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let enable = inlines[0];
		let index = read_uint(inlines, 1, self.bits) as usize;
		let ofs = 1 + self.bits as usize;
		let bussize = inlines.len() - ofs;
		if enable
		{
			for i in 0 .. bussize
			{
				outlines[index*bussize + i] |= inlines[ofs+i];
			}
		}
	}
}

#[derive(Clone)]
struct ElementSEQUENCER
{
	count: u16,
	position: u16,
}
impl Element for ElementSEQUENCER
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		if params.len() != 1 {
			return Err(format!("Invalid parameter count, expected 1, got {}", params.len()));
		}
		let count = params[0] as u16;
		let exp_inputs = 3;
		if n_inputs != exp_inputs {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		return Ok( Box::new(ElementSEQUENCER { count: count, position: 0 }) as Box<Element> );
	}
	fn name(&self) -> String {
		return format!("ElementSEQUENCER{{{}}}", self.count);
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		return self.count as usize;
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
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
			
			outlines[self.position as usize] = true;
		}
	}
}

#[allow(non_camel_case_types)]
#[derive(Default)]
#[derive(Clone)]
struct ElementMEMORY_DRAM
{
	wordsize: u8,
	addrbits: u8,
	data: Vec<u32>,
}
impl Element for ElementMEMORY_DRAM
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		if params.len() != 2 {
			return Err(format!("Invalid parameter count, expected 2, got {}", params.len()));
		}
		let wordsize = params[0] as u8;
		if wordsize == 0 || wordsize > 64 {
			return Err(format!("Word size sanity check failure, must be 0--64 (got {})", wordsize));
		}
		let addrbits = params[1] as u8;
		if addrbits == 0 || addrbits > 20 {	// allows up to 8MiB
			return Err(format!("Address bit sanity check failure, must be 1--14 (got {})", addrbits));
		}
		let exp_inputs = 1 + addrbits + 1 + 2*wordsize;
		if n_inputs != exp_inputs as usize {
			return Err(format!("Incorrect input count, expected {}, got {}", exp_inputs, n_inputs));
		}
		
		Ok( Box::new(ElementMEMORY_DRAM {
			wordsize: wordsize,
			addrbits: addrbits,
			..Default::default()
			}) as Box<Element> )
	}
	fn name(&self) -> String {
		format!("ElementMEMORY_DRAM{{{},{}}}", self.wordsize, self.addrbits)
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		1 + 1 << self.wordsize as usize
	}
	
	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone()) as Box<Element>
	}

	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		if self.data.len() == 0 {
			self.data = ::from_elem( 1 << self.addrbits * self.wordsize / 32, 0 );
		}
		let enable = inlines[0];
		let wordnum = read_uint(inlines, 1, self.addrbits) as usize;
		let write = inlines[1 + self.addrbits as usize];
		let writemask = read_uint(inlines, (1+self.addrbits+1) as usize, self.wordsize);
		let writeval = read_uint(inlines, (1+self.addrbits+1+self.wordsize) as usize, self.wordsize);
		
		assert!( self.wordsize == 32 );
		if enable
		{
			if write
			{
				let val = &mut self.data[wordnum];
				*val &= !writemask as u32;
				*val |= writeval as u32;
			}
			write_uint(outlines, 1, self.wordsize, self.data[wordnum] as u64);
		}
	}
}

#[derive(Clone)]
struct ElementROM
{
	file_index: usize,
	wordsize: usize,
	romdata: Option<Rc<Vec<u64>>>,
}
impl Element for ElementROM
{
	fn new(params: &[u64], n_inputs: usize) -> NewEleResult
	{
		//let addr_size = n_inputs - 1;

		let mut params_it = params.iter().cloned();
		let file_index = params_it.next().ok_or_else(|| format!("No ROM index passed"))? as usize;
		// TODO: How can this easily get the ROM data?
		// - At this stage, will the data be known?
		let wordsize = params_it.next().unwrap_or(8) as usize;
		Ok( Box::new(ElementROM {
			file_index,
			wordsize,
			romdata: None,
			}) as Box<Element> )
	}
	fn finalise(&mut self, unit: &::cct_mesh::Unit) {
		self.romdata = Some( unit.get_rom(self.file_index) );
	}
	fn name(&self) -> String
	{
		format!("Element_ROM{{{}, {}}}", self.wordsize, self.file_index)
	}
	fn get_outputs(&self, _n_inputs: usize) -> usize {
		self.wordsize
	}

	fn dup(&self) -> Box<Element+'static> {
		Box::new(self.clone())
	}
	fn update(&mut self, outlines: &mut [bool], inlines: &[bool])
	{
		let romdata = self.romdata.as_ref().unwrap();

		// If the ROM isn't located, load it now?
		// - Better idea: have a function used to finalise an element after unit is complete
		if inlines[0]
		{
			// Decode address (MSB first)
			let addr = ::simulator::decode_u64_be(&inlines[1..]);
			// Fetch ROM data
			let d = romdata.get(addr as usize).cloned().unwrap_or(0);
			//println!("{} {} = {:x}", self.file_index, addr, d);
			// And output
			for (i,out) in outlines.iter_mut().enumerate()
			{
				*out = (d >> i) & 1 != 0;
			}
		}
	}
}


// vim: ft=rust
