
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use object::{Object, FnResult, U32String, Function, Table, List, Map,
  VARIADIC, MutableFn, EnumFunction
};
use vm::Env;

pub fn update(m: &mut Map, m2: &Map){
  for (key,value) in &m2.m {
    m.m.insert(key.clone(),value.clone());
  }
}

fn fupdate(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return env.argc_error(argv.len(),1,1,"update");
  }
  match *pself {
    Object::Map(ref m) => {
      match argv[0] {
        Object::Map(ref m2) => {
          let m = &mut *m.borrow_mut();
          if m.frozen {
            return env.value_error("Value error in m.update(m2): m is frozen.");
          }
          update(m,&*m2.borrow());
          Ok(Object::Null)
        },
        _ => env.type_error("Type error in m.update(m2): m2 is not a map.")
      }
    },
    _ => env.type_error("Type error in m.update(m2): m is not a map.")
  }
}

fn values(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=0 {
    return env.argc_error(argv.len(),0,0,"values");
  }
  if let Object::Map(ref m) = *pself {
    let mut index: usize = 0;
    let mut v: Vec<Object> = m.borrow().m.values().cloned().collect();
    let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
      if index == v.len() {
        return Ok(Object::Empty);
      }else{
        index+=1;
        return Ok(v[index-1].clone());
      }
    });
    Ok(Object::Function(Rc::new(Function{
      f: EnumFunction::Mut(RefCell::new(f)),
      argc: 0, argc_min: 0, argc_max: 0,
      id: Object::Null
    })))
  }else{
    env.type_error("Type error in m.values(): m is not a map.")
  }
}

fn items(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=0 {
    return env.argc_error(argv.len(),0,0,"items");
  }
  if let Object::Map(ref m) = *pself {
    let mut index: usize = 0;
    let ref m = m.borrow().m;
    let mut keys: Vec<Object> = Vec::with_capacity(m.len());
    let mut values: Vec<Object> = Vec::with_capacity(m.len()); 
    for (key,value) in m.iter() {
      keys.push(key.clone());
      values.push(value.clone());
    }
    let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
      if index == keys.len() {
        return Ok(Object::Empty);
      }else{
        index+=1;
        let t = vec![keys[index-1].clone(),values[index-1].clone()];
        return Ok(List::new_object(t));
      }
    });
    Ok(Object::Function(Rc::new(Function{
      f: EnumFunction::Mut(RefCell::new(f)),
      argc: 0, argc_min: 0, argc_max: 0,
      id: Object::Null
    })))
  }else{
    env.type_error("Type error in m.items(): m is not a map.")
  }
}

fn clear(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=0 {
    return env.argc_error(argv.len(),0,0,"clear");
  }
  match *pself {
    Object::Map(ref m) => {
      let mut m = m.borrow_mut();
      if m.frozen {
        return env.value_error("Value error in m.clear(): m is frozen.");
      }
      m.m.clear();
      Ok(Object::Null)
    },
    _ => env.type_error("Type error in m.clear(): m is not a map.")
  }
}

fn remove(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return env.argc_error(argv.len(),1,1,"remove");
  }
  match *pself {
    Object::Map(ref m) => {
      let mut m = m.borrow_mut();
      if m.frozen {
        return env.value_error("Value error in m.remove(key): m is frozen.");
      }
      match m.m.remove(&argv[0]) {
        Some(value) => Ok(value),
        None => env.index_error("Index error in m.remove(key): key was not in m.")
      }
    },
    _ => env.type_error("Type error in m.remove(key): m is not a map.")
  }
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_plain("update",fupdate,1,1);
  m.insert_fn_plain("values",values,0,0);
  m.insert_fn_plain("items",items,0,0);
  m.insert_fn_plain("clear",clear,0,0);
  m.insert_fn_plain("remove",remove,0,0);
}
