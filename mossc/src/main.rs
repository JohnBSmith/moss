
use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::env;

mod parser;
mod typing;
mod generator;
mod system;
mod debug;

use parser::{parse, Error};
use typing::{TypeChecker, TypeTable};
use generator::generate;
use system::save;

fn print_error(e: &Error, file: &str) {
    print!("File '{}', line {}, col {}:\n", file, e.line + 1, e.col + 1);
    println!("{}", e.text);
}

fn compile(s: &str, file: &str, info: &CmdInfo) -> Result<(),()> {
    let debug_mode = info.debug_mode;
    let t = match parse(s) {
        Ok(t) => t,
        Err(e) => {
            print_error(&e, file);
            return Err(());
        }
    };
    if debug_mode {
        println!("{}", &t);
    }

    let tab = TypeTable::new();
    let mut checker = TypeChecker::new(&tab);
    match checker.type_check(&t) {
        Ok(()) => {},
        Err(e) => {
            e.print();
            return Err(());
        }
    }
    if debug_mode {
        println!("{}", checker.string(&t));
        println!("{}", checker.subs_as_string());
    }
    checker.apply_types();
    match checker.check_constraints() {
        Ok(()) => {},
        Err(e) => {
            e.print();
            return Err(());
        }
    }
    if debug_mode {
        println!("{}", checker.string(&t));
    }
    let code = generate(&t, checker.symbol_table);
    if debug_mode {
        println!("{}", code);
    }
    save(&code, file);
    Ok(())
}

fn read_file(path: &str) -> Result<String,std::io::Error> {
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
    debug_mode: bool
}

impl CmdInfo {
    pub fn new() -> Self {
        let mut info = CmdInfo {id: None, debug_mode: false};
        let mut first = true;
        for arg in env::args() {
            if first {first = false; continue;}
            if is_option(&arg) {
                if arg == "-d" {info.debug_mode = true;}
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

fn main_result() -> Result<(),()> {
    let info = CmdInfo::new();
    if let Some(id) = &info.id {
        let path = format!("{}.moss",id);
        let s = match read_file(&path) {
            Ok(s) => s,
            Err(_) => {
                println!("Error: Could not open file '{}'.", path);
                return Err(());
            }
        };
        compile(&s, &id, &info)?;
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
