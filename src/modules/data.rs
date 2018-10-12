
// Byte buffers

#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt::Write;

use complex::c64;
use object::{
    Object, Exception, Table, FnResult, Interface, List,
    VARIADIC, new_module, interface_object_get
};
use vm::{Env,interface_index};
use iterable::new_iterator;

pub struct Bytes {
    pub data: RefCell<Vec<u8>>
}

impl Bytes {
    pub fn object_from_vec(v: Vec<u8>) -> Object {
        return Object::Interface(Rc::new(Bytes{data: RefCell::new(v)}))
    }
    fn downcast(x: &Object) -> Option<&Bytes> {
        if let Object::Interface(ref a) = *x {
            a.as_any().downcast_ref::<Bytes>()
        }else{
            None
        }
    }
}

impl Interface for Bytes {
    fn as_any(&self) -> &Any {self}
    fn type_name(&self) -> String {
        "Bytes".to_string()
    }
    fn to_string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
        let mut s = "bytes([".to_string();
        let mut first = true;
        for x in self.data.borrow().iter() {
            if first {
                first = false;
                write!(s,"{}",*x).unwrap();
            }else{
                write!(s,", {}",*x).unwrap();
            }
        }
        write!(s,"])").unwrap();
        return Ok(s);
    }
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        interface_object_get("Bytes",key,env,interface_index::BYTES)
    }
    fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
        let a = self.data.borrow();
        let index = if indices.len()==1 {
            match indices[0] {
                Object::Int(index) => {
                    if index<0 {
                        return env.index_error("Index error in a[i]: i is out of lower bound.");
                    } else {
                        let index = index as usize;
                        if index>=a.len() {
                            return env.index_error("Index error in a[i]: i is out of upper bound.");
                        }
                        index
                    }
                },
                _ => return env.type_error("Type error in a[i]: is not an integer.")
            }
        }else{
            panic!()
        };
        return Ok(Object::Int(a[index] as i32));
    }
    fn iter(&self, env: &mut Env) -> FnResult {
        let mut index: usize = 0;
        let a = self.data.clone();
        let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
            let a = a.borrow();
            if index == a.len() {
                return Ok(Object::Empty);
            }else{
                index+=1;
                return Ok(Object::Int(a[index-1] as i32));
            }
        });
        Ok(new_iterator(f))
    }
}

fn bytes(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"bytes")
    }
    if let Object::List(ref a) = argv[0] {
        let v = &a.borrow().v;
        let mut data: Vec<u8> = Vec::with_capacity(v.len());
        for x in v {
            if let Object::Int(x) = *x {
                if 0<=x && x<256 {
                    data.push(x as u8);
                }else{
                    return env.value_error(
                    "Value error in bytes(a): a[i] is out of range 0..255.");
                }
            }else{
                return env.value_error(
                "Value error in bytes(a): a[i] is not an integer.");
            }
        }
        return Ok(Object::Interface(Rc::new(Bytes{
            data: RefCell::new(data)
        })));
    }else{
        return env.type_error("Type error in bytes(a): a is not a list.");
    }
}

pub fn bytes_list(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if let Some(bytes) = Bytes::downcast(pself) {
        let a = bytes.data.borrow();
        let mut v: Vec<Object> = Vec::with_capacity(a.len());
        for &x in a.iter() {
            v.push(Object::Int(x as i32));
        }
        return Ok(List::new_object(v));
    }else{
        env.type_error("Type error in f.read(): f is not a file.")    
    }
}

pub fn load_data(env: &mut Env) -> Object
{
    let data = new_module("data");
    {
        let mut m = data.map.borrow_mut();
        m.insert_fn_plain("bytes",bytes,1,1);
    }
    return Object::Table(Rc::new(data));
}
