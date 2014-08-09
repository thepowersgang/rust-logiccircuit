//
//
//

extern crate collections;
extern crate core;
use std::string::String;
use std::rc::Rc;
use std::default::Default;

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
	value: LinkValue,
}

pub type LinkList = Vec<Rc<Link>>;

#[deriving(Default)]
struct Element
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

#[deriving(Default)]
pub struct Unit
{
	name: String,
	inputs: LinkList,
	outputs: LinkList,
	
	links:  collections::TreeMap<String,Rc<Link>>,	// Definitive list of links
	groups: collections::TreeMap<String,Vec<Rc<Link>>>,
	
	visgroups: List<VisGroup>,
}

#[deriving(Default)]
pub struct Root
{
	value_zero: LinkValue,
	value_one: LinkValue,
	
	pub rootunit: Unit,
	units: collections::TreeMap<String,Unit>,
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

impl core::cmp::Ord for Link {
	fn cmp(&self, x: &Link) -> Ordering { self.name.cmp(&x.name) }
}
impl core::cmp::PartialEq for Link {
	fn eq(&self, x: &Link) -> bool { self.name == x.name }
}
impl core::cmp::PartialOrd for Link {
	fn partial_cmp(&self, x: &Link) -> Option<Ordering> { Some(self.cmp(x)) }
}
impl core::cmp::Eq for Link
{
}

impl Unit
{
	pub fn get_link(&mut self, name: &String) -> Rc<Link> {
		match self.links.find(name)
		{
		Some(x) => return x.clone(),
		None => ()
		}
		
		let val = Rc::new( Link { name: name.clone(), .. Default::default() } );
		self.links.insert(name.clone(), val.clone());
		val
	}
	
	pub fn make_group(&mut self, name: &String, size: uint) {
		let mut val = Vec::with_capacity(size);
		for i in range(0,size) {
			val.push( self.get_link(&format!("{}[{:u}]", name, i)) );
		}
		self.groups.insert(name.clone(), val);
	}
	pub fn get_group(&self, name: &String) -> Option<&Vec<Rc<Link>>> {
		return self.groups.find(name)
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
		Some(x) => return None,
		None => ()
		}
		let val = Unit { ..Default::default() };
		self.units.insert(name.clone(), val);
		return self.units.find_mut(name);
	}
}

// vim: ft=rust

