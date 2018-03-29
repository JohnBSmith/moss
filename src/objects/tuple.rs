
use std::any::Any;
use std::rc::Rc;

use object::{Object, Exception, Interface, FnResult};
use vm::{Env, op_eq};

pub struct Tuple{
  pub v: Vec<Object>
}

impl Tuple {
  pub fn new_object(v: Vec<Object>) -> Object {
    return Object::Interface(Rc::new(Tuple{v}));
  }
  pub fn downcast(x: &Object) -> Option<&Tuple> {
    if let Object::Interface(ref a) = *x {
      a.as_any().downcast_ref::<Tuple>()
    }else{
      None
    }
  }
}

impl Interface for Tuple {
  fn as_any(&self) -> &Any {self}
  fn type_name(&self) -> String {
    "Tuple".to_string()
  }
  fn to_string(&self, env: &mut Env) -> Result<String,Box<Exception>> {
    let mut s = String::from("(");
    let mut first = true;
    for x in &self.v {
      if first {first = false;} else {s.push_str(", ");}
      s.push_str(&try!(x.string(env)));
    }
    s.push_str(")");
    return Ok(s);
  }
  fn index(&self, indices: &[Object], env: &mut Env) -> FnResult {
    match indices.len() {
      1 => {}, n => return env.argc_error(n,1,1,"tuple indexing")
    }
    let i = match indices[0] {
      Object::Int(i) => if i<0 {0} else {i as usize},
      ref i => return env.type_error1("Type error in t[i]: i is not an integer.","i",i)
    };
    if i < self.v.len() {
      Ok(self.v[i].clone())
    }else{
      env.index_error("Index error in t[i]: i is out of upper bound.")
    }
  }
  fn eq_plain(&self, b: &Object) -> bool {
    if let Some(b) = Tuple::downcast(b) {
      if self.v.len()==b.v.len() {
        for i in 0..self.v.len() {
          if self.v[i] != b.v[i] {return false;}
        }
        return true;
      }else{
        return false;
      }
    }else{
      return false;
    }
  }
  fn eq(&self, b: &Object, env: &mut Env) -> FnResult {
    if let Some(b) = Tuple::downcast(b) {
      let len = self.v.len();
      if len == b.v.len() {
        for i in 0..len {
          let y = try!(op_eq(env,&self.v[i],&b.v[i]));
          if let Object::Bool(y) = y {
            if !y {return Ok(Object::Bool(false));}
          }else{
            return env.type_error("Type error in t1==t2: t[i]==t[i] is not a boolean.");
          }
        }
        return Ok(Object::Bool(true));
      }else{
        return Ok(Object::Bool(false));
      }
    }else{
      return Ok(Object::Bool(false));
    }
  }
}