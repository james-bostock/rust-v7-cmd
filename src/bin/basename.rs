// Copyright 2021 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the basename(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/basename
use std::env;
use std::path;
use std::process;

fn basename(path: &str) -> Option<&str> {
    match path::Path::new(path).file_name() {
	Some(file_name) => file_name.to_str(),
	None => Some(path)
    }
}

fn main() {
    let args: Vec<_> = env::args().collect();
    match args.len() {
	2 => {
	    match basename(&args[1]) {
		Some(s) => println!("{}", s),
		None => eprintln!("basename failed")
	    }
	},
	3 => {
	    match basename(&args[1]) {
		Some(s) => {
		    match s.strip_suffix(&args[2]) {
			Some(s) => println!("{}", s),
			None => println!("{}", s)
		    }
		},
		None => eprintln!("basename failed")
	    }
	},
	_ => {
	    eprintln!("usage: {} <path> [suffix]", args[0]);
	    process::exit(1)
	}
    }
}
