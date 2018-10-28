
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fs;
use std::io::Read;
use std::path::Path;

use object::{
    Object, List, Table, Interface, Exception, FnResult,
    new_module, downcast, interface_object_get
};
use vm::{Env,interface_types_set,interface_index};
use data::{Bytes};

struct File {
    file: RefCell<fs::File>,
    id: String
}

impl Interface for File {
    fn as_any(&self) -> &Any {self}
    fn type_name(&self) -> String {
        "File".to_string()
    }
    fn to_string(&self, _env: &mut Env) -> Result<String,Box<Exception>> {
        return Ok("file object".to_string());
    }
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        interface_object_get("File",key,env,interface_index::FILE)
    }
}

fn open(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"open")
    }
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.v.iter().collect(),
        _ => return env.type_error("Type error in open(id): id is not a string.")
    };
    if !env.rte().read_access(&file_id) {
        return env.std_exception(&format!(
            "Error in open(id): Could not open file id=='{}': permission denied.",
            file_id
        ));
    }
    let file = match fs::File::open(&file_id) {
        Ok(file) => file,
        Err(_) => return env.std_exception(
            &format!("Error in open(id): Could not open file id=='{}'.",
            file_id))
    };
    let f = File{
        file: RefCell::new(file),
        id: file_id
    };
    return Ok(Object::Interface(Rc::new(f)));
}

fn file_read(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if let Some(file) = downcast::<File>(pself) {
        match argv.len() {
            0 => {
                let mut buffer: Vec<u8> = Vec::new();
                match file.file.borrow_mut().read_to_end(&mut buffer) {
                    Ok(_) => {},
                    Err(_) => return env.std_exception(&format!(
                        "Error in f.read(): Could not read file '{}'.",
                        file.id))
                }
                return Ok(Bytes::object_from_vec(buffer));
            },
            1 => {
                let n = if let Object::Int(x) = argv[0] {
                    if x<0 {0} else {x as usize}
                }else{
                    return env.type_error("Type error in f.read(n): n is not an integer.");
                };
                let mut buffer: Vec<u8> = vec![0;n];
                match file.file.borrow_mut().read(&mut buffer) {
                    Ok(count) => {buffer.truncate(count)},
                    Err(_) => return env.std_exception(&format!(
                        "Error in f.read(n): Could not read file '{}'.",
                        file.id))
                }
                return Ok(Bytes::object_from_vec(buffer));
            },
            n => env.argc_error(n,0,1,"read")
        }
    }else{
        env.type_error("Type error in f.read(): f is not a file.")
    }
}

fn is_file(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"is_file")
    }
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.v.iter().collect(),
        _ => return env.type_error("Type error in is_file(id): id is not a string.")
    };
    let path = Path::new(&file_id);
    return Ok(Object::Bool(path.is_file()));
}

fn is_dir(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"is_dir")
    }
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.v.iter().collect(),
        _ => return env.type_error("Type error in is_dir(id): id is not a string.")
    };
    let path = Path::new(&file_id);
    return Ok(Object::Bool(path.is_dir()));
}

fn read_dir(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"read_dir")
    }
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.v.iter().collect(),
        _ => return env.type_error("Type error in is_dir(id): id is not a string.")
    };
    let path = Path::new(&file_id);
    let it = match path.read_dir() {
        Ok(it) => it,
        Err(_) => return env.std_exception(&format!(
            "Could not read directory '{}'.",file_id))
    };
    let mut v: Vec<Object> = Vec::new();
    for x in it {
        if let Ok(x) = x {
            if let Some(s) = x.path().to_str() {
                v.push(Object::from(s));
            }
        }
    }
    return Ok(List::new_object(v));
}

pub fn load_fs(env: &mut Env) -> Object
{
    let type_file = Table::new(Object::Null);
    {
        let mut m = type_file.map.borrow_mut();
        m.insert_fn_plain("read",file_read,0,1);
    }
    interface_types_set(env.rte(),interface_index::FILE,type_file);

    let fs = new_module("fs");
    {
        let mut m = fs.map.borrow_mut();
        m.insert_fn_plain("open",open,1,1);
        m.insert_fn_plain("is_file",is_file,1,1);
        m.insert_fn_plain("is_dir",is_dir,1,1);
        m.insert_fn_plain("read_dir",read_dir,1,1);
    }

    return Object::Table(Rc::new(fs));
}

