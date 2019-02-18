
extern crate libc;
use std::str;
use std::env::var;
use std::fs::File;
use std::io::Read;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use object::{Object,List};

pub struct History{}

impl History {
    pub fn new() -> Self {Self{}}
    pub fn append(&mut self, _s: &str){}
}

pub fn getline(prompt: &str) -> io::Result<String> {
    print!("{}",prompt);
    io::stdout().flush().ok();
    let mut input = String::new();
    return match io::stdin().read_line(&mut input) {
        Ok(_) => Ok(input),
        Err(x) => Err(x)
    };
}

pub fn getline_history(prompt: &str, _history: &History) -> io::Result<String> {
    return getline(prompt);
}

static FALLBACK_PATH: &str = "C:/prog/";

pub fn init_search_paths() -> List {
    let mut a: Vec<Object> = Vec::new();
    a.push(Object::from("./"));
    let mut path = match var("HOMEPATH") {
        Ok(s) => PathBuf::from(s),
        Err(_) => PathBuf::from(FALLBACK_PATH)
    };
    path.push(".moss/");
    match path.as_path().to_str() {
        Some(s) => a.push(Object::from(s)),
        None => unreachable!()
    }
    return List{v: a, frozen: false};
}

pub fn read_module_file(search_paths: &[Object], id: &str)
-> Result<String,String>
{
    for path_obj in search_paths {
        let mut path = match path_obj {
            Object::String(ref s) => PathBuf::from(s.to_string()),
            _ => return Err(String::from(
                "Error in load: search paths must be a strings."))
        };
        path.push(id);
        path.set_extension("moss");
        // println!("path: '{}'",path.to_str().unwrap());
        if let Ok(mut f) = File::open(&path) {
            let mut s = String::new();
            if let Err(_) = f.read_to_string(&mut s) {
                return Err(format!(
                    "Error in load: could not read file '{}.moss'.",id));
            }
            return Ok(s);
        }
    }
    return Err(format!("Error in load: could not open file '{}.moss'.",id));
}

pub fn read_file(id: &str) -> Result<String,String> {
    let mut f = match File::open(id) {
        Ok(f) => f,
        Err(_) => return Err(format!("Error in read: could not open file '{}'.",id))
    };
    let mut s = String::new();
    if let Err(_) = f.read_to_string(&mut s) {
        return Err(format!("Error in read: could not read file '{}'.",id));
    }
    return Ok(s);
}
