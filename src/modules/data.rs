
// Byte buffers

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt::Write;

use crate::object::{
    Object, Exception, FnResult, Interface, List,
    new_module, downcast, interface_object_get,
    ptr_eq_plain
};
use crate::vm::{Env, RTE, interface_index, interface_types_set};
use crate::iterable::new_iterator;
use crate::class::Class;
use crate::range::Range;

mod crypto;

pub struct Bytes {
    pub data: RefCell<Vec<u8>>
}

impl Bytes {
    pub fn object_from_vec(v: Vec<u8>) -> Object {
        Object::Interface(Rc::new(Bytes {data: RefCell::new(v)}))
    }
}

impl Interface for Bytes {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "Bytes".to_string()
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        let mut s = "bytes([".to_string();
        let mut first = true;
        for x in self.data.borrow().iter() {
            if first {
                first = false;
                write!(s, "{}", *x).unwrap();
            } else {
                write!(s, ", {}", *x).unwrap();
            }
        }
        write!(s, "])").unwrap();
        Ok(s)
    }
    fn is_instance_of(&self, type_obj: &Object, rte: &RTE) -> bool {
        if let Object::Interface(p) = type_obj {
            ptr_eq_plain(p,&rte.interface_types.borrow()[interface_index::BYTES]) ||
            ptr_eq_plain(p,&rte.type_iterable)
        } else {
            false
        }
    }
    fn get_type(&self, env: &mut Env) -> FnResult {
        Ok(Object::Interface(env.rte().interface_types
           .borrow()[interface_index::BYTES].clone()))
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        interface_object_get("Bytes", key, env, interface_index::BYTES)
    }
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        let a = self.data.borrow();
        match indices.len() {
            1 => {}, n => return env.argc_error(n,1,1,"index operation")
        }
        let index = match indices[0] {
            Object::Int(index) => {
                if index < 0 {
                    return env.index_error("Index error in a[i]: i is out of lower bound.");
                } else {
                    let index = index as usize;
                    if index >= a.len() {
                        return env.index_error("Index error in a[i]: i is out of upper bound.");
                    }
                    index
                }
            },
            _ => {
                if let Some(range) = downcast::<Range>(&indices[0]){
                    return index_range(&a,range,env);
                } else {
                    return env.type_error("Type error in a[i]: is not an integer.")
                }
            }
        };
        Ok(Object::Int(a[index] as i32))
    }
    fn iter(self: Rc<Self>, _env: &mut Env) -> FnResult {
        let mut index: usize = 0;
        let a = self.data.clone();
        let f = Box::new(move |_env: &mut Env, _pself: &Object, _argv: &[Object]| -> FnResult {
            let a = a.borrow();
            Ok(if index == a.len() {
                Object::empty()
            } else {
                index += 1;
                Object::Int(a[index-1] as i32)
            })
        });
        Ok(new_iterator(f))
    }
    fn add(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Bytes>(b) {
            let data1 = self.data.borrow();
            let data2 = b.data.borrow();
            let l1 = data1.len();
            let l2 = data2.len();
            let mut acc: Vec<u8> = Vec::with_capacity(l1 + l2);
            acc.extend_from_slice(&data1);
            acc.extend_from_slice(&data2);
            Ok(Bytes::object_from_vec(acc))
        } else {
            env.type_error1(
                "Type error in a+b: expected b of type Bytes", "b", b)
        }
    }
}

fn index_range(a: &[u8], r: &Range, env: &mut Env) -> FnResult {
    let len = a.len();
    let i = match r.a {
        Object::Int(value) => {
            if value<0 {0} else {value as usize}
        },
        Object::Null => 0,
        _ => return env.type_error1("Type error in a[i..j]: i is not an integer.", "i", &r.a)
    };
    let j = match r.b {
        Object::Int(value) => {
            if value<0 {0}
            else if value >= len as i32 {len}
            else {value as usize + 1}
        },
        Object::Null => len,
        _ => return env.type_error1("Type error in a[i..j]: j is not an integer.","j",&r.a)
    };
    let v = if j<i {Vec::new()} else {a[i..j].to_vec()};
    Ok(Bytes::object_from_vec(v))
}

fn bytes(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"bytes")
    }
    if let Object::List(ref a) = argv[0] {
        let v = &a.borrow().v;
        let mut acc: Vec<u8> = Vec::with_capacity(v.len());
        for x in v {
            if let Object::Int(x) = *x {
                if 0 <= x && x < 256 {
                    acc.push(x as u8);
                } else {
                    return env.value_error(
                    "Value error in bytes(a): a[i] is out of range 0..255.");
                }
            } else {
                return env.value_error(
                "Value error in bytes(a): a[i] is not an integer.");
            }
        }
        let data = RefCell::new(acc);
        Ok(Object::Interface(Rc::new(Bytes {data})))
    } else {
        env.type_error("Type error in bytes(a): a is not a list.")
    }
}

pub fn bytes_list(env: &mut Env, pself: &Object, _argv: &[Object]) -> FnResult {
    if let Some(bytes) = downcast::<Bytes>(pself) {
        let a = bytes.data.borrow();
        let mut acc: Vec<Object> = Vec::with_capacity(a.len());
        for &x in a.iter() {
            acc.push(Object::Int(x as i32));
        }
        Ok(List::new_object(acc))
    } else {
        env.type_error("Type error in a.list(): a is not of type Bytes.")
    }
}

pub fn bytes_decode(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    let spec: String = match argv.len() {
        0 => String::from("utf-8"),
        1 => match argv[0] {
            Object::String(ref s) => s.to_string(),
            _ => return env.type_error1(
                "Type error in a.decode(spec): spec is not a string.",
                "spec",&argv[0])
        },
        n => return env.argc_error(n,1,1,"decode")
    };
    if let Some(bytes) = downcast::<Bytes>(pself) {
        let a = bytes.data.borrow();
        if spec=="utf-8" {
            Ok(Object::from(&*String::from_utf8_lossy(&a)))
        } else {
            env.value_error(&format!(
                "Value error in a.decode(spec): spec=='{}' is unknown.", spec))
        }
    } else {
        env.type_error("Type error in a.decode(spec): a is not a of type Bytes.")
    }
}

pub fn bytes_len(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"len")
    }
    if let Some(data) = downcast::<Bytes>(pself) {
        Ok(Object::Int(data.data.borrow().len() as i32))
    } else {
        env.type_error("Type error in a.len(): a is not of type Bytes.")
    }
}

pub fn bytes_hex(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"hex")
    }
    if let Some(data) = downcast::<Bytes>(pself) {
        Ok(base16(&data.data.borrow()))
    } else {
        env.type_error("Type error in a.hex(): a is not of type Bytes.")
    }
}

struct Hash {
    state: RefCell<Option<crypto::sha3::Keccak>>
}
impl Hash {
    fn new() -> Self {
        Self{state: RefCell::new(Some(crypto::sha3::Keccak::new_sha3_256()))}
    }
}

impl Interface for Hash {
    fn as_any(&self) -> &dyn Any {self}
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        interface_object_get("Hash",key,env,interface_index::HASH)
    }
    fn to_string(self: Rc<Self>, _env: &mut Env) -> Result<String,Box<Exception>> {
        Ok(String::from("hash object"))
    }
}

fn data_hash(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        0 => {}, n => return env.argc_error(n,0,0,"hash")
    }
    Ok(Object::Interface(Rc::new(Hash::new())))
}

fn hash_push(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    if let Some(hash) = downcast::<Hash>(pself) {
        if let Some(data) = downcast::<Bytes>(&argv[0]) {
            let mut state = hash.state.borrow_mut();
            Ok(if let Some(state) = state.as_mut() {
                state.update(&data.data.borrow());
                pself.clone()
            } else {
                Object::Null
            })
        } else {
            env.type_error1(
               "Type error in h.push(a): a is not a of type Bytes.",
               "a", &argv[0])
        }
    } else {
        env.type_error1(
            "Type error in h.push(a): h is not of type Hash.",
            "h", pself)
    }
}

fn hash_value(env: &mut Env, pself: &Object, _argv: &[Object]) -> FnResult {
    if let Some(hash) = downcast::<Hash>(pself) {
        let mut value: Vec<u8> = vec![0; 32];
        Ok(if let Some(state) = hash.state.borrow_mut().take() {
            state.finalize(&mut value);
            Bytes::object_from_vec(value)
        } else {
            Object::Null
        })
    } else {
        env.type_error1(
            "Type error in h.value(): h is not of type Hash.",
            "h", pself)
    }
}

pub fn base16(data: &[u8]) -> Object {
    let mut buffer: String = String::new();
    for byte in data {
        write!(&mut buffer,"{:02x}", byte).unwrap();
    }
    Object::from(&buffer[..])
}

pub fn load_data(env: &mut Env) -> Object
{
    let type_hash = Class::new("Hash", &Object::Null);
    {
        let mut m = type_hash.map.borrow_mut();
        m.insert_fn_plain("push", crate::data::hash_push, 1, 1);
        m.insert_fn_plain("value", crate::data::hash_value, 0, 0);
    }
    interface_types_set(env.rte(),interface_index::HASH,type_hash.clone());

    let data = new_module("data");
    {
        let mut m = data.map.borrow_mut();
        m.insert_fn_plain("bytes", bytes, 1, 1);
        m.insert_fn_plain("hash", data_hash, 1, 1);

        let type_bytes = env.rte().interface_types
            .borrow()[interface_index::BYTES].clone();
        m.insert("Bytes", Object::Interface(type_bytes));
        m.insert("Hash", Object::Interface(type_hash));
    }

    Object::Interface(Rc::new(data))
}
