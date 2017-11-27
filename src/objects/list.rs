
#![allow(unused_imports)]

use object::{Object, FnResult, U32String, Function, Table, List,
  type_error, argc_error, index_error, std_exception,
  VARIADIC, MutableFn
};
use vm::Env;
use rand::Rand;

fn push(pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          for i in 0..argv.len() {
            a.v.push(argv[i].clone());
          }
          Ok(Object::Null)        
        },
        Err(_) => {std_exception(
          "Memory error in a.push(x): internal buffer of a was aliased.\n\
           Try to replace a by copy(a) at some place."
        )}
      }
    },
    _ => type_error("Type error in a.push(x): a is not a list.")
  }
}

fn append(pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      for i in 0..argv.len() {
        match argv[i] {
          Object::List(ref ai) => {
            let mut v = (&ai.borrow().v[..]).to_vec();
            let mut a = a.borrow_mut();
            a.v.append(&mut v);
          },
          _ => return type_error("Type error in a.append(b): b is not a list.")
        }
      }
      Ok(Object::Null)
    },
    _ => type_error("Type error in a.append(b): a is not a list.")
  }
}

fn size(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 0 {
    return argc_error(argv.len(),0,0,"size");
  }
  match *pself {
    Object::List(ref a) => {
      Ok(Object::Int(a.borrow().v.len() as i32))
    },
    _ => type_error("Type error in a.size(): a is not a list.")
  }
}

fn map(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"map");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.map(f): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::with_capacity(a.v.len());
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    v.push(y);
  }
  return Ok(List::new_object(v));
}

fn filter(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"filter");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.filter(p): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::new();
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.filter(p): return value of p is not of boolean type.")
    };
    if condition {
      v.push(x.clone());
    }
  }
  return Ok(List::new_object(v));
}

fn count(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"count");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.count(p): a is not a list.");}
  };
  let mut k: i32 = 0;
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.count(p): return value of p is not of boolean type.")
    };
    if condition {k+=1;}
  }
  return Ok(Object::Int(k));
}

fn each(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"each");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.each(f): a is not a list.");}
  };
  for x in &a.v {
    try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
  }
  return Ok(Object::Null);
}

fn new_shuffle() -> MutableFn {
  let mut rng = Rand::new(0);
  return Box::new(move |pself: &Object, argv: &[Object]| -> FnResult{
    if argv.len() != 0 {
      return argc_error(argv.len(),0,0,"shuffle");
    }
    match *pself {
      Object::List(ref a) => {
        let mut ba = a.borrow_mut();
        rng.shuffle(&mut ba.v);
        Ok(Object::List(a.clone()))
      },
      _ => type_error("Type error in a.shuffle(): a is not a list.")
    }
  });
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert("push",   Function::plain(push,0,VARIADIC));
  m.insert("append", Function::plain(append,0,VARIADIC));
  m.insert("size",   Function::plain(size,0,0));
  m.insert("map",    Function::env(map,1,1));
  m.insert("filter", Function::env(filter,1,1));
  m.insert("count",  Function::env(count,1,1));
  m.insert("each",   Function::env(each,1,1));
  m.insert("shuffle",Function::mutable(new_shuffle(),0,0));
}
