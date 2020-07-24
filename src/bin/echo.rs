// Copyright 2015-2020 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the echo(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/echo

use std::env;

use rust_v7_lib as lib;

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let getopt = lib::GetOpt::new("n", args);
    let mut first = true;
    let mut newline = true;

    for optarg in getopt {
        match optarg {
            Ok(lib::Arg::Opt('n')) => newline = false,
	    Ok(lib::Arg::Arg(arg)) => {
		if first {
		    first = false;
		} else {
		    print!(" ");
		}
		print!("{}", arg);
	    },
            Ok(val) => {
                eprintln!("{}: error: unexpected: {:?}", prog, val);
                std::process::exit(1);
            },
            Err(e) => {
                eprintln!("{}: error: {}", prog, e);
                std::process::exit(1);
            }
	}
    }

    if newline {
	println!();
    }
}
