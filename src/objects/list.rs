
#[allow(unused_imports)]
use object::{Object, FnResult, U32String, Function, Table, List,
  type_error, argc_error, index_error,
  VARIADIC, MutableFn
};
use vm::Env;
use rand::Rand;

fn push(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      let mut a = a.borrow_mut();
      for i in 0..argv.len() {
        a.v.push(argv[i].clone());
      }
      Ok(())
    },
    _ => type_error("Type error in a.push(x): a is not a list.")
  }
}

fn size(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 0 {
    return argc_error(argv.len(),0,0,"size");
  }
  match *pself {
    Object::List(ref a) => {
      *ret = Object::Int(a.borrow().v.len() as i32);
      Ok(())
    },
    _ => type_error("Type error in a.size(): a is not a list.")
  }
}

fn map(env: &mut Env, ret: &mut Object,
  pself: &Object, argv: &[Object]
) -> FnResult
{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"map");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.map(f): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::with_capacity(a.v.len());
  for x in &a.v {
    let mut y = Object::Null;
    try!(env.call(&argv[0],&mut y,&Object::Null,&[x.clone()]));
    v.push(y);
  }
  *ret = List::new_object(v);
  return Ok(());
}

fn filter(env: &mut Env, ret: &mut Object,
  pself: &Object, argv: &[Object]
) -> FnResult
{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"filter");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.filter(p): a is not a list.");}
  };
  let mut v: Vec<Object> = Vec::new();
  for x in &a.v {
    let mut y = Object::Null;
    try!(env.call(&argv[0],&mut y,&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.filter(p): return value of p is not of boolean type.")
    };
    if condition {
      v.push(x.clone());
    }
  }
  *ret = List::new_object(v);
  return Ok(());
}

fn count(env: &mut Env, ret: &mut Object,
  pself: &Object, argv: &[Object]
) -> FnResult
{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"count");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.count(p): a is not a list.");}
  };
  let mut k: i32 = 0;
  for x in &a.v {
    let mut y = Object::Null;
    try!(env.call(&argv[0],&mut y,&Object::Null,&[x.clone()]));
    let condition = match y {
      Object::Bool(u)=>u,
      _ => return type_error("Type error in a.count(p): return value of p is not of boolean type.")
    };
    if condition {k+=1;}
  }
  *ret = Object::Int(k);
  return Ok(());
}

fn each(env: &mut Env, ret: &mut Object,
  pself: &Object, argv: &[Object]
) -> FnResult {
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"each");
  }
  let mut a = match *pself {
    Object::List(ref a) => a.borrow_mut(),
    _ => {return type_error("Type error in a.each(f): a is not a list.");}
  };
  let mut y = Object::Null;
  for x in &a.v {
    try!(env.call(&argv[0],&mut y,&Object::Null,&[x.clone()]));
  }
  return Ok(());
}

fn new_shuffle() -> MutableFn {
  let mut rng = Rand::new(0);
  return Box::new(move
  |ret: &mut Object, pself: &Object, argv: &[Object]| -> FnResult{
    if argv.len() != 0 {
      return argc_error(argv.len(),0,0,"shuffle");
    }
    match *pself {
      Object::List(ref a) => {
        let mut ba = a.borrow_mut();
        rng.shuffle(&mut ba.v);
        *ret = Object::List(a.clone());
        Ok(())
      },
      _ => type_error("Type error in a.shuffle(): a is not a list.")
    }
  });
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert("push",   Function::plain(push,0,VARIADIC));
  m.insert("size",   Function::plain(size,0,0));
  m.insert("map",    Function::env(map,1,1));
  m.insert("filter", Function::env(filter,1,1));
  m.insert("count",  Function::env(count,1,1));
  m.insert("each",   Function::env(each,1,1));
  m.insert("shuffle",Function::mutable(new_shuffle(),0,0));
}
