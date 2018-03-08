
// Linear algebra

#![allow(unused_imports)]

use std::rc::Rc;
use std::any::Any;
use object::{Object, FnResult, Function, Interface, Exception,
  new_module, VARIADIC};
use vm::{Env, op_add, op_sub, op_mpy, op_div};
use complex::Complex64;

struct ShapeStride{
  shape: usize,
  stride: isize
}

struct Array{
  n: usize,
  base: usize,
  s: Box<[ShapeStride]>,
  a: Rc<Vec<Object>>
}

impl Array {
  fn vector(a: Rc<Vec<Object>>) -> Rc<Array> {
    Rc::new(Array{
      n: 1,
      s: Box::new([ShapeStride{shape: a.len(), stride: 1}]),
      base: 0,
      a: a
    })
  }
  fn matrix(m: usize, n: usize, a: Rc<Vec<Object>>) -> Rc<Array> {
    Rc::new(Array{
      n: 2,
      s: Box::new([
        ShapeStride{shape: n, stride: 1},
        ShapeStride{shape: m, stride: n as isize}
      ]),
      base: 0,
      a: a,
    })
  }
  fn downcast(x: &Object) -> Option<&Array> {
    if let Object::Interface(ref a) = *x {
      a.as_any().downcast_ref::<Array>()
    }else{
      None
    }
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
      let mut first = true;
      let stride = self.s[0].stride;
      let base = self.base as isize;
      for i in 0..self.s[0].shape {
        if first {first = false;}
        else {s.push_str(", ");}
        let x = &self.a[(base+i as isize*stride) as usize];
        s.push_str(&try!(x.repr(env)));
      }
      s.push_str(")");
      return Ok(s);
    }else if self.n==2 {
      let m = self.s[1].shape;
      let n = self.s[0].shape;
      let mut s = "matrix(\n".to_string();
      let mut first = true;
      let base = self.base as isize;
      let istride = self.s[1].stride;
      let jstride = self.s[0].stride;
      for i in 0..m {
        if first {first = false;}
        else {s.push_str(",\n");}
        let ibase = base+i as isize*istride;
        s.push_str("  [");
        let mut jfirst = true;
        for j in 0..n {
          if jfirst {jfirst = false;}
          else {s.push_str(", ");}
          let x = &self.a[(ibase+j as isize*jstride) as usize];
          s.push_str(&try!(x.repr(env)));
        }
        s.push_str("]");
      }
      s.push_str("\n)");
      return Ok(s);
    }else{
      panic!();
    }
  }
  fn get(&self, key: &Object, env: &mut Env) -> FnResult {
    if let Object::String(ref s) = *key {
      match s.v.len() {
        1 => if s.v[0]=='T' {
          return Ok(Object::Interface(transpose(self)));
        },
        4 => if s.v[0..4]==['c','o','n','j'] {
          return Ok(Object::Interface(conj(self)));
        },
        _ => {}
      }
      env.index_error("Index error in t.x: x not found.")
    }else{
      env.type_error("Type error in t.x: x is not a string.")
    }
  }
  fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
    if self.n==1 && indices.len()==1 {
      match indices[0] {
        Object::Int(i) => {
          let i = i as isize;
          if i>=0 && (i as usize)<self.s[0].shape {
            return Ok(self.a[(self.base as isize+self.s[0].stride*i) as usize].clone());
          }else{
            return env.index_error("Index error in a[i]: out of bounds.");
          }
        },
        ref i => {
          return env.type_error1("Type error in a[i]: is not an integer.","i",i);
        }
      }
    }else{
      return env.index_error("Index error in a[...]: shape does not fit.");
    }
  }
  fn add(&self, b: &Object, env: &mut Env) -> FnResult {
    if self.n==1 {
      let stride = self.s[0].stride;
      let base = self.base;
      let mut v: Vec<Object> = Vec::with_capacity(self.s[0].shape);
      if let Some(b) = Array::downcast(b) {
        if b.n != 1 {
          return env.type_error(&format!("Type error in v+w: v is a vector, but w is of order {}.",b.n));
        }
        if self.s[0].shape != b.s[0].shape {
          return env.type_error("Type error in v+w: v is not of the same size as w.");
        }
        let stride2 = b.s[0].stride;
        let base2 = b.base;
        for i in 0..self.s[0].shape {
          let y = try!(op_add(env,
            &self.a[(base as isize+i as isize*stride) as usize],
            &b.a[(base2 as isize+i as isize*stride2) as usize]
          ));
          v.push(y);
        }
      }else{
        for i in 0..self.s[0].shape {
          let y = try!(op_add(env,&self.a[(base as isize+i as isize*stride) as usize],b));
          v.push(y);
        }
      }
      return Ok(Object::Interface(Array::vector(Rc::new(v))));
    }else{
      panic!();
    }
  }
  fn sub(&self, b: &Object, env: &mut Env) -> FnResult {
    if self.n==1 {
      let stride = self.s[0].stride;
      let base = self.base;
      let mut v: Vec<Object> = Vec::with_capacity(self.s[0].shape);
      if let Some(b) = Array::downcast(b) {
        if b.n != 1 {
          return env.type_error(&format!("Type error in v-w: v is a vector, but w is of order {}.",b.n));
        }
        if self.s[0].shape != b.s[0].shape {
          return env.type_error("Type error in v-w: v is not of the same size as w.");
        }
        let stride2 = b.s[0].stride;
        let base2 = b.base;
        for i in 0..self.s[0].shape {
          let y = try!(op_sub(env,
            &self.a[(base as isize+i as isize*stride) as usize],
            &b.a[(base2 as isize+i as isize*stride2) as usize]
          ));
          v.push(y);
        }
      }else{
        for i in 0..self.s[0].shape {
          let y = try!(op_sub(env,&self.a[(base as isize+i as isize*stride) as usize],b));
          v.push(y);
        }
      }
      return Ok(Object::Interface(Array::vector(Rc::new(v))));
    }else{
      panic!();
    }
  }
  fn rmpy(&self, a: &Object, env: &mut Env) -> FnResult {
    if self.n==1 {
      let stride = self.s[0].stride;
      let base = self.base;
      let mut v: Vec<Object> = Vec::with_capacity(self.s[0].shape);
      for i in 0..self.s[0].shape {
        let y = try!(op_mpy(env,a,&self.a[(base as isize+i as isize*stride) as usize]));
        v.push(y);
      }
      return Ok(Object::Interface(Array::vector(Rc::new(v))));
    }else{
      panic!();
    }
  }
  fn mpy(&self, b: &Object, env: &mut Env) -> FnResult {
    if self.n==1 {
      if let Some(b) = Array::downcast(b) {
        if b.n==1 {
          let m = self.s[0].shape.min(b.s[0].shape);
          return scalar_product(env,m,
            &self.a, self.base, self.s[0].stride,
            &b.a, b.base, b.s[0].stride
          );
        }
      }
      return self.rmpy(b,env);
    }else if self.n==2 {
      if let Some(b) = Array::downcast(b) {
        if b.n==1 {
          let m = self.s[0].shape.min(b.s[0].shape);
          return mpy_matrix_vector(env,m,self,b);
        }else if b.n==2 {
          if self.s[0].shape != b.s[1].shape {
            return env.value_error(
              "Value error in matrix multiplication A*B:\n  A.shape[0] != B.shape[1]."
            );
          }
          return mpy_matrix_matrix(env,self.s[0].shape,self,b);
        }
      }
      return self.rmpy(b,env);
    }else{
      return self.rmpy(b,env);
    }
  }
}

fn map_plain(a: &Array, f: fn(&Object)->Object) -> Rc<Array> {
  if a.n==1 {
    let mut v: Vec<Object> = Vec::with_capacity(a.s[0].shape);
    let stride = a.s[0].stride;
    let base = a.base as isize;
    for i in 0..a.s[0].shape {
      let x = &a.a[(base+i as isize*stride) as usize];
      v.push(f(x));
    }
    return Array::vector(Rc::new(v));
  }else if a.n==2 {
    let m = a.s[1].shape;
    let n = a.s[0].shape;
    let istride = a.s[1].stride;
    let jstride = a.s[0].stride;

    let mut v: Vec<Object> = Vec::with_capacity(m*n);
    let base = a.base as isize;
    for i in 0..m {
      let jbase = base+i as isize*istride;
      for j in 0..n {
        let x = &a.a[(jbase+j as isize*jstride) as usize];
        v.push(f(x));
      }
    }
    return Array::matrix(m,n,Rc::new(v));
  }else{
    panic!();
  }
}

fn conj_element(x: &Object) -> Object {
  match *x {
    Object::Complex(z) => Object::Complex(Complex64{
      re: z.re, im: -z.im
    }),
    ref x => x.clone()
  }
}

fn conj(a: &Array) -> Rc<Array> {
  map_plain(a,conj_element)
}

fn transpose(a: &Array) -> Rc<Array> {
  if a.n==2 {
    let n = a.s[0].shape;
    let m = a.s[1].shape;
    Rc::new(Array{
      n: 2,
      s: Box::new([
        ShapeStride{shape: m, stride: n as isize},
        ShapeStride{shape: n, stride: 1}
      ]),
      base: 0,
      a: a.a.clone(),
    })
  }else{
    panic!();
  }
}

fn scalar_product(env: &mut Env, n: usize,
  a: &[Object], abase: usize, astride: isize,
  b: &[Object], bbase: usize, bstride: isize
) -> FnResult {
  let mut sum = try!(op_mpy(env,&a[abase],&b[bbase]));
  for i in 1..n {
    let aindex = (abase as isize+i as isize*astride) as usize;
    let bindex = (bbase as isize+i as isize*bstride) as usize;
    let p = try!(op_mpy(env,&a[aindex],&b[bindex]));
    sum = try!(op_add(env,&sum,&p));
  }
  return Ok(sum);
}

fn mpy_matrix_vector(env: &mut Env, n: usize,
  a: &Array, x: &Array
) -> FnResult {
  let mut y: Vec<Object> = Vec::with_capacity(a.s[1].shape);
  for i in 0..a.s[1].shape {
    let base = (a.base as isize+i as isize*a.s[1].stride) as usize;
    let p = try!(scalar_product(env,n,
      &a.a, base, a.s[0].stride,
      &x.a, x.base, x.s[0].stride
    ));
    y.push(p);
  }
  return Ok(Object::Interface(Array::vector(Rc::new(y))));
}

fn mpy_matrix_matrix(env: &mut Env, size: usize,
  a: &Array, b: &Array
) -> FnResult {
  let m = a.s[1].shape;
  let n = b.s[0].shape;
  let mut y: Vec<Object> = Vec::with_capacity(m*n);
  for i in 0..m {
    let ibase = (a.base as isize+i as isize*a.s[1].stride) as usize;
    for j in 0..n {
      let jbase = (b.base as isize+j as isize*b.s[0].stride) as usize;
      let p = try!(scalar_product(env,size,
        &a.a, ibase, a.s[0].stride,
        &b.a, jbase, b.s[1].stride
      ));
      y.push(p);
    }
  }
  if y.len()==1 {
    return Ok(y[0].clone());
  }else{
    return Ok(Object::Interface(Array::matrix(m,n,Rc::new(y))));
  }
}

fn vector(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  return Ok(Object::Interface(Array::vector(Rc::new(Vec::from(argv)))));
}

fn matrix(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  'type_error: loop{
    let m = argv.len();
    let n = match argv[0] {Object::List(ref a) => a.borrow().v.len(), _ => break 'type_error};
    let mut v: Vec<Object> = Vec::with_capacity(m*n);
    for t in argv {
      if let Object::List(ref list) = *t {
        let a = &list.borrow().v;
        for x in a {
          v.push(x.clone());
        }
      }else{
        break 'type_error;
      }
    }
    return Ok(Object::Interface(Array::matrix(m,n,Rc::new(v))));
  }
  return env.type_error("Type error in matrix(args): expected args of type list.");
}

fn array(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    2 => {}, argc => return env.argc_error(argc,2,2,"array")
  }
  let n = match argv[0] {
    Object::Int(x) => x as usize,
    _ => return env.type_error("Type error in array(n,a): n is not an integer.")
  };
  if n==1 {
    let y = try!(::global::list(env,&argv[1]));
    if let Object::List(a) = y {
      return Ok(Object::Interface(Array::vector(Rc::new(a.borrow().v.clone()))));
    }else{
      panic!();
    }
  }else{
    return env.std_exception("Dimension not supported.");
  }
}

pub fn load_la(env: &mut Env) -> Object {
  let la = new_module("la");
  {
    let mut m = la.map.borrow_mut();
    m.insert_fn_plain("vector",vector,0,VARIADIC);
    m.insert_fn_plain("matrix",matrix,0,VARIADIC);
    m.insert_fn_plain("array",array,2,2);
  }
  // let array_type = Table::new(Object::Null);
  
  return Object::Table(Rc::new(la));
}
