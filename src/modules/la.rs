
// Linear algebra

#![allow(unused_imports)]

use std::rc::Rc;
use std::any::Any;
use object::{Object, FnResult, Function,
  type_error, argc_error, new_module,
  Interface, std_exception
};
use vm::{Env, op_add};

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
  fn vector(a: Rc<Vec<Object>>) -> Rc<Interface> {
    Rc::new(Array{
      n: 1,
      s: Box::new([ShapeStride{shape: a.len(), stride: 1}]),
      base: 0,
      a: a
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
  fn to_string(&self) -> String {
    if self.n==1 {
      let mut s = "vector(".to_string();
      let mut first = true;
      let stride = self.s[0].stride;
      let base = self.base;
      for i in 0..self.s[0].shape {
        if first {first = false;}
        else {s.push_str(", ");}
        s.push_str(&self.a[
          (base as isize+i as isize*stride) as usize
        ].repr());
      }
      s.push_str(")");
      return s;
    }else{
      panic!();
    }
  }
  fn add(&self, b: &Object, env: &mut Env) -> FnResult {
    if self.n==1 {
      let stride = self.s[0].stride;
      let base = self.base;
      let mut v: Vec<Object> = Vec::with_capacity(self.s[0].shape);
      if let Some(b) = Array::downcast(b) {
        if b.n != 1 {
          return type_error(&format!("Type error in v+w: v is a vector, but w is of order {}.",b.n));
        }
        if self.s[0].shape != b.s[0].shape {
          return type_error("Type error in v+w: v is not of the same size as w.");
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
}

fn vector(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  return Ok(Object::Interface(Array::vector(Rc::new(Vec::from(argv)))));
}

fn array(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    2 => {}, argc => return argc_error(argc,2,2,"array")
  }
  let n = match argv[0] {
    Object::Int(x) => x as usize,
    _ => return type_error("Type error in array(n,a): n is not an integer.")
  };
  if n==1 {
    let y = try!(::global::list(env,&argv[1]));
    if let Object::List(a) = y {
      return Ok(Object::Interface(Array::vector(Rc::new(a.borrow().v.clone()))));
    }else{
      panic!();
    }
  }else{
    return std_exception("Dimension not supported.");
  }
}

pub fn load_la() -> Object {
  let la = new_module("la");
  {
    let mut m = la.map.borrow_mut();
    m.insert_fn_env("vector",vector,1,1);
    m.insert_fn_env("array",array,2,2);
  }
  return Object::Table(Rc::new(la));
}
