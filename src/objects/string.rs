
// use std::rc::Rc;
// use std::cell::RefCell;
use object::{
  Object, Table, FnResult, U32String
};
use vm::Env;

fn isdigit(c: char) -> bool {
  ('0' as u32)<=(c as u32) && (c as u32)<=('9' as u32)
}

fn isalpha(c: char) -> bool {
  let c = c as u32;
  ('A' as u32)<=c && c<=('Z' as u32) ||
  ('a' as u32)<=c && c<=('z' as u32)
}

fn fisdigit(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {}, n => return env.argc_error(n,0,0,"isdigit")
  }
  match *pself {
    Object::String(ref s) => {
      for c in &s.v {
        if !isdigit(*c) {return Ok(Object::Bool(false));}
      }
      return Ok(Object::Bool(true));
    },
    _ => env.type_error("Type error in s.isdigit(): s is not a string.")
  }
}

fn fisalpha(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {}, n => return env.argc_error(n,0,0,"isalpha")
  }
  match *pself {
    Object::String(ref s) => {
      for c in &s.v {
        if !isalpha(*c) {return Ok(Object::Bool(false));}
      }
      return Ok(Object::Bool(true));
    },
    _ => env.type_error("Type error in s.isalpha(): s is not a string.")
  }
}

fn ljust(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let c = match argv.len() {
    1 => {' '},
    2 => {
      match argv[1] {
        Object::String(ref s) => {
          if s.v.len()==1 {s.v[0]} else {
            return env.value_error("Value error in s.ljust(n,c): size(c)!=1.");
          }
        },
        _ => {
          return env.type_error1("Type error in s.ljust(n,c): c is not a string.","c",&argv[1]);
        }
      }
    },
    n => return env.argc_error(n,2,2,"ljust")
  };
  let s = match *pself {
    Object::String(ref s) => &s.v,
    _ => {return env.type_error1(
      "Type error in s.ljust(n): s is not a string.",
      "s",pself
    );}
  };
  let n = match argv[0] {
    Object::Int(x) => {
      if x<0 {0} else{x as usize}
    },
    _ => {return env.type_error1(
      "Type error in s.ljust(n): n is not an integer.",
      "s",pself
    );}
  };
  let mut v: Vec<char> = s.clone();
  for _ in s.len()..n {
    v.push(c);
  }
  return Ok(U32String::new_object(v));
}

fn rjust(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let c = match argv.len() {
    1 => {' '},
    2 => {
      match argv[1] {
        Object::String(ref s) => {
          if s.v.len()==1 {s.v[0]} else {
            return env.value_error("Value error in s.rjust(n,c): size(c)!=1.");
          }
        },
        _ => {
          return env.type_error1("Type error in s.rjust(n,c): c is not a string.","c",&argv[1]);
        }
      }
    },
    n => return env.argc_error(n,2,2,"ljust")
  };
  let s = match *pself {
    Object::String(ref s) => &s.v,
    _ => {return env.type_error1(
      "Type error in s.rjust(n): s is not a string.",
      "s",pself
    );}
  };
  let n = match argv[0] {
    Object::Int(x) => {
      if x<0 {0} else{x as usize}
    },
    _ => {return env.type_error1(
      "Type error in s.rjust(n): n is not an integer.",
      "s",pself
    );}
  };
  let mut v: Vec<char> = Vec::new();
  for _ in s.len()..n {
    v.push(c);
  }
  for x in s {
    v.push(*x);
  }
  return Ok(U32String::new_object(v));
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("isdigit",fisdigit,0,0);
  m.insert_fn_plain("isalpha",fisalpha,0,0);
  m.insert_fn_plain("ljust",ljust,1,2);
  m.insert_fn_plain("rjust",rjust,1,2);
}
