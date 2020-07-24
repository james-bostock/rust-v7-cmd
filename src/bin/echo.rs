// Copyright 2015-2020 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the echo(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/echo

fn main() {
    let mut first = true;
    for arg in std::env::args().skip(1) {
	if first {
	    first = false;
	} else {
	    print!(" ");
	}
        print!("{}", arg);
    }
    println!();
}
