
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::object::Object;

#[cfg(not(windows))]
#[path = "system-unix.rs"]
mod system_os;

#[cfg(windows)]
#[path = "system-windows.rs"]
mod system_os;

pub use self::system_os::*;

pub fn open_module_file(search_paths: &[Object], id: &str)
-> Result<(File,bool),String>
{
    for path_obj in search_paths {
        let mut path = match path_obj {
            Object::String(ref s) => PathBuf::from(s.to_string()),
            _ => return Err(String::from(
                "Error in load: search paths must be a strings."))
        };

        let mut bin_path = path.clone();
        bin_path.push("bin/");
        bin_path.push(id);
        bin_path.set_extension("bin");
        if let Ok(f) = File::open(&bin_path) {
            return Ok((f,true));
        }

        path.push(id);
        path.set_extension("moss");
        if let Ok(f) = File::open(&path) {
            return Ok((f,false));
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

pub fn library_path() -> String {
    let mut path = match std::env::var("HOME") {
        Ok(s) => PathBuf::from(s),
        Err(_) => panic!()
    };
    path.push(".moss/");
    return String::from(path.to_str().unwrap());
}
