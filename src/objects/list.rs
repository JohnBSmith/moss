
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;

use object::{Object, FnResult, U32String, Function, Table, List,
  VARIADIC, MutableFn, Exception
};
use vm::Env;
use rand::Rand;
use global::list;

fn push(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          if a.frozen {
            return env.value_error("Value error in a.pop(): a is frozen.");
          }
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
    ref a => env.type_error1("Type error in a.push(x): a is not a list.","a",a)
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

fn pop_at_index(env: &mut Env, v: &mut Vec<Object>, index: &Object) -> FnResult {
  let len = v.len();
  let i = match *index {
    Object::Int(i) => if i<0 {
      let i = i as isize+len as isize;
      if i<0 {
        return env.index_error("Index error in a.pop(i): i is out of lower bound.");
      } else {i as usize}
    } else {i as usize},
    ref i => return env.type_error1("Type error in a.pop(i): is not an integer.","i",i)
  };
  if i<v.len() {
    return Ok(v.remove(i));
  }else{
    return env.index_error("Index error in a.pop(i): i is is out of upper bound.");
  }
}

fn pop(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          if a.frozen {
            return env.value_error("Value error in a.pop(): a is frozen.");
          }
          if argv.len()>0 {
            return pop_at_index(env,&mut a.v,&argv[0]);
          }else{
            match a.v.pop() {
              Some(x) => Ok(x),
              None => {
                env.value_error("Value error in a.pop(): a is empty.")
              }
            }
          }
        },
        Err(_) => {env.std_exception(
          "Memory error in a.pop(): internal buffer of a is aliased.\n\
           Try to replace a by copy(a) at some place."
        )}
      }
    },
    ref a => env.type_error1("Type error in a.pop(): a is not a list.","a",a)
  }
}

fn insert(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    2 => {}, n => return env.argc_error(n,0,0,"insert")
  }
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          let index = match argv[0] {
            Object::Int(i) => if i<0 {
              let i = i as isize+a.v.len() as isize;
              if i<0 {
                return env.index_error("Index error in a.insert(i,x): i is out of lower bound.");
              } else {i as usize}
            } else {i as usize},
            ref i => return env.type_error1("Type error in a.insert(i,x): i is not an integer.","i",i)
          };
          if a.frozen {
            return env.value_error("Value error in a.pop(): a is frozen.");
          }
          if index < a.v.len() {
            a.v.insert(index,argv[1].clone());
          }else{
            return env.index_error("Index error in a.insert(i,x): i is out of upper bound.");
          }
          return Ok(Object::Null);
        },
        Err(_) => {
          return env.std_exception("Memory error in a.insert(i,x): internal buffer of a is aliased.")
        }
      }
    },
    ref a => return env.type_error1("Type error in a.insert(i,x): a is not a list.","a",a)
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

pub fn duplicate(a: &Rc<RefCell<List>>, n: usize) -> Object {
  let a = a.borrow();
  let size = a.v.len()*n;
  let mut v: Vec<Object> = Vec::with_capacity(size);
  for _ in 0..n {
    for x in &a.v {
      v.push(x.clone());
    }
  }
  return List::new_object(v);
} 

pub fn map_fn(env: &mut Env, f: &Object, argv: &[Object]) -> FnResult {
  let argc = argv.len();
  if argc==0 {
    return Ok(List::new_object(Vec::new()));
  }
  let mut v: Vec<Rc<RefCell<List>>> = Vec::with_capacity(argc);
  for i in 0..argc {
    match argv[i] {
      Object::List(ref a) => v.push(a.clone()),
      ref a => {
        let y = try!(list(env,a));
        // todo: traceback
        v.push(match y {Object::List(a) => a, _ => unreachable!()});
      }
    }
  }
  let n = v[0].borrow().v.len();
  for i in 0..argc {
    if n != v[i].borrow().v.len() {
      return env.type_error("Type error in f[a1,...,an]: all lists must have the same size.");
    }
  }
  
  let null = &Object::Null;
  let mut vy: Vec<Object> = Vec::with_capacity(argc);
  let mut args: Vec<Object> = vec![Object::Null; argc];
  for k in 0..n {
    for i in 0..argc {
      args[i] = v[i].borrow().v[k].clone();
    }
    let y = try!(env.call(f,null,&args));
    vy.push(y);
  }
  return Ok(List::new_object(vy));
}

fn clear(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match *pself {
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(mut a) => {
          if a.frozen {
            return env.value_error("Value error in a.clear(): a is frozen.");
          }
          match argv.len() {
            0 => {a.v.clear();},
            1 => {
              let n = match argv[0] {
                Object::Int(n) => if n<0 {0} else {n as usize},
                _ => return env.type_error("Type error in a.clear(n): n is not an integer.")
              };
              a.v.truncate(n);
            },
            n => return env.argc_error(n,0,1,"clear")
          }
          Ok(Object::Null)        
        },
        Err(_) => {env.std_exception(
          "Memory error in a.clear(x): internal buffer of a was aliased."
        )}
      }
    },
    ref a => env.type_error1("Type error in a.clear(): a is not a list.","a",a)
  }
}

pub fn cartesian_product(a: &List, b: &List) -> Object {
  let mut v: Vec<Object> = Vec::with_capacity(a.v.len()*b.v.len());
  for x in &a.v {
    for y in &b.v {
      v.push(List::new_object(vec![x.clone(),y.clone()]));
    }
  }
  return List::new_object(v);
}

pub fn cartesian_power(v: &Vec<Object>, n: i32) -> Object {
  let n = if n<0 {0} else {n as u32};
  let m = v.len();
  let len = m.pow(n);
  let mut y: Vec<Vec<Object>> = Vec::with_capacity(len);
  if m==0 {
    let y = List::new_object(Vec::new());
    if n==0 {
      return List::new_object(vec![y]);
    }else{
      return y;
    }
  }
  for _ in 0..len {
    y.push(Vec::new());
  }
  let mut k = len/m;
  let mut count = 1;
  for _ in 0..n {
    let mut j=0;
    for _ in 0..count {
      for index in 0..m {
        for _ in 0..k {
          y[j].push(v[index].clone());
          j+=1;
        }
      }
    }
    count = count*m;
    k = k/m;
  }
  return List::new_object(
    y.into_iter().map(|v| List::new_object(v)).collect()
  );
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("push",push,0,VARIADIC);
  m.insert_fn_plain("append",append,0,VARIADIC);
  m.insert_fn_plain("pop",pop,0,0);
  m.insert_fn_plain("insert",insert,2,2);
  m.insert_fn_plain("size",size,0,0);
  m.insert_fn_plain("map",map,1,1);
  m.insert_fn_plain("filter",filter,1,1);
  m.insert_fn_plain("chain",list_chain,0,0);
  m.insert_fn_plain("rev",list_rev,0,0);
  m.insert_fn_plain("swap",list_swap,2,2);
  m.insert_fn_plain("clear",clear,0,1);
  m.insert("shuffle",Function::mutable(new_shuffle(),0,0));
}
