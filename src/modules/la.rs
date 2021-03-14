
// Linear algebra

// Per default, matrices are stored in row major. The shape order
// of Box<[ShapeStride]> is the same as the index order.

use std::rc::Rc;
use std::cell::RefCell;
use std::any::Any;
use std::fmt::{Write,Display};
use std::ops::{Add,Sub,Mul,AddAssign};

use crate::complex::c64;
use crate::object::{
    Object, List, Exception, FnResult, Interface,
    VARIADIC, float, new_module, downcast
};
use crate::vm::{Env,interface_types_set,interface_index};
use crate::class::Class;

trait Zero {fn zero() -> Self;}
impl Zero for i32 {fn zero() -> i32 {0}}
impl Zero for f64 {fn zero() -> f64 {0.0}}
impl Zero for c64 {fn zero() -> c64{c64{re: 0.0, im: 0.0}}}

trait Number: 'static+Copy
    +Add<Output=Self>
    +Sub<Output=Self>
    +Mul<Output=Self>
    +AddAssign
    +Zero
    +Display
{}

impl<T> Number for T
where T: 'static+Copy
    +Add<Output=T>
    +Sub<Output=Self>
    +Mul<Output=Self>
    +AddAssign
    +Zero
    +Display
{}

struct ShapeStride{
    shape: usize,
    stride: isize
}

struct Array<T> {
    n: usize,
    base: isize,
    s: Box<[ShapeStride]>,
    data: Rc<RefCell<Vec<T>>>
}

impl<T: Number> Array<T> {
    fn vector(v: Vec<T>) -> Rc<Array<T>> {
        let shape = v.len();
        let data = Rc::new(RefCell::new(v));
        Rc::new(Array{
            s: Box::new([ShapeStride{shape, stride: 1}]),
            n: 1, base: 0, data
        })
    }
    fn matrix(m: usize, n: usize, a: Vec<T>) -> Rc<Array<T>> {
        let data = Rc::new(RefCell::new(a));
        Rc::new(Array{
            s: Box::new([
                ShapeStride{shape: m, stride: n as isize},
                ShapeStride{shape: n, stride: 1}
            ]),
            n: 2, base: 0, data,
        })
    }
}

impl Interface for Array<f64> {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "ArrayOfFloat".to_string()
    }
    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        to_string(&self,env)
    }
    fn add(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        add(&self,b,env)
    }
    fn sub(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        sub(&self,b,env)
    }
    fn mul(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        mul(&self,b,env)
    }
    fn rmul(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        let r = match *a {
            Object::Int(x) => float(x),
            Object::Float(x) => x,
            _ => return env.type_error(
                "Type error in r*v: r has to be of type Int or Float.")
        };
        let data = self.data.borrow();
        map_unary(&self,&data,&|x| r*x)
    }
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        index(&self,indices,env)
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        get(&self,key,env)
    }
    fn abs(self: Rc<Self>, _env: &mut Env) -> FnResult {
        return abs_f64(&self);
    }
}

impl Interface for Array<c64> {
    fn as_any(&self) -> &dyn Any {self}
    fn type_name(&self, _env: &mut Env) -> String {
        "ArrayOfComplex".to_string()
    }
    fn to_string(self: Rc<Self>, env: &mut Env) -> Result<String,Box<Exception>> {
        to_string(&self,env)
    }
    fn add(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        add(&self,b,env)
    }
    fn sub(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        sub(&self,b,env)
    }
    fn mul(self: Rc<Self>, b: &Object, env: &mut Env) -> FnResult {
        mul(&self,b,env)
    }
    fn rmul(self: Rc<Self>, a: &Object, env: &mut Env) -> FnResult {
        let r = match *a {
            Object::Int(x) => c64{re: float(x), im: 0.0},
            Object::Float(x) => c64{re: x, im: 0.0},
            Object::Complex(x) => x,
            _ => return env.type_error(
                "Type error in r*v: r has to be of type Int or Float.")
        };
        let data = self.data.borrow();
        map_unary(&self,&data,&|x| r*x)
    }
    fn index(self: Rc<Self>, indices: &[Object], env: &mut Env) -> FnResult {
        index(&self,indices,env)
    }
    fn get(self: Rc<Self>, key: &Object, env: &mut Env) -> FnResult {
        get(&self,key,env)
    }
    fn abs(self: Rc<Self>, _env: &mut Env) -> FnResult {
        return abs_c64(&self);
    }
}

fn to_string<T: Number>(a: &Array<T>, _env: &mut Env)
-> Result<String,Box<Exception>>
{
    if a.n==1 {
        let mut s = "vector(".to_string();
        let data = a.data.borrow();
        vector_to_string(a,&mut s,&data);
        s.push(')');
        return Ok(s);
    }else if a.n==2 {
        let mut s = "matrix(\n".to_string();
        let data = a.data.borrow();            
        matrix_to_string(a,&mut s,&data);
        s.push_str("\n)");
        return Ok(s);
    }else{
        unimplemented!();
    }
}

fn index<T>(a: &Array<T>, indices: &[Object], env: &mut Env) -> FnResult
where T: Number, Object: From<T>
{
    if a.n==1 && indices.len()==1 {
        match indices[0] {
            Object::Int(i) => {
                let i = i as isize;
                if i>=0 && (i as usize)<a.s[0].shape {
                    let data = a.data.borrow_mut();
                    let index = (a.base as isize+a.s[0].stride*i) as usize;
                    return Ok(Object::from(data[index]));
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

fn get<T>(a: &Array<T>, key: &Object, env: &mut Env) -> FnResult
where T: Number, Object: From<T>
{
    if let Object::String(ref s) = *key {
        let v = &s.data;
        match v.len() {
            4 => {
                if v[0..4] == ['l','i','s','t'] {
                    return Ok(array_to_list(a));
                }
            },
            5 => {
                if v[0..5] == ['s','h','a','p','e'] {
                    return shape(a);
                }
            },
            _ => {}
        }
        let t = &env.rte().interface_types.borrow()[interface_index::ARRAY];
        match t.slot(key) {
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

fn shape<T>(a: &Array<T>) -> FnResult {
    let n = a.s.len();
    let mut v: Vec<Object> = Vec::with_capacity(n);
    for i in 0..n {
        v.push(Object::Int(a.s[i].shape as i32));
    }
    return Ok(List::new_object(v));
}

fn array_to_list<T: Number>(a: &Array<T>) -> Object
where Object: From<T>
{
    if a.n==1 {
        let data = &*a.data.borrow();
        let base = a.base;
        let shape = a.s[0].shape;
        let stride = a.s[0].stride;
        let mut v: Vec<Object> = Vec::with_capacity(shape);
        for i in 0..shape {
            let x = data[(base+i as isize*stride) as usize];
            v.push(Object::from(x));
        }
        return List::new_object(v);
    }else{
        panic!();
    }
}

fn abs_f64(a: &Array<f64>) -> FnResult {
    let n = a.s[0].shape;
    let stride = a.s[0].stride;
    let base = a.base;
    let data = &a.data.borrow();
    let mut y = 0.0;
    for i in 0..n {
        let x = data[(base+i as isize*stride) as usize];
        y += x*x;
    }
    return Ok(Object::Float(y.sqrt()));

}

fn abs_c64(a: &Array<c64>) -> FnResult {
    let n = a.s[0].shape;
    let stride = a.s[0].stride;
    let base = a.base;
    let data = &a.data.borrow();
    let mut y = 0.0;
    for i in 0..n {
        let x = data[(base+i as isize*stride) as usize];
        y += x.re*x.re+x.im*x.im;
    }
    return Ok(Object::Float(y.sqrt()));
}

fn add<T: Number>(a: &Array<T>, b: &Object, _env: &mut Env) -> FnResult
where Array<T>: Interface
{
    if let Some(b) = downcast::<Array<T>>(b) {
        let data = a.data.borrow();
        map_binary(a,&data,b,&|x,y| x+y)
    }else{
        panic!()
    }
}

fn sub<T: Number>(a: &Array<T>, b: &Object, _env: &mut Env) -> FnResult
where Array<T>: Interface
{
    if let Some(b) = downcast::<Array<T>>(b) {
        let data = a.data.borrow();
        map_binary(a,&data,b,&|x,y| x-y)
    }else{
        panic!()
    }
}

fn mul<T: Number>(a: &Array<T>, b: &Object, env: &mut Env) -> FnResult
where Array<T>: Interface, Object: From<T>
{
    if let Some(b) = downcast::<Array<T>>(b) {
        let adata = &a.data.borrow();
        let bdata = &b.data.borrow();
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
                    mul_matrix_matrix(m,p,n,a,b,adata,bdata)))
            }else if b.n ==1 {
                let m = a.s[0].shape;
                let n = a.s[1].shape;
                if n != b.s[0].shape {
                    return env.value_error(
                        "Value error in A*b: A.shape[1] != b.shape[0].");
                }
                Ok(Object::Interface(
                    mul_matrix_vector(m,n,a,b,adata,bdata)))
            }else{
                panic!()
            }
        }else if a.n == 1 {
            let n = a.s[0].shape;
            if n != b.s[0].shape {
                return env.value_error(
                    "Value error in a*b: a.shape[0] != b.shape[0].");
            }
            Ok(scalar_product(n,a,b,adata,bdata))
        }else{
            panic!()
        }
    }else{
        panic!()
    }
}

fn map_binary<T: Number>(a: &Array<T>, adata: &[T], b: &Array<T>,
    f: &dyn Fn(T,T)->T
) -> FnResult
where Array<T>: Interface
{
    if a.n==1 {
        let astride = a.s[0].stride;
        let ashape = a.s[0].shape;
        let abase = a.base;
        let mut v: Vec<T> = Vec::with_capacity(ashape);

        if b.n != 1 {
            panic!()
        }
        if ashape != b.s[0].shape {
            panic!()
        }
        let bstride = b.s[0].stride;
        let bbase = b.base;
        let bdata = b.data.borrow();
        for i in 0..ashape {
            let aindex = (abase+(i as isize)*astride) as usize;
            let bindex = (bbase+(i as isize)*bstride) as usize;
            v.push(f(adata[aindex],bdata[bindex]));
        }
        return Ok(Object::Interface(Array::vector(v)));

    }else{
        unimplemented!();
    }
}

fn map_unary<T: Number>(a: &Array<T>, data: &[T], f: &dyn Fn(T)->T)
-> FnResult
where Array<T>: Interface
{
    if a.n==1 {
        let stride = a.s[0].stride;
        let shape = a.s[0].shape;
        let base = a.base;
        let mut v: Vec<T> = Vec::with_capacity(shape);
        for i in 0..shape {
            let index = (base+(i as isize)*stride) as usize;
            v.push(f(data[index]));
        }
        return Ok(Object::Interface(Array::vector(v)));

    }else{
        unimplemented!();
    }
}

fn scalar_product<T: Number>(n: usize,
    a: &Array<T>, b: &Array<T>, adata: &[T], bdata: &[T]
) -> Object
where Object: From<T>
{
    let abase = a.base;
    let bbase = b.base;
    let astride = a.s[0].stride;
    let bstride = b.s[0].stride;
    let mut y = T::zero();
    for i in 0..n {
        let aindex = abase+i as isize*astride;
        let bindex = bbase+i as isize*bstride;
        y += adata[aindex as usize]*bdata[bindex as usize];
    }
    return Object::from(y);
}

fn mul_matrix_vector<T: Number>(m: usize, n: usize,
    a: &Array<T>, b: &Array<T>, adata: &[T], bdata: &[T]
) -> Rc<Array<T>>
where Array<T>: Interface
{
    let abase = a.base;
    let bbase = b.base;
    let aistride = a.s[0].stride;
    let ajstride = a.s[1].stride;
    let bjstride = b.s[0].stride;
    let mut v: Vec<T> = Vec::with_capacity(m);
    for i in 0..m {
        let mut y = T::zero();
        for j in 0..n {
            let aindex = abase+i as isize*aistride+j as isize*ajstride;
            let bindex = bbase+j as isize*bjstride;
            y += adata[aindex as usize]*bdata[bindex as usize];
        }
        v.push(y);
    }
    return Array::vector(v);
}

fn mul_matrix_matrix<T: Number>(m: usize, p: usize, n: usize,
    a: &Array<T>, b: &Array<T>, adata: &[T], bdata: &[T]
) -> Rc<Array<T>>
{
    let abase = a.base;
    let bbase = b.base;
    let aistride = a.s[0].stride;
    let akstride = a.s[1].stride;
    let bkstride = b.s[0].stride;
    let bjstride = b.s[1].stride;
    let mut v: Vec<T> = Vec::with_capacity(m*n);
    for i in 0..m {
        for j in 0..n {
            let mut y = T::zero();
            for k in 0..p {
                let aindex = abase+i as isize*aistride+k as isize*akstride;
                let bindex = bbase+k as isize*bkstride+j as isize*bjstride;
                y += adata[aindex as usize]*bdata[bindex as usize];
            }
            v.push(y);
        }
    }
    return Array::matrix(m,n,v);
}

fn vector_to_string<T: Number>(a: &Array<T>, s: &mut String, data: &[T]) {
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

fn matrix_to_string<T: Number>(a: &Array<T>, s: &mut String, data: &[T]) {
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

fn vector_from_list_c64(env: &mut Env, a: &[Object]) -> FnResult {
    let mut v: Vec<c64> = Vec::with_capacity(a.len());
    for x in a {
        let x = match *x {
            Object::Int(x) => c64{re: float(x), im: 0.0},
            Object::Float(x) => c64{re: x, im: 0.0},
            Object::Complex(x) => x,
            _ => return env.type_error(
                "Type error in vector(*a): expected all a[k] of type Int, Float or Complex.")
        };
        v.push(x);
    }
    return Ok(Object::Interface(Array::<c64>::vector(v)));
}

fn vector_from_list(env: &mut Env, a: &[Object]) -> FnResult {
    if !a.is_empty() {
        if let Object::Complex(_) = a[0] {
            return vector_from_list_c64(env,a);
        }
    }
    let mut v: Vec<f64> = Vec::with_capacity(a.len());
    for x in a {
        let x = match *x {
            Object::Int(x) => float(x),
            Object::Float(x) => x,
            _ => return env.type_error(
                "Type error in vector(*a): expected all a[k] of type Int or Float.")
        };
        v.push(x);
    }
    return Ok(Object::Interface(Array::<f64>::vector(v)));
}

fn matrix_from_lists_c64(m: usize, n: usize,
    env: &mut Env, argv: &[Object]
) -> FnResult
{
    let mut v: Vec<c64> = Vec::with_capacity(m*n);
    for x in argv {
        let a = match *x {
            Object::List(ref a) => a,
            _ => panic!()
        };
        let data = &a.borrow().v;
        if data.len() != n {
            return env.value_error(
                "Value error in matrix(*a): all a[i] must be of the same size.");
        }
        for x in data {
            let x = match *x {
                Object::Int(x) => c64{re: float(x), im: 0.0},
                Object::Float(x) => c64{re: x, im: 0.0},
                Object::Complex(x) => x,
                _ => return env.type_error(
                    "Type error in matrix(*a): expected all a[i][j] of type Int, Float or Complex.")
            };
            v.push(x);
        }
    }
    return Ok(Object::Interface(Array::<c64>::matrix(m,n,v)));
}

fn matrix_from_lists(env: &mut Env, argv: &[Object]) -> FnResult {
    let m = argv.len();
    if m==0 {
        return env.value_error(
            "Value error in matrix(*a): expected at least one row.");
    }
    let n = match argv[0] {
        Object::List(ref a) => {
            let a = &a.borrow().v;
            let n = a.len();
            if n>0 {
                if let Object::Complex(_) = a[0] {
                    return matrix_from_lists_c64(m,n,env,argv);
                }
            }
            n
        },
        _ => panic!()
    };
    let mut v: Vec<f64> = Vec::with_capacity(m*n);
    for x in argv {
        let a = match *x {
            Object::List(ref a) => a,
            _ => panic!()
        };
        let data = &a.borrow().v;
        if data.len() != n {
            return env.value_error(
                "Value error in matrix(*a): all a[i] must be of the same size.");
        }
        for x in data {
            let x = match *x {
                Object::Int(x) => float(x),
                Object::Float(x) => x,
                _ => return env.type_error(
                    "Type error in matrix(*a): expected all a[i][j] of type Int or Float.")
            };
            v.push(x);
        }
    }
    return Ok(Object::Interface(Array::<f64>::matrix(m,n,v)));
}

fn vector(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    return vector_from_list(env,argv);
}

fn matrix(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    return matrix_from_lists(env,argv);
}

pub fn load_la(env: &mut Env) -> Object
{
    let type_array = Class::new("Array",&Object::Null);
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
        m.insert_fn_plain("vec",vector,0,VARIADIC);
    }

    return Object::Interface(Rc::new(la));
}
