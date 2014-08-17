
#[deriving(Copy)]
#[deriving(Clone)]
pub enum NodeRef
{
	NodeZero,
	NodeOne,
	NodeId(uint),
}

/// Flattened mesh items
#[deriving(Clone)]
pub struct ElementInst
{
	pub inst: Box<::elements::Element>,
	pub inputs: Vec<NodeRef>,
	pub outputs: Vec<NodeRef>,
}

#[deriving(Default)]
#[deriving(Clone)]
pub struct Node
{
	pub names: Vec<String>
}

// Represents a flattened (executable) mesh
#[deriving(Clone)]
pub struct Mesh
{
	pub n_nodes: uint,
	//pub nodes: Vec<Node>,	// Aka LinkValues
	pub elements: Vec<ElementInst>,
	pub inputs: Vec<NodeRef>,
	pub outputs: Vec<NodeRef>,
	
	pub breakpoints: Vec<Breakpoint>,
	pub dispitems: Vec<Display>,
}

pub struct Test
{
	unit: ::std::rc::Rc<Mesh>,
	exec_limit:	uint,
	completion: Vec<NodeRef>,
	assertions: Vec<TestAssert>,
}

pub struct TestAssert
{
	pub line: uint,
	pub conditions: Vec<NodeRef>,
	pub values: Vec<NodeRef>,
	pub expected: Vec<NodeRef>,
}

#[deriving(Clone)]
pub struct Display
{
	pub condition: Vec<NodeRef>,
	pub text: String,
	pub values: Vec<NodeRef>,
}

#[deriving(Clone)]
pub struct Breakpoint
{
	name: String,
	pub conds: Vec<NodeRef>,
}

impl Mesh
{
	//pub fn new( nodes: Vec<Node>, elements: Vec<ElementInst>, inputs: &super::LinkList, outputs: &super::LinkList ) -> Mesh
	pub fn new( n_nodes: uint, n_eles: uint, n_bps: uint, n_disp: uint, inputs: &super::LinkList, outputs: &super::LinkList ) -> Mesh
	{
		Mesh {
			n_nodes: n_nodes,
			elements: Vec::with_capacity(n_eles),
			inputs:  linklist_to_noderefs(inputs),
			outputs: linklist_to_noderefs(outputs),
			
			breakpoints: Vec::with_capacity(n_bps),
			dispitems: Vec::with_capacity(n_disp),
		}
	}
	
	pub fn push_ele(&mut self, ele: ElementInst) {
		self.elements.push( ele );
	}
	pub fn push_disp(&mut self, disp: Display) {
		info!("push_disp: '{}'", disp.text);
		self.dispitems.push( disp );
	}
	pub fn push_breakpoint(&mut self, bp: Breakpoint) {
		self.breakpoints.push( bp );
	}
	
	pub fn merge(&mut self, other: &Mesh, aliases: &Vec<Option<NodeRef>>)
	{
		for ele in other.elements.iter()
		{
			let ele_inputs  = noderefs_aliased(&ele.inputs,  aliases);
			let ele_outputs = noderefs_aliased(&ele.outputs, aliases);
			let inst = ElementInst {
				inst: ele.inst.dup(),
				inputs:  ele_inputs,
				outputs: ele_outputs,
				};
			self.push_ele( inst );
		}
		
		for di in other.dispitems.iter()
		{
			self.push_disp( Display {
				condition: noderefs_aliased(&di.condition, aliases),
				text: di.text.clone(),
				values: noderefs_aliased(&di.values, aliases),
				} );
		}
	}
}

impl Test
{
	pub fn new(flat: ::std::rc::Rc<Mesh>, exec_limit: uint, completion: Vec<NodeRef>, assertions: Vec<TestAssert>) -> Test {
		Test {
			exec_limit: exec_limit,
			unit: flat,
			completion: completion,
			assertions: assertions,
			}
	}
	
	pub fn exec_limit(&self) -> uint { self.exec_limit }
	pub fn get_mesh(&self) -> &Mesh { self.unit.deref() }
	pub fn get_completion(&self) -> &Vec<NodeRef> { &self.completion }
	pub fn iter_asserts(&self) -> ::core::slice::Items<::cct_mesh::flat::TestAssert>
	{
		self.assertions.iter()
	}
}

impl TestAssert
{
	pub fn new(line: uint, conds: Vec<NodeRef>, have: Vec<NodeRef>, exp: Vec<NodeRef>) -> TestAssert {
		TestAssert {
			line: line,
			conditions: conds,
			values: have,
			expected: exp,
		}
	}
}

impl Display
{
	pub fn new(text: String, conds: Vec<NodeRef>, values: Vec<NodeRef>) -> Display {
		Display {
			condition: conds,
			text: text,
			values: values,
		}
	}
}

impl Breakpoint
{
	pub fn new(name: String, conds: Vec<NodeRef>) -> Breakpoint {
		Breakpoint {
			name: name,
			conds: conds,
		}
	}
}

/// @brief Convert a LinkList into node references
pub fn linklist_to_noderefs(links: &super::LinkList) -> Vec<NodeRef>
{
	let mut rv = Vec::with_capacity(links.len());
	for link in links.iter()
	{
		let linkref = link.borrow();
		//debug!("Link '{}'", linkref.name);
		let nr = match linkref.name.as_slice() {
			"=0" => NodeZero,
			"=1" => NodeOne,
			_ => NodeId( link.borrow().get_alias().unwrap() ),
			};
		rv.push( nr );
	}
	return rv;
}

fn noderefs_aliased(innodes: &Vec<NodeRef>, aliases: &Vec<Option<NodeRef>>) -> Vec<NodeRef>
{
	let mut rv = Vec::with_capacity(innodes.len());
	for (i,node) in innodes.iter().enumerate()
	{
		let nr = match *node
			{
			NodeId(id) => {
				if id >= aliases.len() {
					fail!("BUG - Node {} (idx {}) not in aliases table (only {} entries)",
						id, i, aliases.len());
				}
				if aliases[id].is_none() {
					fail!("BUG - Node {} (idx {}) was not aliased", id, i);
				}
				aliases[id].unwrap()
				},
			NodeZero => NodeZero,
			NodeOne  => NodeOne,
			};
		rv.push( nr );
	}
	return rv;
}

// vim: ft=rust
