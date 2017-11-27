
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use vm::{object_to_string, object_to_repr, RTE, Env};
use object::{Object, Map, Table, List,
  FnResult, U32String, Function, EnumFunction,
  type_error, argc_error, index_error, value_error,
  VARIADIC
};
use rand::Rand;

pub fn fpanic(pself: &Object, argv: &[Object]) -> FnResult{
  panic!()
}

pub fn print(pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[i]));
  }
  println!();
  return Ok(Object::Null);
}

pub fn put(pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[0]));
  }
  return Ok(Object::Null);
}

fn fstr(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"str");
  }
  let s = object_to_string(&argv[0]);
  return Ok(U32String::new_object_str(&s));
}

fn repr(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"repr");
  }
  let s = object_to_repr(&argv[0]);
  return Ok(U32String::new_object_str(&s));
}

fn abs(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"abs");
  }
  match argv[0] {
    Object::Int(x) => {
      return Ok(Object::Int(x.abs()));
    },
    Object::Float(x) => {
      return Ok(Object::Float(x.abs()));
    },
    Object::Complex(z) => {
      return Ok(Object::Float(z.abs()));
    },
    _ => {
      return type_error("Type error in abs(x): x is not an int, float, complex.");
    }
  }
}

fn eval(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"eval");
  }
  match argv[0] {
    Object::String(ref s) => {
      let a: String = s.v.iter().collect();

      let i = ::Interpreter::new();
      let gtab = Map::new();
      init_gtab(&mut gtab.borrow_mut(),&i.rte);

      return match i.eval_string(&a,"",gtab) {
        Ok(x) => {Ok(x)},
        Err(e) => Err(e)
      }
    },
    _ => {
      return type_error("Type error in eval(s): s is not a string.");
    }
  }
}

fn size(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"size");
  }
  match argv[0] {
    Object::List(ref a) => {
      Ok(Object::Int(a.borrow().v.len() as i32))
    },
    Object::Map(ref m) => {
      Ok(Object::Int(m.borrow().m.len() as i32))
    },
    Object::String(ref s) => {
      Ok(Object::Int(s.v.len() as i32))
    },
    _ => type_error("Type error in size(a): cannot determine the size of a.")
  }
}

fn load(s: &U32String, rte: &Rc<RTE>) -> FnResult{
  let s: String = s.v.iter().collect();
  if s=="math" {
    return Ok(::math::load_math());
  }else if s=="cmath" {
    return Ok(::math::load_cmath());
  }else if s=="sys" {
    return Ok(::sys::load_sys(rte));
  }else{
    return index_error(&format!("Could not load module '{}'.",s));
  }
}

fn fload(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"load");
  }
  match argv[0] {
    Object::String(ref s) => load(s,env.rte()),
    _ => type_error("Type error in load(id): id is not a string.")
  }
}

fn iter(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"iter");
  }
  return ::iterable::iter(&argv[0]);
}

fn record(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"record");    
  }
  match argv[0] {
    Object::Table(ref t) => {
      Ok(Object::Map(t.map.clone()))
    },
    _ => type_error("Type error in record(x): x is not a table.")
  }
}

fn fobject(pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    0 => {
      Ok(Object::Table(Table::new(Object::Null)))
    },
    1 => {
      Ok(Object::Table(Table::new(argv[0].clone())))
    },
    2 => {
      match argv[1] {
        Object::Map(ref m) => {
          Ok(Object::Table(Rc::new(Table{
            prototype: argv[0].clone(),
            map: m.clone()
          })))
        },
        _ => type_error("Type error in object(p,m): m is not a map.")
      }
    },
    n => argc_error(n,0,0,"object")
  }
}

pub fn flist(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"list");
  }
  match argv[0] {
    Object::Int(n) => {
      if n<0 {
        return value_error("Value error in list(n): n<0.");
      }
      let mut v: Vec<Object> = Vec::with_capacity(n as usize);
      for i in 0..n {
        v.push(Object::Int(i));
      }
      Ok(List::new_object(v))
    },
    Object::Range(ref r) => {
      let a = match r.a {
        Object::Int(x)=>x,
        _ => return type_error("Type error in list(a..b): a is not an integer.")
      };
      let b = match r.b {
        Object::Int(x)=>x,
        _ => return type_error("Type error in list(a..b): b is not an integer.")
      };
      let mut n = b-a+1;
      if n<0 {n=0;}
      let mut v: Vec<Object> = Vec::with_capacity(n as usize);
      for i in a..b+1 {
        v.push(Object::Int(i));
      }
      Ok(List::new_object(v))
    },
    Object::List(ref a) => {
      Ok(Object::List(a.clone()))
    },
    _ => type_error("Type error in list(r): r is not a range.")
  }
}

fn copy(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"copy");
  }
  match argv[0] {
    Object::List(ref a) => {
      Ok(List::new_object(a.borrow().v.clone()))
    },
    Object::Map(ref m) => {
      panic!();
    },
    ref x => {
      Ok(x.clone())
    }
  }
}

fn frand(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()==1 {
    match argv[0] {
      Object::Range(ref r) => {
        let a = match r.a {
          Object::Int(x)=>x,
          _ => return type_error("Type error in rand(a..b): a is not an integer.")
        };
        let b = match r.b {
          Object::Int(x)=>x,
          _ => return type_error("Type error in rand(a..b): b is not an integer.")
        };
        let mut rng = Rand::new(0);
        let f = Box::new(move |pself: &Object, argv: &[Object]| -> FnResult {
          Ok(Object::Int(rng.rand_range(a,b)))
        });
        return Ok(Function::mutable(f,0,0));
      },
      _ => return type_error("Type error in rand(r): r is not a range.")
    }
  }else{
    return argc_error(argv.len(),1,1,"rand");
  }
}

pub fn init_gtab(gtab: &mut Map, env: &RTE){
  gtab.insert("print", Function::plain(print,0,VARIADIC));
  gtab.insert("put",   Function::plain(put,0,VARIADIC));
  gtab.insert("str",   Function::plain(fstr,1,1));
  gtab.insert("repr",  Function::plain(repr,1,1));
  gtab.insert("abs",   Function::plain(abs,1,1));
  gtab.insert("eval",  Function::plain(eval,1,1));
  gtab.insert("size",  Function::plain(size,1,1));
  gtab.insert("load",  Function::env(fload,1,1));
  gtab.insert("iter",  Function::plain(iter,1,1));
  gtab.insert("record",Function::plain(record,1,1));
  gtab.insert("object",Function::plain(fobject,0,2));
  gtab.insert("list",  Function::plain(flist,1,1));
  gtab.insert("copy",  Function::plain(copy,1,1));
  gtab.insert("rand",  Function::plain(frand,1,1));
  gtab.insert("empty", Object::Empty);

  let type_list = env.list.clone();
  ::list::init(&type_list);
  gtab.insert("List", Object::Table(type_list));

  let type_function = env.function.clone();
  ::function::init(&type_function);
  gtab.insert("Function", Object::Table(type_function));
  
  let type_iterable = env.iterable.clone();
  ::iterable::init(&type_iterable);
  gtab.insert("Iterable", Object::Table(type_iterable));
}

