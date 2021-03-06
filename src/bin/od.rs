// Copyright 2016-2021 James Bostock. See the LICENSE file at the top-level
// directory of this distribution.

// An implementation of the od(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/od
use std::env;
use std::io;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Stdout;
use std::io::Write;
use std::num::ParseIntError;

use rust_v7_lib as lib;

type FmtFn = fn(&mut BufWriter<Stdout>, &[u8], usize) -> io::Result<usize>;

/// Writes a chunk of output data as octal byte values.
fn write_oct_bytes(out: &mut BufWriter<Stdout>, data: &[u8], _: usize)
                   -> io::Result<usize> {
    for word in data.chunks(2) {
	if word.len() == 1 {
	    write!(out, " {:03o}", word[0])?;
	} else {
	    write!(out, " {:03o} {:03o}", word[0], word[1])?;
	}
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a word using the supplied format specifier.
macro_rules! write_word {
    ($out:expr, $word:expr, $fmt:expr, $width:expr) => {
	if $word.len() == 1 {
	    write!($out, " {1:>0$}", $width, format!($fmt, u16::from($word[0])))?;
	} else {
            write!($out, " {1:>0$}", $width,
		   format!($fmt, u16::from($word[1]) << 8 | u16::from($word[0])))?;
	}
    }
}

/// Writes a chunk of output data as octal (16 bit) word values. Words are
/// assumed to be little endian.
fn write_oct_words(out: &mut BufWriter<Stdout>, data: &[u8], width: usize)
                   -> io::Result<usize> {
    for word in data.chunks(2) {
	write_word!(out, word, "{:06o}", width);
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of output data as decimal (16 bit) word values. Words are
/// assumed to be little endian.
fn write_dec_words(out: &mut BufWriter<Stdout>, data: &[u8], width: usize)
                   -> io::Result<usize> {
    for word in data.chunks(2) {
	write_word!(out, word, "{:5}", width);
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of output data as hexadecimal (16 bit) word values. Words
/// are assumed to be little endian.
fn write_hex_words(out: &mut BufWriter<Stdout>, data: &[u8], width: usize)
                   -> io::Result<usize> {
    for word in data.chunks(2) {
	write_word!(out, word, "{:04x}", width);
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Writes a chunk of data as ASCII, reverting to octal byte values for
/// non-printable characters. Standard escape sequences are supported.
fn write_ascii_chars(out: &mut BufWriter<Stdout>, data: &[u8], _: usize)
                     -> io::Result<usize> {
    for word in data.chunks(2) {
	write_ascii_char(out, word[0])?;
	if word.len() > 1 {
	    write_ascii_char(out, word[1])?;
	}
    }
    writeln!(out)?;
    Ok(data.len())
}

/// Write a byte as ASCII, reverting to octal byte values for
/// non-printable characters. Standard escape sequences are supported.
fn write_ascii_char(out: &mut BufWriter<Stdout>, byte: u8) -> io::Result<()> {
    match byte {
        7u8 => write!(out, "  \\g")?,
        8u8 => write!(out, "  \\b")?,
        9u8 => write!(out, "  \\t")?,
        10u8 => write!(out, "  \\n")?,
        11u8 => write!(out, "  \\v")?,
        12u8 => write!(out, "  \\f")?,
        13u8 => write!(out, "  \\r")?,
        _ => if !(32u8..=126u8).contains(&byte) {
            write!(out, " {:03o}", byte)?
        } else {
            write!(out, "   {}", byte as char)?
        }
    }

    Ok(())
 }

const CHUNK_SIZE: usize = 16;

// The offset string is of the form [+]offset[.][b]
// +100 => 0o100
// +100. => 100
// +100b => 0o100 * 512
// +100.b => 100 * 512
fn parse_offset(offstr: &str) -> Result<u64, ParseIntError> {
    let mut char_indices = offstr.char_indices().rev();
    let mut mult = 1;
    let (s, r) = match char_indices.next() {
        Some((x, 'b')) => {
            mult = 512;
            match char_indices.next() {
                Some((y, '.')) => (&offstr[0..y], 10),
                Some((_, _)) => (&offstr[0..x], 8),
                None => (&offstr[0..0], 8)
            }
        },
        Some((x, '.')) => (&offstr[0..x], 10),
        Some((_, _)) => (offstr, 8),
        None => (&offstr[0..0], 8)
    };

    match u64::from_str_radix(s, r) {
        Ok(n) => Ok(n * mult),
        Err(e) => Err(e)
    }
}

#[test]
fn test_parse_offset() {
    match parse_offset("100") {
        Ok(off) => assert!(off == 0o100),
        Err(_) => assert!(false)
    }

    match parse_offset("100.") {
        Ok(off) => assert!(off == 100),
        Err(_) => assert!(false)
    }

    match parse_offset("100b") {
        Ok(off) => assert!(off == 0o100 * 512),
        Err(_) => assert!(false)
    }

    match parse_offset("100.b") {
        Ok(off) => assert!(off == 100 * 512),
        Err(_) => assert!(false)
    }
}

/// Dumps the data read from the named input source to the standard output.
fn od(filename: &str, offset: u64,
      fmt_fns: &[FmtFn], width: usize)
      -> io::Result<u64> {
    let mut reader = BufReader::new(lib::Input::open(filename)?);
    let mut writer = BufWriter::new(io::stdout());
    let mut offset = offset;

    if offset > 0 {
        reader.seek(SeekFrom::Start(offset))?;
    }

    let mut chunk = [0; CHUNK_SIZE];
    loop {
        let n = reader.read(&mut chunk)?;
        if n > 0 {
            let mut first = true;
            for fmt_fn in fmt_fns.iter() {
                if first {
                    write!(writer, "{:07o}", offset)?;
                    first = false;
                } else {
                    write!(writer, "       ")?;
                }
                fmt_fn(&mut writer, &chunk[0..n], width)?;
            }
            offset += chunk.len() as u64;
        }

        if n < CHUNK_SIZE {
            break
        }
    }
    writeln!(writer, "{:07o}", offset)?;
    Ok(offset)
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let mut offset : u64 = 0;
    let mut offstr = String::from("0");
    let mut fmt_fns: Vec<FmtFn> = Vec::new();
    let mut width : usize = 0;
    let getopt = lib::GetOpt::new("bcdox", args);

    // Default to reading from standard input.
    let mut filename = String::from("-");

    for arg in getopt {
	match arg {
	    Ok(lib::Arg::Opt('b')) => {
		fmt_fns.push(write_oct_bytes);
		if width < 7 {
		    width = 7;
		}
	    },
	    Ok(lib::Arg::Opt('c')) => {
		fmt_fns.push(write_ascii_chars);
		if width < 7 {
		    width = 7;
		}
	    },
	    Ok(lib::Arg::Opt('d')) => {
		fmt_fns.push(write_dec_words);
		if width < 5 {
		    width = 5;
		}
	    },
	    Ok(lib::Arg::Opt('x')) => {
		fmt_fns.push(write_hex_words);
		if width < 4 {
		    width = 4;
		}
	    },
	    Ok(lib::Arg::Opt('o')) => {
		fmt_fns.push(write_oct_words);
		if width < 6 {
		    width = 6;
		}
	    },
	    Ok(lib::Arg::Arg(val)) => {
		if val.starts_with('+') {
		    offstr = val;
		} else {
		    filename = val;
		}
	    },
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

    // If no output formats have been specified, default to octal words.
    if fmt_fns.is_empty() {
        fmt_fns.push(write_oct_words);
	width = 6;
    }

    match parse_offset(&offstr) {
        Ok(off) => offset = off,
        Err(e) => println!("{}: {}", offstr, e)
    }

    match od(&filename, offset, &fmt_fns, width) {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1)
        }
    }
}
