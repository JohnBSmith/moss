
use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fs;
use std::io::{Read,Write};
use std::path::Path;

use crate::object::{
    Object, List, Interface, Exception, FnResult,
    new_module, downcast, interface_object_get
};
use crate::vm::{Env,interface_types_set,interface_index};
use crate::data::{Bytes};
use crate::class::Class;

struct File {
    file: RefCell<fs::File>,
    id: String
}

impl Interface for File {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "File".to_string()
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        return Ok("file object".to_string());
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        interface_object_get("File",key,env,interface_index::FILE)
    }
}

fn open(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let mode = match argv.len() {
        1 => {'r'},
        2 => {
            match argv[1] {
                Object::String(ref s) => {
                    if s.data == ['w'] {'w'} else {'r'}
                },
                ref x => return env.type_error1(
                    "Type error in open(path,mode): mode is not a string.","mode",x)
            }
        },
        n => return env.argc_error(n,1,2,"open")
    };
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.to_string(),
        ref x => return env.type_error1(
            "Type error in open(path): path is not a string.","path",x)
    };
    let open_result = if mode=='r' {
        if !env.rte().read_access(&file_id) {
            return env.std_exception(&format!(
                "Error in open(path): permission denied.\n\
                 Note: path = '{}'", file_id
            ));
        }
        fs::File::open(&file_id)
    }else{
        if !env.rte().write_access(&file_id) {
            return env.std_exception(&format!(
                "Error in open(path,'w'): permission denied.\n\
                 Note: path = '{}'.", file_id
            ));
        }
        fs::File::create(&file_id)
    };
    let file = match open_result {
        Ok(file) => file,
        Err(_) => return env.std_exception(
            &format!("Error in open(path): could not open file, path = '{}'.",
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

fn file_write(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"write")
    }
    if let Some(file) = downcast::<File>(pself) {
        if let Object::String(ref s) = argv[0] {
            let data = &s.to_string().into_bytes();
            if let Ok(()) = file.file.borrow_mut().write_all(data) {
                return Ok(Object::Null);
            }
        }else if let Some(a) = downcast::<Bytes>(&argv[0]) {
            let data = &a.data.borrow();
            if let Ok(()) = file.file.borrow_mut().write_all(data) {
                return Ok(Object::Null);
            }
        }else{
            return env.type_error1(
                "Type error in f.write(data): data must be a string or binary data.",
                 "data",&argv[0]);
        }
        env.std_exception(&format!(
            "Error in f.write(data): failed to write to file '{}'.",file.id))
    }else{
        env.type_error("Type error in f.write(data): f is not a file.")
    }
}

fn is_file(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"is_file")
    }
    let file_id: String = match argv[0] {
        Object::String(ref s) => s.to_string(),
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
        Object::String(ref s) => s.to_string(),
        _ => return env.type_error("Type error in is_dir(id): id is not a string.")
    };
    let path = Path::new(&file_id);
    return Ok(Object::Bool(path.is_dir()));
}

fn read_dir(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    let file_id: String = match argv.len() {
        0 => String::from("."),
        1 => match argv[0] {
            Object::String(ref s) => s.to_string(),
            _ => return env.type_error(
                "Type error in ls(path): path is not a string.")
        },
        n => return env.argc_error(n,0,1,"ls")
    };
    let path = Path::new(&file_id);
    let it = match path.read_dir() {
        Ok(it) => it,
        Err(_) => return env.std_exception(&format!(
            "Error in ls(path): Could not read directory '{}'.",file_id))
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

fn change_dir(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"cd")
    }
    let dir_id: String = match argv[0] {
        Object::String(ref s) => s.to_string(),
        _ => return env.type_error("Type error in cd(path): path is not a string.")
    };
    let path = Path::new(&dir_id);
    if std::env::set_current_dir(&path).is_err() {
        return env.std_exception(&format!(
            "Error: could not change to directory '{}'.",dir_id));
    }else{
        return Ok(Object::Null);
    }
}

fn working_dir(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"wd")
    }
    let path = match std::env::current_dir() {
        Ok(path) => path,
        Err(_) => return env.std_exception(
            "Error in wd(): could not determine the working directory.")
    };
    if let Some(s) = path.to_str() {
        Ok(Object::from(s))
    }else{
        env.std_exception(
            "Error in wd(): could not encode the working directory as UTF-8.")
    }
}

pub fn load_fs(env: &mut Env) -> Object
{
    let type_file = Class::new("File",&Object::Null);
    {
        let mut m = type_file.map.borrow_mut();
        m.insert_fn_plain("read",file_read,0,1);
        m.insert_fn_plain("write",file_write,1,1);
    }
    interface_types_set(env.rte(),interface_index::FILE,type_file);

    let fs = new_module("fs");
    {
        let mut m = fs.map.borrow_mut();
        m.insert_fn_plain("open",open,1,2);
        m.insert_fn_plain("is_file",is_file,1,1);
        m.insert_fn_plain("is_dir",is_dir,1,1);
        m.insert_fn_plain("ls",read_dir,1,1);
        m.insert_fn_plain("cd",change_dir,1,1);
        m.insert_fn_plain("wd",working_dir,0,0);
    }

    return Object::Interface(Rc::new(fs));
}

