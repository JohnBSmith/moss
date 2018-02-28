
#![allow(unused_imports)]

use object::{Object, FnResult, U32String, Function, Table, List,
  VARIADIC, MutableFn
};
use vm::Env;
use rand::Rand;

fn push(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          for i in 0..argv.len() {
            a.v.push(argv[i].clone());
          }
          Ok(Object::Null)        
        },
        Err(_) => {env.std_exception(
          "Memory error in a.push(x): internal buffer of a was aliased.\n\
           Try to replace a by copy(a) at some place."
        )}
      }
    },
    _ => env.type_error("Type error in a.push(x): a is not a list.")
  }
}

fn append(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      for i in 0..argv.len() {
        match argv[i] {
          Object::List(ref ai) => {
            let mut v = (&ai.borrow().v[..]).to_vec();
            let mut a = a.borrow_mut();
            a.v.append(&mut v);
          },
          ref b => return env.type_error1(
            "Type error in a.append(b): b is not a list.",
            "b",b)
        }
      }
      Ok(Object::Null)
    },
    _ => env.type_error("Type error in a.append(b): a is not a list.")
  }
}

fn size(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 0 {
    return env.argc_error(argv.len(),0,0,"size");
  }
  match *pself {
    Object::List(ref a) => {
      Ok(Object::Int(a.borrow().v.len() as i32))
    },
    _ => env.type_error("Type error in a.size(): a is not a list.")
  }
}

fn map(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"map");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return env.type_error("Type error in a.map(f): a is not a list.");}
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
    return env.argc_error(argv.len(),1,1,"filter");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return env.type_error("Type error in a.filter(p): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::new();
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      ref value => return env.type_error2(
        "Type error in a.filter(p): return value of p is not of boolean type.",
        "x","p(x)",x,value)
    };
    if condition {
      v.push(x.clone());
    }
  }
  return Ok(List::new_object(v));
}

fn count(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"count");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return env.type_error("Type error in a.count(p): a is not a list.");}
  };
  let mut k: i32 = 0;
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      ref value => return env.type_error2(
        "Type error in a.count(p): return value of p is not of boolean type.",
        "x","p(x)",x,value)
    };
    if condition {k+=1;}
  }
  return Ok(Object::Int(k));
}

fn any(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"any");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return env.type_error("Type error in a.any(p): a is not a list.");}
  };
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      ref value => return env.type_error2(
        "Type error in a.any(p): return value of p is not of boolean type.",
        "x","p(x)",x,value)
    };
    if condition {
      return Ok(Object::Bool(true));
    }
  }
  return Ok(Object::Bool(false));
}

fn all(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"all");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return env.type_error("Type error in a.all(p): a is not a list.");}
  };
  for x in &a.v {
    let y = try!(env.call(&argv[0],&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      ref value => return env.type_error2(
        "Type error in a.all(p): return value of p is not of boolean type.",
        "x","p(x)",x,value)
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
      return env.argc_error(argv.len(),0,0,"shuffle");
    }
    match *pself {
      Object::List(ref a) => {
        let mut ba = a.borrow_mut();
        rng.shuffle(&mut ba.v);
        Ok(Object::List(a.clone()))
      },
      _ => env.type_error("Type error in a.shuffle(): a is not a list.")
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

fn list_join(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  let a = match *pself {
    Object::List(ref x)=>x,
    _ => return env.type_error("Type error in a.join(): a is not a list.")
  };
  let y = match argv.len() {
    0 => join(&a.borrow().v,None,None,None),
    1 => join(&a.borrow().v,Some(&argv[0]),None,None),
    2 => join(&a.borrow().v,Some(&argv[0]),Some(&argv[1]),None),
    3 => join(&a.borrow().v,Some(&argv[0]),Some(&argv[1]),Some(&argv[2])),
    n => return env.argc_error(n,0,3,"join")
  };
  Ok(U32String::new_object_str(&y))
}

fn list_chain(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => return env.type_error("Type error in a.chain(): a is not a list.")
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

fn list_rev(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => return env.type_error("Type error in a.rev(): a is not a list.")
  };
  match argv.len() {
    0 => {
      a.v[..].reverse();
      Ok(pself.clone())
    },
    n => env.argc_error(n,0,0,"rev")
  }
}

fn list_swap(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => return env.type_error("Type error in a.swap(i,j): a is not a list.")
  };
  match argv.len() {
    2 => {
      let x = match argv[0] {
        Object::Int(x)=>x,
        ref index => return env.type_error1(
          "Type error in a.swap(i,j): i is not an integer.",
          "i",index)
      };
      let y = match argv[1] {
        Object::Int(y)=>y,
        ref index => return env.type_error1(
          "Type error in a.swap(i,j): j is not an integer.",
          "j",index)
      };
      let len = a.v.len();
      let i = if x<0 {
        // #overflow-transmutation: len as isize
        let x = x as isize+len as isize;
        if x<0 {
          return env.index_error("Index error in a.swap(i,j): i is out of lower bound.");
        }else{
          x as usize
        }
      }else if x as usize >= len {
        return env.index_error("Index error in a.swap(i,j): i is out of upper bound.");
      }else{
        x as usize
      };
      let j = if y<0 {
        // #overflow-transmutation: len as isize
        let y = y as isize+len as isize;
        if y<0 {
          return env.index_error("Index error in a.swap(i,j): j is out of lower bound.");
        }else{
          y as usize
        }
      }else if y as usize >= len {
        return env.index_error("Index error in a.swap(i,j): j is out of upper bound.");
      }else{
        y as usize
      };
      a.v.swap(i,j);
      Ok(Object::Null)
    },
    n => env.argc_error(n,2,2,"swap")
  }
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("push",push,0,VARIADIC);
  m.insert_fn_plain("append",append,0,VARIADIC);
  m.insert_fn_plain("size",size,0,0);
  m.insert_fn_plain("map",map,1,1);
  m.insert_fn_plain("filter",filter,1,1);
  m.insert_fn_plain("count",count,1,1);
  m.insert_fn_plain("any",any,1,1);
  m.insert_fn_plain("all",all,1,1);
  m.insert_fn_plain("join",list_join,0,1);
  m.insert_fn_plain("chain",list_chain,0,0);
  m.insert_fn_plain("rev",list_rev,0,0);
  m.insert_fn_plain("swap",list_swap,2,2);
  m.insert("shuffle",Function::mutable(new_shuffle(),0,0));
}
