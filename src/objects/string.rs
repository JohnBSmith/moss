
// use std::rc::Rc;
// use std::cell::RefCell;
use object::{
  Object, Table, FnResult,
  type_error, argc_error,
};
// use vm::Env;

fn isdigit(c: char) -> bool {
  ('0' as u32)<=(c as u32) && (c as u32)<=('9' as u32)
}

fn isalpha(c: char) -> bool {
  let c = c as u32;
  ('A' as u32)<=c && c<=('Z' as u32) ||
  ('a' as u32)<=c && c<=('z' as u32)
}

fn fisdigit(pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {}, n => return argc_error(n,0,0,"isdigit")
  }
  match *pself {
    Object::String(ref s) => {
      for c in &s.v {
        if !isdigit(*c) {return Ok(Object::Bool(false));}
      }
      return Ok(Object::Bool(true));
    },
    _ => type_error("Type error in s.isdigit(): s is not a string.")
  }
}

fn fisalpha(pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {}, n => return argc_error(n,0,0,"isalpha")
  }
  match *pself {
    Object::String(ref s) => {
      for c in &s.v {
        if !isalpha(*c) {return Ok(Object::Bool(false));}
      }
      return Ok(Object::Bool(true));
    },
    _ => type_error("Type error in s.isalpha(): s is not a string.")
  }
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("isdigit",fisdigit,0,0);
  m.insert_fn_plain("isalpha",fisalpha,0,0);
}
