
#![allow(unused_imports)]

use object::{Object, FnResult, U32String, Function, Table, List,
  type_error, argc_error, index_error, std_exception,
  VARIADIC, MutableFn
};
use vm::Env;
use rand::Rand;
use std::cmp::Ordering;

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

fn any(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"any");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.any(p): a is not a list.");}
  };
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.any(p): return value of p is not of boolean type.")
    };
    if condition {
      return Ok(Object::Bool(true));
    }
  }
  return Ok(Object::Bool(false));
}

fn all(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"all");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.all(p): a is not a list.");}
  };
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.all(p): return value of p is not of boolean type.")
    };
    if !condition {
      return Ok(Object::Bool(false));
    }
  }
  return Ok(Object::Bool(true));
}

fn new_shuffle() -> MutableFn {
  let mut rng = Rand::new(0);
  return Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
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

fn join(a: &[Object], sep: Option<&Object>,
  left: Option<&Object>, right: Option<&Object>
) -> String {
  let mut s: String = String::new();
  if let Some(left) = left {
    s.push_str(&left.to_string());
  }
  if let Some(sep) = sep {
    let sep = &sep.to_string();
    let mut first = true;
    for x in a {
      if first {
        first = false;
      }else{
        s.push_str(sep);
      }
      s.push_str(&x.to_string());
    }
  }else{
    for x in a {
      s.push_str(&x.to_string());
    }
  }
  if let Some(right) = right {
    s.push_str(&right.to_string());
  }
  return s;
}

fn list_join(pself: &Object, argv: &[Object]) -> FnResult{
  let a = match *pself {
    Object::List(ref x)=>x,
    _ => return type_error("Type error in a.join(): a is not a list.")
  };
  let y = match argv.len() {
    0 => join(&a.borrow().v,None,None,None),
    1 => join(&a.borrow().v,Some(&argv[0]),None,None),
    2 => join(&a.borrow().v,Some(&argv[0]),Some(&argv[1]),None),
    3 => join(&a.borrow().v,Some(&argv[0]),Some(&argv[1]),Some(&argv[2])),
    n => return argc_error(n,0,3,"join")
  };
  Ok(U32String::new_object_str(&y))
}

fn compare(a: &Object, b: &Object) -> Ordering {
  match *a {
    Object::Int(x) => {
      match *b {
        Object::Int(y) => x.cmp(&y),
        _ => Ordering::Equal
      }
    },
    _ => Ordering::Equal
  }
}

fn compare_by_value(a: &(Object,Object), b: &(Object,Object)) -> Ordering {
  match a.1 {
    Object::Int(x) => {
      match b.1 {
        Object::Int(y) => x.cmp(&y),
        _ => Ordering::Equal
      }
    },
    _ => Ordering::Equal
  }
}

fn list_sort(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => return type_error("Type error in a.sort(): a is not a list.")
  };
  match argv.len() {
    0 => {
      a.v.sort_by(compare);
      Ok(Object::Null)
    },
    1 => {
      let mut v: Vec<(Object,Object)> = Vec::with_capacity(a.v.len());
      for x in &a.v {
        let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
        v.push((x.clone(),y));
      }
      v.sort_by(compare_by_value);
      a.v = v.into_iter().map(|x| x.0).collect();
      Ok(Object::Null)
    },
    n => argc_error(n,0,0,"sort")
  }
}

fn list_chain(pself: &Object, argv: &[Object]) -> FnResult{
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => return type_error("Type error in a.sort(): a is not a list.")
  };
  let mut v: Vec<Object> = Vec::new();
  for t in &a.v {
    match *t {
      Object::List(ref t) => {
        for x in &t.borrow_mut().v {
          v.push(x.clone());
        }
      },
      ref x => {
        v.push(x.clone());
      }
    }
  }
  Ok(List::new_object(v))
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("push",push,0,VARIADIC);
  m.insert_fn_plain("append",append,0,VARIADIC);
  m.insert_fn_plain("size",size,0,0);
  m.insert_fn_env  ("map",map,1,1);
  m.insert_fn_env  ("filter",filter,1,1);
  m.insert_fn_env  ("count",count,1,1);
  m.insert_fn_env  ("any",any,1,1);
  m.insert_fn_env  ("all",all,1,1);
  m.insert_fn_plain("join",list_join,0,1);
  m.insert_fn_env  ("sort",list_sort,0,1);
  m.insert_fn_plain("chain",list_chain,0,0);
  m.insert("shuffle",Function::mutable(new_shuffle(),0,0));
}
