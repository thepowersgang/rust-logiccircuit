//
//
//

use std::string::String;
use std::rc::Rc;
use std::cell::RefCell;
use std::default::Default;
use std::collections::Deque;

enum List<T> {
    Cons(T, Box<List<T>>),
    Nil
}

#[deriving(Default)]
struct LinkValue
{
	has_changed: bool,
	value:       bool,
	idle_time:   u32,
	n_drivers:   u32,
	reference_count: u32,
}

#[deriving(Default)]
struct Link
{
	name: String,
	value: Rc<RefCell<LinkValue>>,
}

pub type LinkRef = Rc<RefCell<Link>>;
pub type LinkList = Vec<LinkRef>;

#[deriving(Default)]
pub struct Element
{
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
	value_zero: LinkValue,
	value_one: LinkValue,
	
	rootunit: Unit,
	units: ::collections::TreeMap<String,Unit>,
	tests: ::collections::TreeMap<String,Test>,
}

// Represents a flattened (executable) mesh
pub struct Mesh
{
	values: Vec<LinkValue>,
	elements: Vec<Element>,
}

impl<T> Default for List<T>
{
	fn default() -> List<T> {
		Nil
	}
}

impl Default for RefCell<LinkValue> {
	fn default() -> RefCell<LinkValue> {
		RefCell::new(LinkValue {..Default::default()})
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
	pub fn bind(&mut self, other: &Link) {
		self.value = other.value.clone();
		//debug!("Bound link {} to {}'s value", self.name, other.name);
	}
}

impl Element
{
	pub fn new(name: String, args: Vec<u64>, inputs: LinkList) -> Element {
		Element { inputs: inputs, ..Default::default() }
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
			val.push( self.get_link(&format!("{}[{:u}]", name, i)) );
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
	
	pub fn append_element(&mut self, element: Element) -> &mut Element {
		self.elements.push(element);
		return self.elements.back_mut().unwrap();
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
		let val = Unit { ..Default::default() };
		self.units.insert(name.clone(), val);
		return self.units.find_mut(name);
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
}

// vim: ft=rust

