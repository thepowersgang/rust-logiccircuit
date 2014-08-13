//
//
//

use std::string::String;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::default::Default;
use std::collections::Deque;

enum List<T> {
    Cons(T, Box<List<T>>),
    Nil
}

#[deriving(Default)]
struct Link
{
	name: String,
	reflink: Option<LinkWRef>,
	aliases: ::collections::DList<LinkWRef>,
	aliased: Option<uint>,	// Used during node counting
}

pub type LinkRef = Rc<RefCell<Link>>;
pub type LinkWRef = Weak<RefCell<Link>>;
pub type LinkList = Vec<LinkRef>;

#[deriving(Default)]
pub struct Element
{
	name: String,
	params: Vec<uint>,
	inputs: LinkList,
	outputs: LinkList,
}

#[deriving(Default)]
struct VisGroup
{
	name: String,
	elements: List<Element>
}

struct Breakpoint
{
	condition: LinkList,
	name: String,
}

struct DisplayItem
{
	condition: LinkList,
	text: String,
	values: LinkList,
}

struct UnitRef
{
	name: String,
	inputs: LinkList,
	outputs: LinkList,
}

#[deriving(Default)]
pub struct Unit
{
	name: String,
	inputs: LinkList,
	outputs: LinkList,
	
	link_zero: LinkRef,
	link_one: LinkRef,
	anon_links: ::collections::DList<LinkRef>,
	links:  ::collections::TreeMap<String,LinkRef>,	// Definitive list of links
	groups: ::collections::TreeMap<String,LinkList>,
	
	elements: ::collections::DList<Element>,
	subunits: ::collections::DList<UnitRef>,
	
	breakpoints: ::collections::DList<Breakpoint>,
	disp_items: ::collections::DList<DisplayItem>,
	
	visgroups: List<VisGroup>,
}

struct TestAssert
{
	conditions: LinkList,
	values: LinkList,
	expected: LinkList,
}

#[deriving(Default)]
pub struct Test
{
	exec_limit: uint,
	completion: LinkList,
	unit: Unit,
	assertions: ::collections::DList<TestAssert>,
}

#[deriving(Default)]
pub struct Root
{
	rootunit: Unit,
	units: ::collections::TreeMap<String,Unit>,
	tests: ::collections::TreeMap<String,Test>,
}

enum NodeRef
{
	NodeZero,
	NodeOne,
	NodeId(uint),
}
// Represents a flattened (executable) mesh
pub struct Mesh
{
	nodes: Vec<Node>,	// Aka LinkValues
	inputs: Vec<NodeRef>,
	outputs: Vec<NodeRef>,
	elements: Vec<ElementInst>,
}

/// Flattened mesh items
pub struct ElementInst
{
	inst: Box<::elements::Element>,
	inputs: Vec<NodeRef>,
	outputs: Vec<NodeRef>,
}
#[deriving(Default)]
#[deriving(Clone)]
pub struct Node
{
	names: Vec<String>
}

impl<T> Default for List<T>
{
	fn default() -> List<T> {
		Nil
	}
}

impl Default for RefCell<Link> {
	fn default() -> RefCell<Link> {
		RefCell::new(Link {..Default::default()})
	}
}
impl ::core::cmp::Ord for Link {
	fn cmp(&self, x: &Link) -> Ordering { self.name.cmp(&x.name) }
}
impl ::core::cmp::PartialEq for Link {
	fn eq(&self, x: &Link) -> bool { self.name == x.name }
}
impl ::core::cmp::PartialOrd for Link {
	fn partial_cmp(&self, x: &Link) -> Option<Ordering> { Some(self.cmp(x)) }
}
impl ::core::cmp::Eq for Link
{
}
impl Link
{
	pub fn bind(&mut self, other: &Rc<RefCell<Link>>) {
		self.reflink = Some(other.downgrade());
		//debug!("Bound link {} to {}'s value", self.name, other.name);
	}
	pub fn tag(&mut self, value: uint) -> bool {
		//debug!("Tagging '{}' to {}", self.name, value);
		if self.aliased != None {
			fail!("Link '{}' already aliased to #{}", self.name, self.aliased.unwrap());
		}
		assert!( self.aliased == None );
		match self.reflink {
		Some(_) => {
			false
			},
		None => {
			self.aliased = Some(value);
			true
			}
		}
	}
	pub fn tag_from_ref(&mut self)
	{
		if self.aliased == None
		{
			match self.reflink
			{
			Some(ref other) => {
				let other_rc = other.upgrade().unwrap();
				let other_link = other_rc.borrow();
				self.aliased = other_link.aliased;
				debug!("Indirect tag of '{}' from '{}' ({})",
					self.name, other_link.name, self.aliased);
				assert!(self.aliased != None);
				},
			None => {}
			}
		}
	}
	pub fn get_alias(&self) -> Option<uint> {
		return self.aliased;
	}
}
impl ::core::fmt::Show for ::core::cell::RefCell<::cct_mesh::Link>
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		write!(f, "{}", self.borrow().name)
	}
}

impl Element
{
	pub fn new(name: String, args: Vec<u64>, inputs: LinkList) -> Element {
		let params = Vec::from_fn(args.len(), |i| args[i] as uint);
		Element { name: name, params: params, inputs: inputs, ..Default::default() }
	}
	
	pub fn set_outputs(&mut self, outputs: LinkList) -> bool {
		// TODO: Check counts and error
		self.outputs = outputs;
		return true;
	}
	pub fn anon_outputs(&mut self, unit: &mut Unit) -> LinkList {
		self.outputs = unit.make_anon_links(1);
		return self.outputs.clone();
	}
}

impl Test
{
	pub fn get_unit(&mut self) -> &mut Unit {
		&mut self.unit
	}
	
	pub fn set_completion(&mut self, conds: LinkList) -> bool {
		if self.completion.len() > 0 {
			return true;
		}
		else {
			self.completion = conds;
			return false;
		}
	}
	pub fn add_assert(&mut self, conds: LinkList, vals: LinkList, exp: LinkList) {
		self.assertions.push(TestAssert{
			conditions: conds,
			values: vals,
			expected: exp,
			});
	}
}

macro_rules! exp( ($val:expr, $e:pat => $res:expr) => (match $val { $e=>$res, _=>fail!("exp!")}))
impl Unit
{
	pub fn get_constant(&mut self, is_one: bool) -> LinkRef {
		if is_one {
			self.link_one.clone()
		}
		else {
			self.link_zero.clone()
		}
	}
	pub fn get_link(&mut self, name: &String) -> LinkRef {
		match self.links.find(name)
		{
		Some(x) => return x.clone(),
		None => ()
		}
		
		let val = Rc::new(RefCell::new( Link { name: name.clone(), .. Default::default() } ));
		self.links.insert(name.clone(), val.clone());
		val
	}
	fn make_anon_link(&mut self) -> LinkRef {
		self.anon_links.push(Rc::new( RefCell::new( Link {
			name:"".to_string(),
			..Default::default()
			} )
			));
		return self.anon_links.back().unwrap().clone();
	}
	pub fn make_anon_links(&mut self, count: uint) -> LinkList {
		let mut ret = Vec::with_capacity(count);
		for i in range(0,count) {
			ret.push( self.make_anon_link() );
		}
		return ret;
	}
	
	pub fn make_group(&mut self, name: &String, size: uint) {
		let mut val = Vec::with_capacity(size);
		for i in range(0,size) {
			val.push( self.get_link(&format!("{}[{:2u}]", name, i)) );
		}
		self.groups.insert(name.clone(), val);
		debug!("make_group: {} created with {} items", *name, size);
	}
	pub fn get_group(&self, name: &String) -> Option<&LinkList> {
		return self.groups.find(name)
	}
	
	pub fn set_input(&mut self, inputs: LinkList) -> bool {
		if self.inputs.len() > 0 {
			return true;
		}
		else {
			self.inputs = inputs;
			return false;
		}
	}
	
	pub fn set_output(&mut self, outputs: LinkList) -> bool {
		if self.outputs.len() > 0 {
			return true;
		}
		else {
			self.outputs = outputs;
			return false;
		}
	}
	
	pub fn append_element(&mut self, meshroot: &Root, name: String, params: Vec<u64>, inputs: LinkList, outputs: Option<LinkList>) -> LinkList
	{
		debug!("append_element('{}', {}, in={}, out={})",
			name, params, inputs, outputs);
		match meshroot.get_unit(&name)
		{
		// Referencing a sub-unit
		Some(unit) => {
			let out = match outputs { None => self.make_anon_links(unit.outputs.len()), Some(o) => o };
			if out.len() != unit.outputs.len() {
				fail!("Output mismatch for unit '{}', got {} expected {}",
					name, out.len(), unit.outputs.len());
			}
			if inputs.len() != unit.inputs.len() {
				fail!("Input mismatch for unit '{}', got {} expected {}",
					name, inputs.len(), unit.inputs.len());
			}
			let r = UnitRef {
				name: name,
				inputs: inputs,
				outputs: out.clone(),
				};
			self.subunits.push(r);
			out
			},
		None => {
			let mut ele = Element::new(name, params, inputs);
			
			let rv = if outputs == None {
					ele.anon_outputs(self)
				}
				else {
					let o = outputs.unwrap();
					ele.set_outputs(o.clone());
					o
				};
			self.elements.push( ele );
			rv
			}
		}
	}
	pub fn append_display(&mut self, cond: LinkList, text: String, values: LinkList) {
		self.disp_items.push( DisplayItem {
			condition: cond,
			text: text,
			values: values,
			});
	}
	pub fn append_breakpoint(&mut self, name: String, cond: LinkList) {
		self.breakpoints.push( Breakpoint {
			condition: cond,
			name: name,
			});
	}
	
	pub fn flatten(&self, root: &Root) -> Mesh
	{
		debug!("Flattening unit '{}'", self.name);
		let subunits = self.flatten_subunits(root);
		
		let mut n_eles = 0u;
		let mut n_links = 0u;
		
		n_eles += self.elements.len();
		
		// Count nodes
		debug!("Tagging anon links (Unit '{}')", self.name);
		for link in self.anon_links.iter() {
			if link.borrow_mut().tag(n_links) { n_links += 1; }
		}
		debug!("Tagging named links");
		for (_,link) in self.links.iter() {
			if link.borrow_mut().tag(n_links) { n_links += 1; }
		}
		// > Groups are stored in named links
		for link in self.anon_links.iter() {
			link.borrow_mut().tag_from_ref();
		}
		for (_,link) in self.links.iter() {
			link.borrow_mut().tag_from_ref();
		}
		let n_local_links = n_links;
		
		debug!("n_eles = {}, n_links = {}", n_eles, n_links);
		
		// Count elements and nodes from sub units
		for subu in subunits.iter()
		{
			n_eles += subu.elements.len();
			// 1. Assert that no input connects directly to output
			// 2. Add link count, subtract nInput and nOutput
			n_links += subu.nodes.len() - subu.inputs.len() - subu.outputs.len();
		}
		debug!("w/ subunits n_eles = {}, n_links = {}", n_eles, n_links);
		
		// Add names to nodes
		let mut nodes = Vec::<Node>::from_elem(n_links, Node{..Default::default()});
		for (i,link) in self.anon_links.iter().enumerate() {
			let id = link.borrow().get_alias().unwrap();
			nodes.get_mut(id).names.push( format!("#{}", i) );
		}
		for (name,link) in self.links.iter() {
			let id = link.borrow().get_alias().unwrap();
			nodes.get_mut(id).names.push( name.clone() );
		}
		debug!("- Links added");
		
		
		// Add elements
		let mut elements = Vec::<ElementInst>::with_capacity(n_eles);
		for ele in self.elements.iter()
		{
			let inst = ElementInst {
				inst: ::elements::create(&ele.name, &ele.params, ele.inputs.len()),
				inputs:  self.linklist_to_noderefs(&ele.inputs),
				outputs: self.linklist_to_noderefs(&ele.outputs),
				};
			elements.push( inst );
		}
		debug!("- Elements added");
		
		// Populate from sub-units
		let mut bind_node_idx = n_local_links;
		for (i,subu) in self.subunits.iter().enumerate()
		{
			let flattened = &subunits[i];
			let inputs = self.linklist_to_noderefs( &subu.inputs );
			let outputs = self.linklist_to_noderefs( &subu.outputs );
			let mut aliases = Vec::<Option<uint>>::with_capacity( flattened.nodes.len() );
			
			debug!("> Import subunit {}", subu.name);
			assert_eq!( flattened.inputs.len(),  inputs.len()  );
			assert_eq!( flattened.outputs.len(), outputs.len() );
			for (j,noderef) in flattened.inputs.iter().enumerate()
			{
				let inner_node = exp!(*noderef, NodeId(id) => id);
				let outer_node = exp!(inputs[j], NodeId(id) => id);
				*(aliases.get_mut(inner_node)) = Some( outer_node );
			}
			for (j,noderef) in flattened.outputs.iter().enumerate()
			{
				*(aliases.get_mut(exp!(*noderef, NodeId(id) => id))) = Some( exp!(outputs[j], NodeId(id) => id) );
			}
			
			// Iterate
			for alias in aliases.mut_iter()
			{
				if *alias == None {
					*alias = Some(bind_node_idx);
					bind_node_idx += 1;
				}
			}
		}
		
		// Unit inputs/outputs
		let mut inputs  = Vec::<NodeRef>::with_capacity( self.inputs.len() );
		let mut outputs = Vec::<NodeRef>::with_capacity( self.outputs.len() );
		for link in self.inputs.iter() {
			inputs.push( NodeId( link.borrow().get_alias().unwrap() ) );
		}
		for link in self.outputs.iter() {
			outputs.push( NodeId( link.borrow().get_alias().unwrap() ) );
		}
		
		debug!("'{}' flattened: {} nodes, {} elements", self.name, nodes.len(), elements.len());
		return Mesh {
			nodes: nodes,
			inputs: inputs,
			outputs: outputs,
			elements: elements,
		};
	}
	fn flatten_subunits(&self, root: &Root) -> Vec<Mesh> {
		let mut ret = Vec::with_capacity(self.subunits.len());
		for unitref in self.subunits.iter() {
			ret.push( root.units.find(&unitref.name).unwrap().flatten(root) );
		}
		return ret;
	}
	
	/// @brief Convert a LinkList into node references
	fn linklist_to_noderefs(&self, links: &LinkList) -> Vec<NodeRef>
	{
		let mut rv = Vec::with_capacity(links.len());
		for link in links.iter()
		{
			let nr = if *link == self.link_zero {
					NodeZero
				} else if *link == self.link_one {
					NodeOne
				} else {
					NodeId( link.borrow().get_alias().unwrap() )
				};
			rv.push( nr );
		}
		return rv;
	}
}

impl Root
{
	pub fn new() -> Root {
		Root { ..Default::default() }
	}
	
	pub fn get_root_unit(&mut self) -> &mut Unit {
		return &mut self.rootunit;
	}
	pub fn add_unit(&mut self, name: &String) -> Option<&mut Unit> {
		match self.units.find_mut(name)
		{
		Some(_) => return None,
		None => ()
		}
		let val = Unit { name: name.clone(), ..Default::default() };
		self.units.insert(name.clone(), val);
		return self.units.find_mut(name);
	}
	pub fn get_unit(&self, name: &String) -> Option<&Unit> {
		return self.units.find(name);
	}
	pub fn add_test(&mut self, name: &String, exec_limit: uint) -> Option<&mut Test> {
		match self.tests.find_mut(name)
		{
		Some(_) => return None,
		None => ()
		}
		
		let val = Test { exec_limit: exec_limit, ..Default::default() };
		self.tests.insert(name.clone(), val);
		return self.tests.find_mut(name);
	}
	
	pub fn flatten_root(&self) -> Mesh {
		self.rootunit.flatten(self)
	}
}

// vim: ft=rust

