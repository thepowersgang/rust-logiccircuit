//
//
//

pub struct Engine<'a>
{
	mesh: &'a ::cct_mesh::flat::Mesh,
	elements: Vec<::cct_mesh::flat::ElementInst>,
	curstate: Vec<bool>,
	newstate: Vec<bool>,
}

macro_rules! getval( ($state:expr, $nr:expr) => ( match $nr {
	::cct_mesh::flat::NodeOne => true,
	::cct_mesh::flat::NodeZero => false,
	::cct_mesh::flat::NodeId(id) => $state[id],
	}))

impl<'a> Engine<'a>
{
	pub fn new<'a>(mesh: &'a ::cct_mesh::flat::Mesh) -> Engine<'a>
	{
		Engine {
			mesh: mesh,
			elements: mesh.elements.iter().map(
				|e| e.clone()
				).collect(),
			curstate: Vec::from_elem(mesh.n_nodes, false),
			newstate: Vec::from_elem(mesh.n_nodes, false),
		}
	}
	
	pub fn tick(&mut self)
	{
		for ele in self.elements.mut_iter()
		{
			// Obtain inputs
			let inputs = {
				let mut inp = Vec::with_capacity(ele.inputs.len());
				for i in ele.inputs.iter() {
					inp.push( getval!(self.curstate, *i) );
				}
				inp
				};
			let mut outputs = Vec::from_elem(ele.outputs.len(), false);

			// Update
			ele.inst.update(&mut outputs, &inputs);
		
			// Save results
			for (i,line) in ele.outputs.iter().enumerate()
			{
				match *line
				{
				::cct_mesh::flat::NodeId(id) => {
					*(self.newstate.get_mut(id)) |= outputs[i]
					},
				_ => {
					},
				}
			}
		}
		::std::mem::swap( &mut self.curstate, &mut self.newstate );
	}
	
	/// @param logical_and - If true, perform a logical AND on the values, else do an OR
	pub fn are_set(&self, nodes: &Vec<::cct_mesh::flat::NodeRef>, logical_and: bool) -> bool
	{
		for node in nodes.iter()
		{
			// true, !true >> skip (logical AND and  high, need to check more)
			// true, !false >> return (logical OR, and high, return true)
			// false, !true >> return (logical AND, and low, return false)
			// false, !false >> skip (logical OR, and low, need to check more)
			if getval!(self.curstate, *node) != logical_and
			{
				return !logical_and;
			}
		}
		return logical_and;	// If no short-circuits happened, AND=true, OR=false
	}
	pub fn get_values(&self, nodes: &Vec<::cct_mesh::flat::NodeRef>) -> Vec<bool>
	{
		let mut rv = Vec::with_capacity(nodes.len());
		for i in nodes.iter() {
			rv.push( getval!(self.curstate, *i) );
		}
		rv
	}
	
	pub fn check_breakpoints(&self) -> bool
	{
		for bp in self.mesh.breakpoints.iter()
		{
			if self.are_set(&bp.conds, true) {
				return true;
			}
		}
		return false;
	}
	
	pub fn show_display(&self)
	{
	}
}

// vim: ft=rust
