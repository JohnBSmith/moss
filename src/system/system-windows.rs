
use std::str;
use std::env::var;
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

static FALLBACK_PATH: &str = "C:/prog/moss/";

pub fn init_search_paths() -> List {
    let mut a: Vec<Object> = Vec::with_capacity(3);
    a.push(Object::from("./"));
    match var("HOMEPATH") {
        Ok(s) => {
            let mut path = PathBuf::from(s);
            path.push("prog/moss/");
            match path.as_path().to_str() {
                Some(s) => a.push(Object::from(s)),
                None => unreachable!()
            }
        }
        Err(_) => {}
    };
    a.push(Object::from(FALLBACK_PATH));
    return List{v: a, frozen: false};
}

