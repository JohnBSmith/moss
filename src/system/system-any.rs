
use std::str;
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
