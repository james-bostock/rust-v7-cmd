// Copyright 2015-2020 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the cat(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/cat
use std::env;
use std::io;

use rust_v7_lib as lib;

fn cat(filename: &str) -> io::Result<u64> {
    let mut reader = lib::Input::open(filename)?;
    io::copy(&mut reader, &mut io::stdout())
}

fn main() {
    let mut args: Vec<_> = env::args().collect();

    if args.len() == 1 {
        args.push("-".to_string());
    }

    for arg in args.iter().skip(1) {
        match cat(arg) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}: {}", arg, e);
            }
        };
    }
}
