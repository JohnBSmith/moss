
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
  VARIADIC, new_module, downcast
};
use vm::{Env,interface_types_set,interface_index};

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
    base: isize,
    s: Box<[ShapeStride]>,
    data: Rc<RefCell<Data>>
}

impl Array{
    fn vector_f64(v: Vec<f64>) -> Rc<Array> {
        let shape = v.len();
        let data = Rc::new(RefCell::new(Data::F64(
            v.into_boxed_slice()
        )));
        Rc::new(Array{
            s: Box::new([ShapeStride{shape, stride: 1}]),
            n: 1, base: 0, data: data
        })
    }
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
    fn add(&self, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            match *self.data.borrow() {
                Data::F64(ref data) => map_binary_f64(self,data,b,&|x,y| x+y),
                _ => panic!()
            }
        }else{
            panic!()
        }
    }
    fn sub(&self, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            match *self.data.borrow() {
                Data::F64(ref data) => map_binary_f64(self,data,b,&|x,y| x-y),
                _ => panic!()
            }
        }else{
            panic!()
        }
    }
    fn rmpy(&self, a: &Object, env: &mut Env) -> FnResult {
        let r = match *a {
            Object::Int(x) => x as f64,
            Object::Float(x) => x,
            _ => return env.type_error(
                "Type error in r*v: r has to be of type Int or Float.")
        };
        match *self.data.borrow() {
            Data::F64(ref data) => map_unary_f64(self,data,&|x| r*x),
            _ => panic!()
        }        
    }
    fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
        if self.n==1 && indices.len()==1 {
            match indices[0] {
                Object::Int(i) => {
                    let i = i as isize;
                    if i>=0 && (i as usize)<self.s[0].shape {
                        match *self.data.borrow_mut() {
                            Data::F64(ref data) => {
                                let index = (self.base as isize+self.s[0].stride*i) as usize;
                                return Ok(Object::Float(data[index]));
                            },
                            _ => panic!()
                        }
                    }else{
                        return env.index_error("Index error in a[i]: out of bounds.");
                    }
                },
                ref i => {
                    return env.type_error1("Type error in a[i]: i is not an integer.","i",i);
                }
            }
        }else{
            panic!()
        }
    }
}

fn map_binary_f64(a: &Array, adata: &[f64], b: &Array,
    f: &Fn(f64,f64)->f64
) -> FnResult
{
    if a.n==1 {
        let astride = a.s[0].stride;
        let ashape = a.s[0].shape;
        let abase = a.base;
        let mut v: Vec<f64> = Vec::with_capacity(ashape);

        if b.n != 1 {
            panic!()
        }
        if ashape != b.s[0].shape {
            panic!()
        }
        let bstride = b.s[0].stride;
        let bbase = b.base;
        let bdata_borrow = b.data.borrow();
        let bdata = match *bdata_borrow {
            Data::F64(ref data) => data, _ => panic!()
        };
        for i in 0..ashape {
            let aindex = (abase+(i as isize)*astride) as usize;
            let bindex = (bbase+(i as isize)*bstride) as usize;
            v.push(f(adata[aindex],bdata[bindex]));
        }
        return Ok(Object::Interface(Array::vector_f64(v)));

    }else{
        unimplemented!();
    }
}

fn map_unary_f64(a: &Array, data: &[f64], f: &Fn(f64)->f64)
-> FnResult
{
    if a.n==1 {
        let stride = a.s[0].stride;
        let shape = a.s[0].shape;
        let base = a.base;
        let mut v: Vec<f64> = Vec::with_capacity(shape);
        for i in 0..shape {
            let index = (base+(i as isize)*stride) as usize;
            v.push(f(data[index]));
        }
        return Ok(Object::Interface(Array::vector_f64(v)));

    }else{
        unimplemented!();
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
    interface_types_set(env.rte(),interface_index::ARRAY,type_array);

    let la = new_module("la");
    {
        let mut m = la.map.borrow_mut();
        m.insert_fn_plain("vector",vector,0,VARIADIC);
    }

    return Object::Table(Rc::new(la));
}
