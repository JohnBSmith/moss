
// Linear algebra

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

const INDEX: usize = 1;

struct ShapeStride{
    shape: usize,
    stride: isize
}

enum Data{
    F64(Box<[f64]>),
    C64(Box<[c64]>)
}

struct Array{
    n: usize,
    base: usize,
    s: Box<[ShapeStride]>,
    data: Rc<RefCell<Data>>
}

impl Interface for Array {
    fn as_any(&self) -> &Any {self}
    fn type_name(&self) -> String {
        "Array".to_string()
    }
    fn to_string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
        if self.n==1 {
            let mut s = "vector(".to_string();
            let data = self.data.borrow();
            match *data {
                Data::F64(ref data) => {
                    vector_f64_to_string(self,&mut s,data);
                },
                _ => unimplemented!()
            }
            s.push_str(")");
            return Ok(s);
        }else{
            unimplemented!();
        }
    }
}

fn vector_f64_to_string(a: &Array, s: &mut String, data: &[f64]) {
    let mut first = true;
    let stride = a.s[0].stride;
    let base = a.base as isize;
    for i in 0..a.s[0].shape {
        if first {first = false;}
        else {s.push_str(", ");}
        let x = data[(base+i as isize*stride) as usize];
        write!(s,"{}",x).ok();
    }
}

fn vector_from_list(env: &mut Env, a: &[Object]) -> FnResult {
    let mut v: Vec<f64> = Vec::with_capacity(a.len());
    for x in a {
        let x = match *x {
            Object::Int(x) => x as f64,
            Object::Float(x) => x,
            _ => return env.type_error(
                "Type error in vector(*a): expected a[k] of type Int or Float.")
        };
        v.push(x);
    }
    let data = Rc::new(RefCell::new(Data::F64(
        v.into_boxed_slice()
    )));
    return Ok(Object::Interface(Rc::new(Array{
        s: Box::new([ShapeStride{shape: a.len(), stride: 1}]),
        n: 1, base: 0, data: data
    })));
}

fn vector(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
    return vector_from_list(env,argv);
}

pub fn load_la(env: &mut Env) -> Object
{
    let type_array = Table::new(Object::Null);
    /*{
        let mut m = type_array.map.borrow_mut();
        m.insert_fn_plain("map",map,1,1);
    }*/
    interface_types_set(env,INDEX,type_array);

    let la = new_module("la");
    {
        let mut m = la.map.borrow_mut();
        m.insert_fn_plain("vector",vector,0,VARIADIC);
    }

    return Object::Table(Rc::new(la));
}
