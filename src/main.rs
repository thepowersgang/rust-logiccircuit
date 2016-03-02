// LogicCircuit simulator
//
//
#[macro_use] extern crate log;
extern crate env_logger;

extern crate getopts;
extern crate glob;

// HACK!
fn from_elem<T: Clone, C: ::std::iter::FromIterator<T>>(count: usize, val: T) -> C {
	::std::iter::repeat(val).take(count).collect()
}

mod cct_mesh;
mod parse;
mod elements;
mod simulator;

enum TestStatus
{
	Pass(u32),
	Fail(u32, String),
	Timeout(u32),
}

//#[cfg(not(test))]
#[allow(dead_code)]
fn main()
{
	env_logger::init().unwrap();
	
	println!("main()");
	// 1. Parse command line arguments
	let mut opts = ::getopts::Options::new();
	opts.optflag("h", "help", "Print help text");
	opts.optflag("", "test", "Run tests");
	opts.optopt("", "test-glob", "Run tests matching glob", "GLOB");
	opts.optflag("", "test-display", "Print display items during tests");

	println!("> opts = ");
	let args_s: Vec<_> = ::std::env::args().collect();
	let args = match opts.parse(&args_s[1..])
		{
		Ok(m) => m,
		Err(f) => panic!(f.to_string()),
		};
	println!("> args = {:?}", args.free);
	
	if args.opt_present("h")
	{
		print_usage( &args_s[0], &opts );
		return ;
	}
	
	for argument in args.free.iter() {
		println!("Arg '{}'", argument);
	}
	
	// 2. Load circuit file
	let mut mesh = match parse::load( &args.free[0] ) {
		Some(x) => x,
		None => panic!("Parsing of {} failed", args.free[1])
		};
	
	// - Flatten root (also flattens all other units)
	let flat = mesh.flatten_root();

	// 3. Run the mesh!
	if args.opt_present("test")
	{
		// Run circuit unit tests
		
		let show_display = args.opt_present("test-display");
		let test_glob = args.opt_str("test-glob").unwrap_or( From::from("*") );
		let pat = ::glob::Pattern::new(&*test_glob).unwrap();

		// Only flatten tests if required
		// TODO: Pass a glob to this function so it doesn't flatten unless it will be run
		mesh.flatten_tests();
		
		// Unit test!
		for (name,test) in mesh.iter_tests()
		{
			if pat.matches(&name)
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
				TestStatus::Pass(cyc) => println!("- PASS ({}/{} cycles)", cyc, test.exec_limit()),
				TestStatus::Fail(cyc,msg) => println!("- FAIL ({} cycles): {}", cyc, msg),
				TestStatus::Timeout(cyc) => println!("- TIMEOUT ({} cycles)", cyc),
				}
			}
		}
	}
	else
	{
		// Simulate until stopped
		let mut sim = ::simulator::Engine::new( &flat );
		let step_count: u32 = 20;
		for _ in (0 .. step_count)
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
	for ticknum in (0 .. test.exec_limit())
	{
		sim.tick();
		
		if show_display
		{
			if sim.show_display()
			{
				println!("=== {:4} ===", ticknum);
			}
		}
		
		if sim.are_set(test.get_completion(), true)
		{
			return TestStatus::Pass(ticknum+1);
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
					return TestStatus::Fail(ticknum+1, format!("Assertion #{} failed (line {}) - have:{:?} != exp:{:?}",
						ass_idx, assert.line, have, exp));
				}
			}
		}
	}
	TestStatus::Timeout(test.exec_limit())
}

fn print_usage(program_name: &str, opts: &::getopts::Options)
{
	println!("Usage: {}", opts.short_usage(program_name));
	println!("");
	println!("{}", opts.usage("Logic gate simulator") );
}


// vim: ft=rust


