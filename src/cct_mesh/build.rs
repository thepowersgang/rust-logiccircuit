

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

pub struct Element
{
	inst: Box<::elements::Element>,
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
	
	flattened: Option<Rc<Mesh>>,
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

macro_rules! exp( ($val:expr, $e:pat => $res:expr) => (match $val { $e=>$res, _=>fail!("exp!")}))

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
