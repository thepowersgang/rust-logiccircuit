//
//
//
#![feature(globs)]
#![feature(macro_rules)]
#![feature(phase)]
#[phase(plugin, link)] extern crate log;

extern crate getopts;
extern crate std;
extern crate collections;
extern crate core;
extern crate glob;

mod cct_mesh;
mod parse;
mod elements;
mod simulator;

enum TestStatus
{
	TestPass(uint),
	TestFail(uint, String),
	TestTimeout(uint),
}

fn main()
{
	println!("main()");
	// 1. Parse command line arguments
	let opts = [
		getopts::optflag("h", "help", "Print help text"),
		getopts::optflag("", "test", "Run tests"),
		getopts::optopt("", "test-glob", "Run tests matching glob", "GLOB"),
		getopts::optflag("", "test-display", "Print display items during tests"),
		];
	println!("> opts = ");
	let args = match getopts::getopts(std::os::args().as_slice(), opts) {
		Ok(m) => m,
		Err(f) => fail!(f.to_string()),
		};
	println!("> args = {}", args.free);
	
	if args.opt_present("h")
	{
		print_usage( std::os::args()[0].as_slice(), opts );
		return ;
	}
	
	for argument in args.free.iter() {
		println!("Arg '{}'", argument);
	}
	
	// 2. Load circuit file
	let mut mesh = match parse::load( args.free[1].as_slice() ) {
		Some(x) => x,
		None => fail!("Parsing of {} failed", args.free[1])
		};
	
	// - Flatten root (also flattens all other units)
	let flat = mesh.flatten_root();

	// 3. Run the mesh!
	if args.opt_present("test")
	{
		// Run circuit unit tests
		
		let show_display = args.opt_present("test-display");
		let pat = ::glob::Pattern::new( match args.opt_str("test-glob"){Some(ref v)=>v.as_slice(),_=>"*"} );

		// Only flatten tests if required
		// TODO: Pass a glob to this function so it doesn't flatten unless it will be run
		mesh.flatten_tests();
		
		// Unit test!
		for (name,test) in mesh.iter_tests()
		{
			if pat.matches(name.as_slice())
			{
				if show_display {
					println!("TEST: '{}'", name);
				}
				let res = run_test(test, show_display);
				if ! show_display {
					print!("{:40} ", name);
				}
				match res
				{
				TestPass(cyc) => println!("- PASS ({}/{} cycles)", cyc, test.exec_limit()),
				TestFail(cyc,msg) => println!("- FAIL ({} cycles): {}", cyc, msg),
				TestTimeout(cyc) => println!("- TIMEOUT ({} cycles)", cyc),
				}
			}
		}
	}
	else
	{
		// Simulate until stopped
		let mut sim = ::simulator::Engine::new( &flat );
		for i in range(0, 20u)
		{
			sim.tick();
			
			if sim.check_breakpoints()
			{
				println!("Breakpoint hit.");
			}
			sim.show_display();
		}
	}
}

fn run_test(test: &cct_mesh::flat::Test, show_display: bool) -> TestStatus
{
	let mut sim = ::simulator::Engine::new( test.get_mesh() );
	for ticknum in range(0, test.exec_limit())
	{
		sim.tick();
		
		if show_display
		{
			if sim.show_display()
			{
				println!("=== {:4u} ===", ticknum);
			}
		}
		
		if sim.are_set(test.get_completion(), true)
		{
			return TestPass(ticknum+1);
		}
		
		// Check assertions
		for (ass_idx,assert) in test.iter_asserts().enumerate()
		{
			if sim.are_set(&assert.conditions, true)
			{
				let have = sim.get_values(&assert.values);
				let exp  = sim.get_values(&assert.expected);
				
				if have != exp
				{
					return TestFail(ticknum+1, format!("Assertion #{} failed (line {}) - have:{} != exp:{}",
						ass_idx, assert.line, have, exp));
				}
			}
		}
	}
	TestTimeout(test.exec_limit())
}

fn print_usage(program_name: &str, opts: &[getopts::OptGroup])
{
	println!("Usage: {}", getopts::short_usage(program_name, opts));
	println!("");
	::std::io::stdio::print( getopts::usage("Logic gate simulator", opts).as_slice() );
}


// vim: ft=rust


