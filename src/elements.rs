//
//
//

pub trait Element
{
	fn update(&mut self, instate: &Vec<bool>, inlines: &Vec<uint>, outstate: &mut Vec<bool>, outlines: &Vec<uint>);
}

struct TypeDummy;

macro_rules! get_or( ($vec:expr, $idx:expr, $def:expr) => ({let _i=$idx; let _v=$vec; (if _i < _v.len(){_v[_i]}else{$def})}) )

pub fn create(name: &String, params: &Vec<uint>, n_inputs: uint) -> Box<Element>
{
	match name.as_slice()
	{
	"DELAY" => box TypeDelay::new(params, n_inputs) as Box<Element>,
	_ => box TypeDummy as Box<Element>
	}
}

impl Element for TypeDummy
{
	fn update(&mut self, instate: &Vec<bool>, inlines: &Vec<uint>, outstate: &mut Vec<bool>, outlines: &Vec<uint>)
	{
		let mut val = false;
		
		for i in inlines.iter() {
			val |= instate[*i];
		}
		for i in outlines.iter() {
			*(outstate.get_mut(*i)) = val;
		}
	}
}

struct TypeDelay
{
	count: uint,
	idx: uint,
	vals: Vec<bool>,
}
impl TypeDelay
{
	pub fn new(params: &Vec<uint>, n_inputs: uint) -> TypeDelay
	{
		let count = get_or!(params, 0, 1);
		TypeDelay {
			count: count,
			idx: 0,
			vals: Vec::<bool>::from_fn(count*n_inputs, |_| false),
		}
	}
}
impl Element for TypeDelay
{
	fn update(&mut self, instate: &Vec<bool>, inlines: &Vec<uint>, outstate: &mut Vec<bool>, outlines: &Vec<uint>)
	{
		assert!( inlines.len() == outlines.len() );
		if self.count == 0
		{
			for i in range(0, inlines.len())
			{
				*(outstate.get_mut(outlines[i])) = instate[inlines[i]];
			}
		}
		else
		{
			let baseidx = self.idx * inlines.len();
			for i in range(0, inlines.len()) {
				*(outstate.get_mut(outlines[i])) = self.vals[ baseidx + i ];
				*(self.vals.get_mut(baseidx + i)) = instate[inlines[i]];
			}
			
			self.idx += 1;
			if self.idx == self.count {
				self.idx = 0;
			}
		}
	}
}

// vim: ft=rust
