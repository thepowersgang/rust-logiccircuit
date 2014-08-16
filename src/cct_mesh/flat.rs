
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
}

pub struct Test
{
	exec_limit:	uint,
	completion: Vec<NodeRef>,
	assertions: Vec<TestAssert>,
}

struct TestAssert
{
	line:	uint,
	conditions: Vec<NodeRef>,
	values: Vec<NodeRef>,
	expected: Vec<NodeRef>,
}

pub struct Display
{
	condition: Vec<NodeRef>,
	values: Vec<NodeRef>,
	text: String,
}

pub struct Breakpoint
{
	name: String,
	conds: Vec<NodeRef>,
}

impl Mesh
{
	//pub fn new( nodes: Vec<Node>, elements: Vec<ElementInst>, inputs: &super::LinkList, outputs: &super::LinkList ) -> Mesh
	pub fn new( n_nodes: uint, elements: Vec<ElementInst>, inputs: &super::LinkList, outputs: &super::LinkList ) -> Mesh
	{
		Mesh {
			n_nodes: n_nodes,
			elements: elements,
			inputs:  linklist_to_noderefs(inputs),
			outputs: linklist_to_noderefs(outputs),
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

// vim: ft=rust
