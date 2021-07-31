// Copyright 2018-2020 James Bostock. See the LICENSE file at the
// top-level directory of this distribution.

// An implementation of the wc(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/wc
use std::env;
use std::fmt;
use std::io;
use std::ops;

use rust_v7_lib as lib;

struct Format {
    chars: bool, // Technically bytes.
    words: bool,
    lines: bool
}

impl Format {
    fn new() -> Format {
        // By default, all values are printed.
        Format {chars: false, words: false, lines: false}
    }
}

struct Counts<'a, 'b> {
    chars: usize,
    words: usize,
    lines: usize,
    file: &'a str,
    in_word: bool,
    format: &'b Format
}

impl<'a, 'b> Counts<'a, 'b> {
    fn new(file: &'a str, format: &'b Format) -> Counts<'a, 'b> {
        Counts {chars: 0, words: 0, lines: 0, file, in_word: false,
                format}
    }
}

impl<'a, 'b> ops::AddAssign for Counts<'a, 'b> {

    fn add_assign(&mut self, rhs: Counts) {
        *self = Counts {
            chars: self.chars + rhs.chars,
            words: self.words + rhs.words,
            lines: self.lines + rhs.lines,
            file: self.file,
            in_word: self.in_word,
            format: self.format
        };
    }
}

impl<'a, 'b> fmt::Display for Counts<'a, 'b> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.format.lines {
            let _ = write!(f, "{:7} ", self.lines);
        }
        if self.format.words {
            let _ = write!(f, "{:7} ", self.words);
        }
        if self.format.chars {
            let _ = write!(f, "{:7} ", self.chars);
        }
        write!(f, "{}", self.file)
    }
}

impl<'a, 'b> io::Write for Counts<'a, 'b> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // ASCII codes of the characters that we consider whitespace.
        const CR: u8 = 13; // Carriage return
        const HT: u8 = 9;  // Horizontal tab
        const LF: u8 = 10; // Line feed
        const SP: u8 = 32; // Space
        const VT: u8 = 11; // Vertical tab

        for c in buf {
            if *c == CR || *c == HT || *c == LF || *c == SP || *c == VT {
                if self.in_word {
                    self.in_word = false
                }
                if *c == LF {
                    self.lines += 1;
                }
            } else if !self.in_word {
                self.in_word = true;
                self.words += 1;
            }
        }
        self.chars += buf.len();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn wc<'a, 'b>(filename: &'a str, format: &'b Format) -> io::Result<Counts<'a, 'b>> {
    let mut reader = lib::Input::open(filename)?;
    let mut wc = Counts::new(filename, format);
    io::copy(&mut reader, &mut wc)?;
    Ok(wc)
}

fn main () {
    let mut args = env::args();
    let prog = args.next().unwrap();

    let mut format = Format::new();
    let getopt = lib::GetOpt::new("clw", args);

    let mut format_specified = false;
    let mut files : Vec<String> = Vec::new();

    for optarg in getopt {
        match optarg {
            Ok(lib::Arg::Opt('c')) => {
                format.chars = true;
                format_specified = true;
            },
            Ok(lib::Arg::Opt('l')) => {
                format.lines = true;
                format_specified = true;
            },
            Ok(lib::Arg::Opt('w')) => {
                format.words = true;
                format_specified = true;
            },
            Ok(lib::Arg::Arg(arg)) => files.push(arg),
	    Ok(val) => {
		// Should never happen.
		eprintln!("{}: error: unexpected: {:?}", prog, val);
		std::process::exit(1);
	    },
	    Err(e) => {
		eprintln!("{}: error: {}", prog, e);
		std::process::exit(1);
	    }
        }
    }

    if !format_specified {
        format.chars = true;
        format.lines = true;
        format.words = true;
    }

    let mut total = Counts::new("total", &format);

    if files.is_empty() {
        files.push("-".to_string());
    }

    for arg in &files {
        match wc(arg, &format) {
            Ok(wc) => {
                println!("{}", wc);
                total += wc;
            }
            Err(e) => {
                eprintln!("{}: {}: {}", prog, arg, e);
            }
        };
    }

    if files.len() > 1 {
        println!("{}", total);
    }
}
