
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
    Object, Exception, Table, FnResult, Interface,
    VARIADIC, new_module
};
use vm::{Env,interface_types_set};

const INDEX: usize = 2;

struct Bytes {
    data: RefCell<Vec<u8>>
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

pub fn load_data(env: &mut Env) -> Object
{
    let type_bytes = Table::new(Object::Null);
    interface_types_set(env,INDEX,type_bytes);

    let data = new_module("data");
    {
        let mut m = data.map.borrow_mut();
        m.insert_fn_plain("bytes",bytes,1,1);
    }

    return Object::Table(Rc::new(data));
}
