//
//
//
use std::rc::Rc;
use std::collections::{HashMap,hash_map};
use std::collections::LinkedList;

use cct_mesh::flat::NodeRef;
use cct_mesh::flat::NodeRef::*;

pub mod flat;

macro_rules! chain{ ($base:expr, $($next:expr),+) => ( $base $(.chain($next) )+ ) }
macro_rules! zip  { ($base:expr, $($next:expr),+) => ( $base $(.zip($next) )+ ) }

#[derive(Copy,Clone,Debug,PartialEq,PartialOrd)]
struct LinkIdx(u32);
impl From<usize> for LinkIdx { fn from(v: usize) -> Self { LinkIdx(v as u32) } }
impl From<u32> for LinkIdx { fn from(v: u32) -> Self { LinkIdx(v) } }
impl ::std::ops::Deref for LinkIdx { type Target = u32; fn deref(&self) -> &u32 { &self.0 } }
#[derive(Default,Debug)]
struct Link
{
	name: String,
	reflink: Option<LinkRef>,
	aliased: Option<LinkIdx>,	// Used during node counting
}

#[derive(Debug)]
pub struct LinkRef(usize);
pub type LinkList = Vec<LinkRef>;
type Flatmap = HashMap<String,Rc<flat::Mesh>>;

pub struct Element
{
	inst: Box<::elements::Element+'static>,
	inputs: LinkList,
	outputs: LinkList,
}

//#[derive(Default)]
//struct VisGroup
//{
//	name: String,
//	elements: LinkedList<Element>
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

#[derive(Default)]
pub struct Unit
{
	name: String,
	inputs: LinkList,
	outputs: LinkList,
	
	link_zero: LinkRef,
	link_one: LinkRef,
	anon_links: LinkedList<LinkRef>,
	links:  ::std::collections::HashMap<String,LinkRef>,	// Definitive list of links
	groups: ::std::collections::HashMap<String,LinkList>,
	link_collection: Vec<Link>,
	
	elements: LinkedList<Element>,
	subunits: LinkedList<UnitRef>,
	
	breakpoints: LinkedList<Breakpoint>,
	disp_items: LinkedList<DisplayItem>,
	
	//visgroups: LinkedList<VisGroup>,
	
	flattened: Option<Rc<flat::Mesh>>,
}

struct TestAssert
{
	line: u32,
	conditions: LinkList,
	values: LinkList,
	expected: LinkList,
}

#[derive(Default)]
pub struct Test
{
	exec_limit: u32,
	completion: LinkList,
	unit: Unit,
	assertions: LinkedList<TestAssert>,
}

#[derive(Default)]
pub struct Root
{
	rootunit: Unit,
	units: ::std::collections::HashMap<String,Unit>,
	tests: ::std::collections::HashMap<String,Test>,
	
	flat_units: Flatmap,
	flat_tests: ::std::collections::HashMap<String,flat::Test>,
}

impl Clone for Box<::elements::Element+'static>
{
	fn clone(&self) -> Box<::elements::Element+'static> {
		return self.dup();
	}
}

macro_rules! exp{ ($val:expr, $e:pat => $res:expr) => (match $val { $e=>$res, _=>panic!("exp!")}) }

impl ::std::cmp::Ord for Link {
	fn cmp(&self, x: &Link) -> ::std::cmp::Ordering { self.name.cmp(&x.name) }
}
impl ::std::cmp::PartialEq for Link {
	fn eq(&self, x: &Link) -> bool { self.name == x.name }
}
impl ::std::cmp::PartialOrd for Link {
	fn partial_cmp(&self, x: &Link) -> Option<::std::cmp::Ordering> { Some(self.cmp(x)) }
}
impl ::std::cmp::Eq for Link { }

impl Link
{
	pub fn new<T: Into<String>>(name: T) -> Link {
		Link {
			name: name.into(),
			..Default::default()
		}
	}
	
	/// Bind this link to another
	pub fn bind(&mut self, other: &LinkRef) {
		self.reflink = Some(other.clone());
		//debug!("Bound link {} to {}'s value", self.name, other.name);
	}
	/// Tag a link with the target value
	pub fn tag(&mut self, value: LinkIdx) -> bool {
		if let Some(ref tgt) = self.aliased {
			panic!("Link '{}' already aliased to #{:?}", self.name, tgt);
		}
		match self.reflink {
		Some(ref l) => {
			debug!("Link '{}' refers to #{:?}'", self.name, l);
			false
			},
		None => {
			debug!("Tagging '{}' to {:?}", self.name, value);
			self.aliased = Some(value);
			true
			}
		}
	}
	/// ? Set the alias/tag by following `self.reflink`
	pub fn tag_from_ref(&mut self)
	{
		if self.aliased == None
		{
			match self.reflink
			{
			Some(ref other_ref) => {
				let other = other_ref.clone();
				loop
				{
					let other_link: &Link = unimplemented!();
					if let Some(ref x) = other_link.reflink
					{
						other = (*x).clone();
					}
					else
					{
						self.aliased = other_link.aliased;
						debug!("Indirect tag of '{}' from '{}' ({:?})",
							self.name, other_link.name, self.aliased);
						assert!(self.aliased != None);
						break;
					}
				}
				
				
				},
			None => {}
			}
		}
	}
	pub fn get_alias(&self) -> Option<LinkIdx> {
		self.aliased
	}
}

impl Clone for LinkRef {
	fn clone(&self) -> LinkRef {
		LinkRef( self.0 )
	}
}
impl Default for LinkRef {
	fn default() -> LinkRef {
		LinkRef(!0)
	}
}

//impl Default for Unit
//{
//	fn default() -> Unit {
//		Unit {
//			name: "".to_string(),
//			inputs: Vec::new(),
//			outputs: Vec::new(),
//			
//			link_zero: Unit::make_link( "=0".to_string() ),
//			link_one: Unit::make_link( "=1".to_string() ),
//			anon_links: LinkedList::new(),
//			links:  ::std::collections::HashMap::new(),	// Definitive list of links
//			groups: ::std::collections::HashMap::new(),
//			
//			elements: LinkedList::new(),
//			subunits: LinkedList::new(),
//			
//			breakpoints: LinkedList::new(),
//			disp_items: LinkedList::new(),
//			//visgroups: LinkedList::new(),
//			flattened: None,
//		}
//	}
//}
impl Unit
{
	pub fn new(name: String) -> Unit {
		Unit {
			name: name,
			link_zero: LinkRef(0),
			link_one:  LinkRef(1),
			links: chain!(
				Some((String::from("=0"), LinkRef(0))).into_iter(),
				Some((String::from("=1"), LinkRef(1))).into_iter()
				).collect(),
			link_collection: vec![ Link::new("=0"), Link::new("=1"), ],
			..Default::default()
			}
	}
	
	fn make_link(&mut self, name: String) -> LinkRef {
		self.link_collection.push( Link { name: name, .. Default::default() } );
		LinkRef( self.link_collection.len()-1 )
	}
	pub fn get_constant(&mut self, is_one: bool) -> LinkRef {
		if is_one {
			self.link_one.clone()
		}
		else {
			self.link_zero.clone()
		}
	}
	pub fn get_link_ref(&self, lr: &LinkRef) -> &Link {
		&self.link_collection[lr.0]
	}
	pub fn get_link_mut(&mut self, lr: &LinkRef) -> &mut Link {
		&mut self.link_collection[lr.0]
	}
	
	pub fn get_link(&mut self, name: &str) -> LinkRef {
		match self.links.get(name)
		{
		Some(x) => return x.clone(),
		None => ()
		}
		
		let val = self.make_link(String::from(name));
		self.links.insert(String::from(name), val.clone());
		val
	}
	fn make_anon_link(&mut self) -> LinkRef {
		let name = format!("#{}", self.anon_links.len());
		let link = self.make_link(name);
		self.anon_links.push_back( link.clone() );
		return link;
	}
	pub fn make_anon_links(&mut self, count: usize) -> LinkList {
		(0 .. count).map(|_| self.make_anon_link()).collect()
	}
	
	pub fn make_group(&mut self, name: &String, size: usize) {
		let mut val = Vec::with_capacity(size);
		for i in 0 .. size {
			val.push( self.get_link(&format!("{}[{:2}]", name, i)) );
		}
		self.groups.insert(name.clone(), val);
		debug!("make_group: {} created with {} items", *name, size);
	}
	pub fn get_group(&self, name: &String) -> Option<&LinkList> {
		return self.groups.get(name)
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
	
	pub fn append_element(&mut self, meshroot: &Root, name: &str, params: Vec<u64>, inputs: LinkList, outputs: Option<LinkList>) -> Result<LinkList,String>
	{
		debug!("append_element('{}', {:?}, in={:?}, out={:?})", name, params, inputs, outputs);
		match meshroot.get_unit(name)
		{
		// Referencing a sub-unit
		Some(unit) => {
			let out = match outputs { None => self.make_anon_links(unit.outputs.len()), Some(o) => o };
			if out.len() != unit.outputs.len() {
				return Err( format!("Output mismatch for unit '{}', got {} expected {}",
					name, out.len(), unit.outputs.len()) );
			}
			if inputs.len() != unit.inputs.len() {
				return Err(format!("Input mismatch for unit '{}', got {} expected {}",
					name, inputs.len(), unit.inputs.len()));
			}
			let r = UnitRef {
				name: String::from(name),
				inputs: inputs,
				outputs: out.clone(),
				};
			self.subunits.push_back(r);
			Ok( out )
			},
		None => {
			let ele = try!(::elements::create(name, &*params, inputs.len()));
			
			let out = match outputs { Some(o) => o, None => self.make_anon_links( ele.get_outputs(inputs.len()) ) };
			
			self.elements.push_back( Element {
				inst: ele,
				inputs: inputs,
				outputs: out.clone(),
				});
			Ok( out )
			}
		}
	}
	pub fn append_display(&mut self, cond: LinkList, text: String, values: LinkList) {
		self.disp_items.push_back( DisplayItem {
			condition: cond,
			text: text,
			values: values,
			});
	}
	pub fn append_breakpoint(&mut self, name: String, cond: LinkList) {
		self.breakpoints.push_back( Breakpoint {
			conds: cond,
			name: name,
			});
	}
	
	pub fn flatten(&mut self, pre_flattened: &HashMap<String,Rc<flat::Mesh>>) -> Rc<flat::Mesh>
	{
		debug!("Flattening unit '{}'", self.name);
		let subunits = self.flatten_subunits(pre_flattened);
		
		let mut n_eles = 0;
		let mut n_links: usize = 0;
		
		n_eles += self.elements.len();
		
		// Count nodes
		debug!("Tagging anon links (Unit '{}')", self.name);
		// - Tag (and count) all links that arent not referencing another link
		for link in chain!( self.anon_links.iter(), self.links.values() )
		{
			if self.link_collection[link.0].tag(From::from(n_links)) {
				n_links += 1;
			}
		}
		// - And once all non-reference links are tagged, copy those tags to the reference links
		for link in chain!( self.anon_links.iter(), self.links.values() )
		{
			self.link_collection[link.0].tag_from_ref();
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
			for (i,e) in chain!( subu.inputs.iter().enumerate(), subu.outputs.iter().enumerate() ) {
				debug!("ext {}=#{:?}", i, *e);
				match *e { NodeId(_) => { n_links -= 1; }, _=>{}}
			}
			// Add breakpoints and display items
			n_bps += subu.breakpoints.len();
			n_disp += subu.dispitems.len();
		}
		info!("w/ subunits n_eles={}, n_links={}, n_bps={}, n_disp={}",
			n_eles, n_links, n_bps, n_disp);
		
		let mut ret = flat::Mesh::new(n_links, n_eles, n_bps, n_disp, self, &self.inputs, &self.outputs);

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
				inputs:  flat::linklist_to_noderefs(self, &ele.inputs),
				outputs: flat::linklist_to_noderefs(self, &ele.outputs),
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
				flat::linklist_to_noderefs(self, &bp.conds)
				) );
		}
		// Add display items
		for di in self.disp_items.iter()
		{
			debug!("Display item '{}'", di.text);
			ret.push_disp( flat::Display::new(
				di.text.clone(),
				flat::linklist_to_noderefs(self, &di.condition),
				flat::linklist_to_noderefs(self, &di.values),
				) );
		}
		
		// Populate from sub-units
		let mut bind_node_idx = n_local_links as u32;
		for (i,subu) in self.subunits.iter().enumerate()
		{
			bind_node_idx += self.flatten_merge_subunit(&mut ret, &*subunits[i], subu, bind_node_idx);
		}
		assert!(bind_node_idx as usize == n_links);
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
	fn flatten_merge_subunit(&self, mesh: &mut flat::Mesh, flattened: &flat::Mesh, subu: &UnitRef, bind_node_idx: u32) -> u32
	{
		let inputs  = flat::linklist_to_noderefs( self, &subu.inputs );
		let outputs = flat::linklist_to_noderefs( self, &subu.outputs );
		let mut aliases: Vec<Option<NodeRef>> = ::from_elem( flattened.n_nodes, None );
		
		assert_eq!( flattened.inputs.len(),  inputs.len()  );
		assert_eq!( flattened.outputs.len(), outputs.len() );
		
		info!("> Import subunit {}", subu.name);
		// Iterate all external links, aliasing the inside node to the relevant outside node
		for (j,noderef) in flattened.inputs.iter().enumerate()
		{
			let inner_node = exp!(*noderef, NodeId(id) => id);
			aliases[inner_node as usize] = Some(inputs[j]);
		}
		for (j,noderef) in flattened.outputs.iter().enumerate()
		{
			let inner_node = exp!(*noderef,   NodeId(id) => id);
			aliases[inner_node as usize] = Some(outputs[j]);
		}
		
		// Count (and alias) all nodes that were not external links
		let mut unbound_nodes = 0;
		for (_j,alias) in aliases.iter_mut().enumerate()
		{
			if alias.is_none()
			{
				//debug!("Alias inner #{} to outer #NodeId({}) (int)", _j, bind_node_idx);
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
	fn flatten_subunits(&self, pre_flattened: &HashMap<String,Rc<flat::Mesh>>) -> Vec<Rc<flat::Mesh>>
	{
		let mut ret = Vec::with_capacity(self.subunits.len());
		for unitref in self.subunits.iter()
		{
			let unit = match pre_flattened.get(&unitref.name) {
				Some(x) => x.clone(),
				None => panic!("BUG - Subunit '{}' referenced by '{}' not yet converted", unitref.name, self.name),
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
	pub fn new(name: String, exec_limit: u32) -> Test {
		Test {
			unit: Unit::new(format!("!TEST:{}",name)),
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
	pub fn add_assert(&mut self, line: u32, conds: LinkList, vals: LinkList, exp: LinkList) {
		self.assertions.push_back(TestAssert{
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
				flat::linklist_to_noderefs(&self.unit, &a.conditions),
				flat::linklist_to_noderefs(&self.unit, &a.values),
				flat::linklist_to_noderefs(&self.unit, &a.expected),
				)
			).collect();
		flat::Test::new(flat, self.exec_limit, flat::linklist_to_noderefs(&self.unit, &self.completion), asserts)
	}
}

impl Root
{
	pub fn new() -> Root {
		Root {
			rootunit: Unit::new( String::from("") ),
			..Default::default()
			}
	}
	
	pub fn get_root_unit(&mut self) -> &mut Unit {
		return &mut self.rootunit;
	}
	pub fn add_unit(&mut self, name: String) -> Result<&mut Unit,String> {
		match self.units.entry(name.clone())
		{
		hash_map::Entry::Occupied(_) => Err(name),
		hash_map::Entry::Vacant(e) => Ok( e.insert(Unit::new(name)) ),
		}
	}
	pub fn get_unit(&self, name: &str) -> Option<&Unit> {
		self.units.get(name)
	}
	pub fn add_test(&mut self, name: String, exec_limit: u32) -> Result<&mut Test,String> {
		match self.tests.entry(name.clone())
		{
		hash_map::Entry::Occupied(_) => Err(name),
		hash_map::Entry::Vacant(e) => Ok( e.insert(Test::new(name, exec_limit)) ),
		}
	}
	
	pub fn flatten_root(&mut self) -> flat::Mesh
	{
		let mut flat_units = ::std::collections::HashMap::new();
		for name in self.rootunit.get_subunits().iter()
		{
			flatten_unit( &mut self.units, &mut flat_units, name );
		}
		self.flat_units = flat_units;
		let ret = (*self.rootunit.flatten(&self.flat_units)).clone();
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
		for (name,test) in self.tests.iter_mut()
		{
			self.flat_tests.insert( name.clone(), test.flatten(&self.flat_units) );
		}
	}
	
	pub fn iter_tests(&self) -> ::std::collections::hash_map::Iter<String,flat::Test>
	{
		self.flat_tests.iter()
	}
}

fn flatten_unit(units: &mut HashMap<String,Unit>, flat_units: &mut Flatmap, name: &str)
{
	if flat_units.get(name).is_none()
	{
		for su_name in units[name].get_subunits().iter()
		{
			flatten_unit(units, flat_units, su_name);
		}
		let unit = units.get_mut(name).unwrap();
		let flat = unit.flatten(&*flat_units);
		flat_units.insert( From::from(name), flat );
	}
}

// vim: ft=rust

