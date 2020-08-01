// Copyright 2016-2020 James Bostock. See the LICENSE file at the
// top-level directory of this distribution.

// An implementation of the rm(1) command in Rust.
// See http://man.cat-v.org/unix-7th/1/rm
use std::env;
use std::fs;
use std::io;
use std::io::Write;

use rust_v7_lib as lib;

/// Prompts user for confirmation
fn confirm(msg: &str) -> io::Result<bool> {
    print!("{}: ", msg);
    io::stdout().flush()?;
    let mut resp = String::new();
    io::stdin().read_line(&mut resp)?;
    match resp.chars().next() {
        Some(c) => {
            if c == 'y' {
                Ok(true)
            } else {
                Ok(false)
            }
        },
        None => Ok(false)
    }
}

/// Removes a file or directory. Returns OK(()) unless one of the
/// filesystem operations fails.
fn rm(prog: &str, name: &str, force: bool, recursive: bool,
      interactive: bool) -> io::Result<()> {
    let md = fs::metadata(name)?;
    let readonly = md.permissions().readonly();

    if name == "." || name == ".." {
	println!("{}: cannot remove directory '{}'", prog, name);
	return Ok(())
    }

    if md.is_dir() && !recursive {
	println!("{}: cannot remove '{}': it is a directory", prog, name);
	return Ok(())
    }

    let go = if (!force && readonly) || interactive {
	let msg = format!("{}: remove {}{} '{}'?",
			  prog,
			  if readonly {
			      "readonly "
			  } else {
			      ""
			  },
			  if md.is_dir() {
			      "directory"
			  } else {
			      "file"
			  },
			  name
	);
        confirm(&msg)?
    } else {
        true
    };

    if go {
        if md.is_dir() {
            fs::remove_dir_all(name)
        } else {
	    fs::remove_file(name)
        }
    } else {
        Ok(())
    }
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap();
    let mut force: bool = false;
    let mut interactive: bool = false;
    let mut recursive: bool = false;
    let mut print_usage = true;
    let getopt = lib::GetOpt::new("fri", args);

    for optarg in getopt {
        match optarg {
            Ok(lib::Arg::Opt('f')) => force = true,
            Ok(lib::Arg::Opt('r')) => recursive = true,
            Ok(lib::Arg::Opt('i')) => interactive = true,
            Ok(lib::Arg::Arg(arg)) => {
                match rm(&prog, &arg, force, recursive, interactive) {
                    Ok(_) => print_usage = false,
                    Err(e) => {
                        eprintln!("{}: {}", arg, e);
                        std::process::exit(1);
                    }
                }
            }
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

    if print_usage {
        eprintln!("usage: {} [-fri] file ...", prog);
        std::process::exit(1);
    }
    std::process::exit(0);
}
