//
//
//

use std::string::String;
use std::rc::Rc;
use std::rc::Weak;
use std::cell::RefCell;
use std::default::Default;

use cct_mesh::flat::NodeRef;
use cct_mesh::flat::NodeId;
use cct_mesh::flat::NodeOne;
use cct_mesh::flat::NodeZero;

pub mod flat;

macro_rules! chain( ($base:expr $(.. $next:expr)+) => ( $base $(.chain($next) )+ ) )
macro_rules! zip  ( ($base:expr $(, $next:expr)+) => ( $base $(.zip($next) )+ ) )

#[deriving(Default)]
struct Link
{
	name: String,
	reflink: Option<LinkWRef>,
	aliased: Option<uint>,	// Used during node counting
}

pub type LinkRef = Rc<RefCell<Link>>;
pub type LinkWRef = Weak<RefCell<Link>>;
pub type LinkList = Vec<LinkRef>;
type Flatmap = ::collections::TreeMap<String,Rc<flat::Mesh>>;

pub struct Element
{
	inst: Box<::elements::Element>,
	inputs: LinkList,
	outputs: LinkList,
}

//#[deriving(Default)]
//struct VisGroup
//{
//	name: String,
//	elements: ::collections::DList<Element>
//}

struct Breakpoint
{
	conds: LinkList,
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
	
	//visgroups: ::collections::DList<VisGroup>,
	
	flattened: Option<Rc<flat::Mesh>>,
}

struct TestAssert
{
	line: uint,
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
	
	flat_units: Flatmap,
	flat_tests: ::collections::TreeMap<String,flat::Test>,
}

impl Clone for Box<::elements::Element>
{
	fn clone(&self) -> Box<::elements::Element> {
		return self.dup();
	}
}

macro_rules! exp( ($val:expr, $e:pat => $res:expr) => (match $val { $e=>$res, _=>fail!("exp!")}))

impl Default for RefCell<Link> {
	fn default() -> RefCell<Link> {
		RefCell::new(Link {name: "<default>".to_string(), ..Default::default()})
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
		if self.aliased != None {
			fail!("Link '{}' already aliased to #{}", self.name, self.aliased.unwrap());
		}
		assert!( self.aliased == None );
		match self.reflink {
		Some(ref l) => {
			debug!("Link '{}' refers to '{}'", self.name, l.upgrade().unwrap().borrow().name);
			false
			},
		None => {
			debug!("Tagging '{}' to {}", self.name, value);
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
			Some(ref other_ref) => {
				let mut other = other_ref.clone();
				loop
				{
					let other_rc = other.upgrade().unwrap();
					let other_link = other_rc.borrow();
					if other_link.reflink.is_none()
					{
						self.aliased = other_link.aliased;
						debug!("Indirect tag of '{}' from '{}' ({})",
							self.name, other_link.name, self.aliased);
						assert!(self.aliased != None);
						break;
					}
					else
					{
						other = exp!(other_link.reflink, Some(ref x) => x.clone());
					}
				}
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
		let link = self.borrow();
		if link.name.as_slice() == "" {
			write!(f, "<anon>")
		}
		else {
			write!(f, "{}", self.borrow().name)
		}
	}
}

impl ::core::fmt::Show for NodeRef
{
	fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
		match *self
		{
		NodeOne => write!(f, "NodeOne"),
		NodeZero => write!(f, "NodeZero"),
		NodeId(id) => write!(f, "NodeId({})", id),
		}
	}
}

impl Default for Unit
{
	fn default() -> Unit {
		Unit {
			name: "".to_string(),
			inputs: Vec::new(),
			outputs: Vec::new(),
			
			link_zero: Unit::make_link( "=0".to_string() ),
			link_one: Unit::make_link( "=1".to_string() ),
			anon_links: ::collections::DList::new(),
			links:  ::collections::TreeMap::new(),	// Definitive list of links
			groups: ::collections::TreeMap::new(),
			
			elements: ::collections::DList::new(),
			subunits: ::collections::DList::new(),
			
			breakpoints: ::collections::DList::new(),
			disp_items: ::collections::DList::new(),
			//visgroups: ::collections::DList::new(),
			flattened: None,
		}
	}
}
impl Unit
{
	pub fn new(name: &String) -> Unit {
		Unit {
			name: name.clone(),
			link_zero: Unit::make_link( "=0".to_string() ),
			link_one:  Unit::make_link( "=1".to_string() ),
			..Default::default()
			}
	}
	fn make_link(name: String) -> LinkRef {
		Rc::new(RefCell::new( Link { name: name, .. Default::default() } ))
	}
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
		
		let val = Unit::make_link(name.clone());
		self.links.insert(name.clone(), val.clone());
		val
	}
	fn make_anon_link(&mut self) -> LinkRef {
		let link = Unit::make_link( format!("#{}", self.anon_links.len()) );
		self.anon_links.push( link.clone() );
		return link;
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
			let ele = match ::elements::create(&name, &params, inputs.len()) {
				Ok(e) => e,
				Err(msg) => fail!("Error in creating '{}' - {}", name, msg),
				};
			
			let out = match outputs { Some(o) => o, None => self.make_anon_links( ele.get_outputs(inputs.len()) ) };
			
			self.elements.push( Element {
				inst: ele,
				inputs: inputs,
				outputs: out.clone(),
				});
			out
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
			conds: cond,
			name: name,
			});
	}
	
	pub fn flatten(&mut self, pre_flattened: &Map<String,Rc<flat::Mesh>>) -> Rc<flat::Mesh>
	{
		debug!("Flattening unit '{}'", self.name);
		let subunits = self.flatten_subunits(pre_flattened);
		
		let mut n_eles = 0u;
		let mut n_links = 0u;
		
		n_eles += self.elements.len();
		
		// Count nodes
		debug!("Tagging anon links (Unit '{}')", self.name);
		// - Tag (and count) all links that arent not referencing another link
		for link in chain!( self.anon_links.iter() .. self.links.values() )
		{
			if link.borrow_mut().tag(n_links) { n_links += 1; }
		}
		// - And once all non-reference links are tagged, copy those tags to the reference links
		for link in chain!( self.anon_links.iter() .. self.links.values() )
		{
			link.borrow_mut().tag_from_ref();
		}
		let n_local_links = n_links;
		
		let mut n_bps  = self.breakpoints.len();
		let mut n_disp = self.disp_items.len();
		
		debug!("n_eles = {}, n_links = {}", n_eles, n_links);
		
		// Count elements and nodes from sub units
		for (subu_ref,subu) in zip!( self.subunits.iter(), subunits.iter() )
		{
			// Add element count
			n_eles += subu.elements.len();
			// Add node count, ignoring inputs and outputs
			n_links += subu.n_nodes;
			debug!("SubUnit #{} - {} in, {} out", subu_ref.name, subu.inputs.len(), subu.outputs.len());
			for (i,e) in chain!( subu.inputs.iter().enumerate() .. subu.outputs.iter().enumerate() ) {
				debug!("ext {}=#{}", i, *e);
				match *e { NodeId(_) => { n_links -= 1; }, _=>{}}
			}
			// Add breakpoints and display items
			n_bps += subu.breakpoints.len();
			n_disp += subu.dispitems.len();
		}
		info!("w/ subunits n_eles={}, n_links={}, n_bps={}, n_disp={}",
			n_eles, n_links, n_bps, n_disp);
		
		let mut ret = flat::Mesh::new(n_links, n_eles, n_bps, n_disp, &self.inputs, &self.outputs);

		/*
		// Add names to nodes
		// - TODO: This is never propagated, it uses too much memory to do so. Maybe remove it?
		let mut nodes = Vec::<flat::Node>::from_elem(n_links, flat::Node{..Default::default()});
		for (i,link) in self.anon_links.iter().enumerate()
		{
			let id = link.borrow().get_alias().unwrap();
			nodes.get_mut(id).names.push( format!("#{}", i) );
		}
		for (name,link) in self.links.iter()
		{
			let id = link.borrow().get_alias().unwrap();
			nodes.get_mut(id).names.push( name.clone() );
		}
		debug!("- Links added");
		*/
		
		// Add elements
		for ele in self.elements.iter()
		{
			debug!(" Element '{}'", ele.inst.name());
			let inst = flat::ElementInst {
				inst: ele.inst.dup(),
				inputs:  flat::linklist_to_noderefs(&ele.inputs),
				outputs: flat::linklist_to_noderefs(&ele.outputs),
				};
			ret.push_ele( inst );
		}
		info!("- Elements added");
		// Add breakpoints
		for bp in self.breakpoints.iter()
		{
			debug!("Breakpoint '{}'", bp.name);
			ret.push_breakpoint( flat::Breakpoint::new(
				bp.name.clone(),
				flat::linklist_to_noderefs(&bp.conds)
				) );
		}
		// Add display items
		for di in self.disp_items.iter()
		{
			debug!("Display item '{}'", di.text);
			ret.push_disp( flat::Display::new(
				di.text.clone(),
				flat::linklist_to_noderefs(&di.condition),
				flat::linklist_to_noderefs(&di.values),
				) );
		}
		
		// Populate from sub-units
		let mut bind_node_idx = n_local_links;
		for (i,subu) in self.subunits.iter().enumerate()
		{
			bind_node_idx += self.flatten_merge_subunit(&mut ret, subunits.get(i).deref(), subu, bind_node_idx);
		}
		assert!(bind_node_idx == n_links);
		assert!(ret.elements.len() == n_eles);
		
		info!("'{}' flattened: {} nodes, {} elements", self.name, n_links, n_eles);
		let rv = Rc::new( ret );
		self.flattened = Some( rv.clone() );
		return rv;
	}
	/// Merge a subunit into the flattened element list
	///
	/// @param elements	- Output element list (new elements appeneded)
	/// @param flattened	- Flattened sub-mesh
	/// @param subu 	- Subunit reference (used for outside node IDs)
	/// @param bind_node_idx	- ID to use for the next internal node
	/// @return Number of internal noes
	fn flatten_merge_subunit(&self, mesh: &mut flat::Mesh, flattened: &flat::Mesh, subu: &UnitRef, bind_node_idx: uint) -> uint
	{
		let inputs = flat::linklist_to_noderefs( &subu.inputs );
		let outputs = flat::linklist_to_noderefs( &subu.outputs );
		let mut aliases = Vec::<Option<NodeRef>>::from_elem( flattened.n_nodes, None );
		
		assert_eq!( flattened.inputs.len(),  inputs.len()  );
		assert_eq!( flattened.outputs.len(), outputs.len() );
		
		info!("> Import subunit {}", subu.name);
		// Iterate all external links, aliasing the inside node to the relevant outside node
		for (j,noderef) in flattened.inputs.iter().enumerate()
		{
			let inner_node = exp!(*noderef, NodeId(id) => id);
			*(aliases.get_mut(inner_node)) = Some(inputs[j]);
		}
		for (j,noderef) in flattened.outputs.iter().enumerate()
		{
			let inner_node = exp!(*noderef,   NodeId(id) => id);
			*(aliases.get_mut(inner_node)) = Some(outputs[j]);
		}
		
		// Count (and alias) all nodes that were not external links
		let mut unbound_nodes = 0u;
		for (j,alias) in aliases.mut_iter().enumerate()
		{
			if alias.is_none()
			{
				//debug!("Alias inner #{} to outer #NodeId({}) (int)", j, bind_node_idx);
				*alias = Some( NodeId(bind_node_idx + unbound_nodes) );
				unbound_nodes += 1;
			}
		}
		debug!("{} unbound nodes", unbound_nodes);
		
		/*
		// Append node names to link name list
		for (j,alias) in aliases.iter().enumerate()
		{
			match alias.unwrap()
			{
			NodeId(id) => {
				for name in flattened.nodes[j].names.iter() {
					nodes.get_mut(id).names.push( format!(">{}-{}:{}", subu.name, i, flattened.nodes[j].names) );
				}
				},
			_ => {
				}
			}
		}
		*/
		
		// Import elements
		mesh.merge(flattened, &aliases);
		
		return unbound_nodes;
	}
	/// Returns a vector references to the flattend meshes of all sub-units referenced by this unit
	fn flatten_subunits(&self, pre_flattened: &Map<String,Rc<flat::Mesh>>) -> Vec<Rc<flat::Mesh>>
	{
		let mut ret = Vec::with_capacity(self.subunits.len());
		for unitref in self.subunits.iter()
		{
			let unit = match pre_flattened.find(&unitref.name) {
				Some(x) => x.clone(),
				None => fail!("BUG - Subunit '{}' referenced by '{}' not yet converted", unitref.name, self.name),
				};
			ret.push( unit );
		}
		return ret;
	}
	fn get_subunits(&self) -> Vec<String> {
		let mut ret = Vec::with_capacity(self.subunits.len());
		for unitref in self.subunits.iter()
		{
			ret.push( unitref.name.clone() );
		}
		return ret;
	}
}

impl Test
{
	pub fn new(name: &String, exec_limit: uint) -> Test {
		Test {
			unit: Unit::new( &format!("!TEST:{}",name)),
			exec_limit: exec_limit,
			..Default::default()
		}
	}
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
	pub fn add_assert(&mut self, line: uint, conds: LinkList, vals: LinkList, exp: LinkList) {
		self.assertions.push(TestAssert{
			line: line,
			conditions: conds,
			values: vals,
			expected: exp,
			});
	}
	
	pub fn flatten(&mut self, flat_units: &Flatmap) -> flat::Test
	{
		let flat = self.unit.flatten(flat_units);
		let asserts = self.assertions.iter().map( |a|
			flat::TestAssert::new(
				a.line,
				flat::linklist_to_noderefs(&a.conditions),
				flat::linklist_to_noderefs(&a.values),
				flat::linklist_to_noderefs(&a.expected),
				)
			).collect();
		flat::Test::new(flat, self.exec_limit, flat::linklist_to_noderefs(&self.completion), asserts)
	}
}

impl Root
{
	pub fn new() -> Root {
		Root {
			rootunit: Unit::new( &"".to_string() ),
			..Default::default()
			}
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
		let val = Unit::new(name);
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
		
		let val = Test::new(name, exec_limit);
		self.tests.insert(name.clone(), val);
		return self.tests.find_mut(name);
	}
	
	pub fn flatten_root(&mut self) -> flat::Mesh
	{
		let mut flat_units = ::collections::TreeMap::new();
		for name in self.rootunit.get_subunits().iter()
		{
			flatten_unit( &mut self.units, &mut flat_units, name );
		}
		self.flat_units = flat_units;
		let ret = self.rootunit.flatten(&self.flat_units).deref().clone();
		return ret;
	}
	pub fn flatten_tests(&mut self)
	{
		for (name,test) in self.tests.iter()
		{
			info!("Flattening deps for '{}'", name);
			for name in test.unit.get_subunits().iter()
			{
				flatten_unit(&mut self.units, &mut self.flat_units, name);
			}
		}
		for (name,test) in self.tests.mut_iter()
		{
			self.flat_tests.insert( name.clone(), test.flatten(&self.flat_units) );
		}
	}
	
	pub fn iter_tests(&self) -> ::collections::treemap::Entries<String,flat::Test>
	{
		self.flat_tests.iter()
	}
}

fn flatten_unit(units: &mut MutableMap<String,Unit>, flat_units: &mut Flatmap, name: &String)
{
	if flat_units.find(name).is_none()
	{
		for su_name in units.find(name).unwrap().get_subunits().iter()
		{
			flatten_unit(units, flat_units, su_name);
		}
		let unit = units.find_mut(name).unwrap();
		let flat = unit.flatten(&*flat_units);
		flat_units.insert( name.clone(), flat );
	}
}

// vim: ft=rust

