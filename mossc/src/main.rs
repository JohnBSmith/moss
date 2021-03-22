
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::env;

mod error;
mod parser;
mod typing;
mod generator;
mod system;
mod debug;

#[cfg(test)]
mod tests;

use error::Error;
use parser::parse;
use typing::{TypeChecker, TypeTable};
use generator::generate;
use system::save;

fn compile(s: &str, file: &str, info: &CmdInfo) -> Result<(), Error> {
    let debug_mode = info.debug_mode;
    let t = parse(s)?;
    if debug_mode {
        println!("{}", &t);
    }

    let tab = TypeTable::new();
    let mut checker = TypeChecker::new(&tab);
    checker.type_check(&t)?;

    if debug_mode {
        println!("{}", checker.string(&t));
        println!("{}", checker.subs_as_string());
    }
    checker.apply_types();
    checker.check_constraints()?;

    if debug_mode {
        println!("{}", checker.string(&t));
    }
    let code = generate(&t, checker.symbol_table);
    if debug_mode {
        println!("{}", code);
    }
    if !file.is_empty() {
        save(&code, file);
    }
    Ok(())
}

fn read_file(path: &str) -> Result<String, std::io::Error> {
    let file = File::open(path)?;
    let mut buf_reader = BufReader::new(file);
    let mut s = String::new();
    buf_reader.read_to_string(&mut s)?;
    Ok(s)
}

fn is_option(arg: &str) -> bool {
    !arg.is_empty() && &arg[0..1] == "-"
}

struct CmdInfo {
    id: Option<String>,
    debug_mode: bool,
    eval: bool
}

impl CmdInfo {
    pub fn new() -> Self {
        let mut info = CmdInfo {
            id: None,
            debug_mode: false,
            eval: false
        };
        let mut first = true;
        for arg in env::args() {
            if first {first = false; continue;}
            if is_option(&arg) {
                if arg == "-d" {info.debug_mode = true;}
                else if arg == "-e" {info.eval = true;}
            } else {
                info.id = Some(arg);
            }
        }
        info
    }
}

static HELP_MESSAGE: &str = "
Usage: mossc file
";

fn main_result() -> Result<(), ()> {
    let info = CmdInfo::new();
    if let Some(id) = &info.id {
        let path = format!("{}.moss", id);
        let s = match read_file(&path) {
            Ok(s) => s,
            Err(_) => {
                println!("Error: Could not open file '{}'.", path);
                return Err(());
            }
        };
        match compile(&s, &id, &info) {
            Ok(()) => {/* pass */},
            Err(e) => {
                println!("{}", e);
                return Err(());
            }
        }
        if info.eval {
            let _ = std::process::Command::new("moss")
                .arg(format!("{}.bin", id)).status();
        }
    } else {
        println!("{}", HELP_MESSAGE);
        return Err(());
    }
    Ok(())
}

fn main()  {
    let result = match main_result() {Ok(()) => 0, Err(()) => 1};
    std::process::exit(result);
}
