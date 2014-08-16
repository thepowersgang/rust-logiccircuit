//
//
//

pub struct Engine<'a>
{
	mesh: &'a mut ::cct_mesh::flat::Mesh,
	curstate: Vec<bool>,
	newstate: Vec<bool>,
}

impl<'a> Engine<'a>
{
	pub fn new<'a>(mesh: &'a mut ::cct_mesh::flat::Mesh) -> Engine<'a>
	{
		Engine {
			curstate: Vec::from_elem(mesh.n_nodes, false),
			newstate: Vec::from_elem(mesh.n_nodes, false),
			mesh: mesh,
		}
	}
	
	pub fn tick(&mut self)
	{
		for ele in self.mesh.elements.mut_iter()
		{
			// Obtain inputs
			let inputs = {
				let mut v = Vec::with_capacity( ele.inputs.len() );
				for i in ele.inputs.iter() {
					v.push( match *i {
						::cct_mesh::flat::NodeOne => true,
						::cct_mesh::flat::NodeZero => false,
						::cct_mesh::flat::NodeId(id) => self.curstate[id],
						});
				}
				v
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
	
	pub fn check_breakpoints(&self) -> bool
	{
		//for bp in self.mesh.breakpoints.iter()
		//{
		//}
		return false;
	}
	
	pub fn show_display(&self)
	{
	}
}

// vim: ft=rust