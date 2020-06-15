// Copyright 2017-2020 James Bostock. See the LICENSE file at the
// top-level directory of this distribution.

// An implementation of the tee(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/tee
use std::env;
use std::fs::File;
use std::io;
use std::io::{Result, Write};

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

fn main() {
    let args: Vec<_> = env::args().collect();

    let prog = &args[0];

    let mut tee: Tee = Tee::new();

    tee.push(Box::new(io::stdout()));

    for arg in args.iter().skip(1) {
        match File::create(&arg) {
            Ok(f) => { tee.writers.push(Box::new(f)); },
            Err(e) => { eprintln!("{}: {}: {}", prog, arg, e); }
        }
    }

    io::copy(&mut io::stdin(), &mut tee).expect(prog);
}
