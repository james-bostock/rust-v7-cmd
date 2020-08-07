// Copyright 2017-2020 James Bostock. See the LICENSE file at the
// top-level directory of this distribution.

// An implementation of the tee(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/tee
use std::env;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::{Result, Write};

use rust_v7_lib as lib;

/// A multi-way writer.
struct Tee {
    writers: Vec<Box<dyn Write>>
}

impl Tee {
    // Create a new Tee
    fn new() -> Self {
        Tee { writers: Vec::new() }
    }

    // Add a writer to a Tee
    fn push(&mut self, w: Box<dyn Write>) {
        self.writers.push(w);
    }
}

impl Write for Tee {
    /// Writes a buffer to each of the writers, returning how many
    /// bytes were returned by the last write.
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut n: usize = 0;
        for w in &mut self.writers {
            n = w.write(buf)?
        }
        Ok(n)
    }

    /// Flushes each writer.
    fn flush(&mut self) -> Result<()> {
        for w in &mut self.writers {
            w.flush()?
        }
        Ok(())
    }
}

// Opens a file for either writing (i.e. truncating) or appending.
fn open_helper(path: &str, append: bool) -> io::Result<File> {
    if append {
	OpenOptions::new().append(true).open(path)
    } else {
	File::create(path)
    }
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let getopt = lib::GetOpt::new("a", args);
    let mut tee: Tee = Tee::new();
    let mut append = false;

    tee.push(Box::new(io::stdout()));

    for optarg in getopt {
        match optarg {
	    Ok(lib::Arg::Opt('a')) => append = true,
            Ok(lib::Arg::Arg(arg)) => {
		match open_helper(&arg, append) {
			Ok(f) => { tee.writers.push(Box::new(f)); },
			Err(e) => { eprintln!("{}: {}: {}", prog, arg, e); }
		    }
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

    io::copy(&mut io::stdin(), &mut tee).expect(&prog);
}
