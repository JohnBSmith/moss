
use std::str;
use std::io;
use std::io::Write;

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

static PATH: &[&str] = &[
    "/usr/local/lib/moss/",
    "C:/prog/moss/"
];

pub fn init_search_paths() -> List {
    let mut a: Vec<Object> = Vec::with_capacity(3);
    a.push(Object::from("./"));

    for path in PATH {
        a.push(Object::from(*path));
    }
    return List{v: a, frozen: false};
}

