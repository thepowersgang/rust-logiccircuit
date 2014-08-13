//
//
//
#![feature(macro_rules)]
#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate getopts;
extern crate std;
extern crate collections;
extern crate core;

mod cct_mesh;
mod parse;
mod elements;

fn main()
{
	// 1. Parse command line arguments
	let opts = [
		getopts::optflag("h", "help", "Print help text"),
		getopts::optflag("", "test", "Run tests"),
		getopts::optopt("", "test-glob", "Run tests matching glob", "GLOB"),
		];
	let args = match getopts::getopts(std::os::args().as_slice(), opts) {
		Ok(m) => m,
		Err(f) => fail!(f.to_string()),
		};
	
	if args.opt_present("h") {
		print_usage( std::os::args()[0].as_slice(), opts );
		return ;
	}
	
	for argument in args.free.iter() {
		println!("Arg '{}'", argument);
	}
	
	// 2. Load circuit file
	let mesh = parse::load( args.free[1].as_slice() );
	
	let flat = mesh.unwrap().flatten_root();

	// 3. Simulation/Visualisation
}

fn print_usage(program_name: &str, opts: &[getopts::OptGroup])
{
	println!("Usage: {}", getopts::short_usage(program_name, opts));
	println!("");
	::std::io::stdio::print( getopts::usage("Logic gate simulator", opts).as_slice() );
}

// vim: ft=rust


