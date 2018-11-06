
// Linear algebra

// Per default, matrices are stored in row major. The shape order
// of Box<[ShapeStride]> is the same as the index order.

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt::Write;

use complex::c64;
use object::{
  Object, List, Exception, Table, FnResult, Interface,
  VARIADIC, new_module, downcast
};
use vm::{Env,interface_types_set,interface_index};

struct ShapeStride{
    shape: usize,
    stride: isize
}

#[allow(dead_code)]
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
    fn matrix_f64(m: usize, n: usize, a: Vec<f64>) -> Rc<Array> {
        let data = Rc::new(RefCell::new(Data::F64(
            a.into_boxed_slice()
        )));
        Rc::new(Array{
            s: Box::new([
                ShapeStride{shape: m, stride: n as isize},
                ShapeStride{shape: n, stride: 1}
            ]),
            n: 2, base: 0, data: data,
        })
    }
}

impl Interface for Array {
    fn as_any(&self) -> &Any {self}
    fn type_name(&self) -> String {
        "Array".to_string()
    }
    fn to_string(&self, _env: &mut Env) -> Result<String,Box<Exception>> {
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
        }else if self.n==2 {
            let mut s = "matrix(\n".to_string();
            let data = self.data.borrow();            
            match *data {
                Data::F64(ref data) => {
                    matrix_f64_to_string(self,&mut s,data);
                },
                _ => unimplemented!()
            }
            s.push_str("\n)");
            return Ok(s);
        }else{
            unimplemented!();
        }
    }
    fn add(&self, b: &Object, _env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            match *self.data.borrow() {
                Data::F64(ref data) => map_binary_f64(self,data,b,&|x,y| x+y),
                _ => panic!()
            }
        }else{
            panic!()
        }
    }
    fn sub(&self, b: &Object, _env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            match *self.data.borrow() {
                Data::F64(ref data) => map_binary_f64(self,data,b,&|x,y| x-y),
                _ => panic!()
            }
        }else{
            panic!()
        }
    }
    fn mul(&self, b: &Object, env: &mut Env) -> FnResult {
        if let Some(b) = downcast::<Array>(b) {
            match *self.data.borrow() {
                Data::F64(ref adata) => {
                    match *b.data.borrow() {
                        Data::F64(ref bdata) => mul_f64(env,self,b,adata,bdata),
                        _ => panic!()
                    }
                },
                _ => panic!()
            }
        }else{
            panic!()
        }
    }
    fn rmul(&self, a: &Object, env: &mut Env) -> FnResult {
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
    fn get(&self, key: &Object, env: &mut Env) -> FnResult {
        if let Object::String(ref s) = *key {
            let v = &s.data;
            match v.len() {
                3 => {
                    if v[0..3] == ['a','b','s'] {
                        return abs(self);
                    }
                },
                5 => {
                    if v[0..5] == ['s','h','a','p','e'] {
                        return shape(self);
                    }
                },
                _ => {}
            }
            let t = &env.rte().interface_types.borrow()[interface_index::ARRAY];
            match t.get(key) {
                Some(value) => return Ok(value),
                None => {
                    env.index_error(&format!(
                        "Index error in Array.{0}: {0} not found.", key
                    ))
                }
            }
        }else{
            env.type_error("Type error in Array.x: x is not a string.")
        }
    }
    fn abs(&self, _env: &mut Env) -> FnResult {
        return abs(self);
    }
}

fn shape(a: &Array) -> FnResult {
    let n = a.s.len();
    let mut v: Vec<Object> = Vec::with_capacity(n);
    for i in 0..n {
        v.push(Object::Int(a.s[i].shape as i32));
    }
    return Ok(List::new_object(v));
}

fn abs(a: &Array) -> FnResult {
    let n = a.s[0].shape;
    let stride = a.s[0].stride;
    let base = a.base;
    match *a.data.borrow() {
        Data::F64(ref data) => {
            let mut y = 0.0;
            for i in 0..n {
                let x = data[(base+i as isize*stride) as usize];
                y += x*x;
            }
            return Ok(Object::Float(y.sqrt()));
        },
        _ => panic!()
    }
}

fn mul_f64(env: &mut Env,
    a: &Array, b: &Array, adata: &[f64], bdata: &[f64]
) -> FnResult
{
    if a.n == 2 {
        if b.n == 2 {
            let m = a.s[0].shape;
            let p = a.s[1].shape;
            let n = b.s[1].shape;
            if p != b.s[0].shape {
                return env.value_error(
                    "Value error in matrix multiplication A*B:\n  A.shape[1] != B.shape[0].");
            }
            Ok(Object::Interface(
                mul_matrix_matrix_f64(m,p,n,a,b,adata,bdata)))
        }else if b.n ==1 {
            let m = a.s[0].shape;
            let n = a.s[1].shape;
            if n != b.s[0].shape {
                return env.value_error(
                    "Value error in A*b: A.shape[1] != b.shape[0].");
            }
            Ok(Object::Interface(
                mul_matrix_vector_f64(m,n,a,b,adata,bdata)))
        }else{
            panic!()
        }
    }else if a.n == 1 {
        let n = a.s[0].shape;
        if n != b.s[0].shape {
            return env.value_error(
                "Value error in a*b: a.shape[0] != b.shape[0].");
        }
        Ok(Object::Float(
            scalar_product_f64(n,a,b,adata,bdata)))
    }else{
        panic!()
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

fn scalar_product_f64(n: usize,
    a: &Array, b: &Array, adata: &[f64], bdata: &[f64]
) -> f64
{
    let abase = a.base;
    let bbase = b.base;
    let astride = a.s[0].stride;
    let bstride = b.s[0].stride;
    let mut y = 0.0;
    for i in 0..n {
        let aindex = abase+i as isize*astride;
        let bindex = bbase+i as isize*bstride;
        y += adata[aindex as usize]*bdata[bindex as usize];
    }
    return y;
}

fn mul_matrix_vector_f64(m: usize, n: usize,
    a: &Array, b: &Array, adata: &[f64], bdata: &[f64]
) -> Rc<Array>
{
    let abase = a.base;
    let bbase = b.base;
    let aistride = a.s[0].stride;
    let ajstride = a.s[1].stride;
    let bjstride = b.s[0].stride;
    let mut v: Vec<f64> = Vec::with_capacity(m);
    for i in 0..m {
        let mut y = 0.0;
        for j in 0..n {
            let aindex = abase+i as isize*aistride+j as isize*ajstride;
            let bindex = bbase+j as isize*bjstride;
            y += adata[aindex as usize]*bdata[bindex as usize];
        }
        v.push(y);
    }
    return Array::vector_f64(v);
}

fn mul_matrix_matrix_f64(m: usize, p: usize, n: usize,
    a: &Array, b: &Array, adata: &[f64], bdata: &[f64]
) -> Rc<Array>
{
    let abase = a.base;
    let bbase = b.base;
    let aistride = a.s[0].stride;
    let akstride = a.s[1].stride;
    let bkstride = b.s[0].stride;
    let bjstride = b.s[1].stride;
    let mut v: Vec<f64> = Vec::with_capacity(m*n);
    for i in 0..m {
        for j in 0..n {
            let mut y = 0.0;
            for k in 0..p {
                let aindex = abase+i as isize*aistride+k as isize*akstride;
                let bindex = bbase+k as isize*bkstride+j as isize*bjstride;
                y += adata[aindex as usize]*bdata[bindex as usize];
            }
            v.push(y);
        }
    }
    return Array::matrix_f64(m,n,v);
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

fn matrix_f64_to_string(a: &Array, s: &mut String, data: &[f64]) {
    let istride = a.s[0].stride;
    let jstride = a.s[1].stride;
    let m = a.s[0].shape;
    let n = a.s[1].shape;
    let base = a.base as isize;
    let mut ifirst = true;
    for i in 0..m {
        if ifirst {ifirst = false;} else {s.push_str(",\n");}
        write!(s,"  [").ok();
        let mut jfirst = true;
        for j in 0..n {
            if jfirst {jfirst = false;} else {s.push_str(", ");}
            let index = base+i as isize*istride+j as isize*jstride;
            let x = data[index as usize];
            write!(s,"{}",x).ok();
        }
        write!(s,"]").ok();
    }
}

fn vector_from_list(env: &mut Env, a: &[Object]) -> FnResult {
    let mut v: Vec<f64> = Vec::with_capacity(a.len());
    for x in a {
        let x = match *x {
            Object::Int(x) => x as f64,
            Object::Float(x) => x,
            _ => return env.type_error(
                "Type error in vector(*a): expected all a[k] of type Int or Float.")
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

fn matrix_from_lists(env: &mut Env, argv: &[Object]) -> FnResult {
    let m = argv.len();
    if m==0 {
        return env.value_error(
            "Value error in matrix(*a): expected at least one row.");
    }
    let n = match argv[0] {
        Object::List(ref a) => a.borrow().v.len(),
        _ => panic!()
    };
    let mut v: Vec<f64> = Vec::with_capacity(m*n);
    for x in argv {
        let a = match *x {
            Object::List(ref a) => a,
            _ => panic!()
        };
        let data = &a.borrow_mut().v;
        if data.len() != n {
            return env.value_error(
                "Value error in matrix(*a): all a[i] must be of the same size.");
        }
        for x in data {
            let x = match *x {
                Object::Int(x) => x as f64,
                Object::Float(x) => x,
                _ => return env.type_error(
                    "Type error in matrix(*a): expected all a[i][j] of type Int or Float.")
            };
            v.push(x);
        }
    }
    return Ok(Object::Interface(Array::matrix_f64(m,n,v)));
}

fn vector(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    return vector_from_list(env,argv);
}

fn matrix(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    return matrix_from_lists(env,argv);
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
        m.insert_fn_plain("matrix",matrix,1,VARIADIC);
    }

    return Object::Table(Rc::new(la));
}
