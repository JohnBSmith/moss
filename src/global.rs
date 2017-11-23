
use std::rc::Rc;
use std::cell::RefCell;
use vm::{object_to_string, object_to_repr, RTE};
use object::{Object, Map, Table, List,
  FnResult, U32String, Function, EnumFunction,
  type_error, argc_error, index_error, value_error,
  VARIADIC
};

pub fn print(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[i]));
  }
  println!();
  return Ok(());
}

pub fn put(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[0]));
  }
  return Ok(());
}

pub fn fstr(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"str");
  }
  let s = object_to_string(&argv[0]);
  *ret = U32String::new_object_str(&s);
  return Ok(());
}

pub fn repr(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"repr");
  }
  let s = object_to_repr(&argv[0]);
  *ret = U32String::new_object_str(&s);
  return Ok(());
}

pub fn abs(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"abs");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Int(x.abs());
      return Ok(());
    },
    Object::Float(x) => {
      *ret = Object::Float(x.abs());
      return Ok(());
    },
    Object::Complex(z) => {
      *ret = Object::Float(z.abs());
      return Ok(());
    },
    _ => {
      return type_error("Type error in abs(x): x is not an int, float, complex.");
    }
  }
}

pub fn fpanic(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  panic!()
}

pub fn eval(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"eval");
  }
  match argv[0] {
    Object::String(ref s) => {
      let a: String = s.v.iter().collect();

      let i = ::Interpreter::new();
      let gtab = Map::new();
      init_gtab(&mut gtab.borrow_mut(),&i.env);

      return match i.eval_string(&a,"",gtab) {
        Ok(x) => {*ret=x; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {
      return type_error("Type error in eval(s): s is not a string.");
    }
  }
}

pub fn size(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"size");
  }
  match argv[0] {
    Object::List(ref a) => {
      *ret = Object::Int(a.borrow().v.len() as i32);
      Ok(())
    },
    Object::Map(ref m) => {
      *ret = Object::Int(m.borrow().m.len() as i32);
      Ok(())
    },
    Object::String(ref s) => {
      *ret = Object::Int(s.v.len() as i32);
      Ok(())
    },
    _ => type_error("Type error in size(a): cannot determine the size of a.")
  }
}

fn load(ret: &mut Object, s: &U32String) -> FnResult{
  let s: String = s.v.iter().collect();
  if s=="math" {
    *ret = ::math::load_math();
  }else if s=="cmath" {
    *ret = ::math::load_cmath();
  }else{
    return index_error(&format!("Could not load module '{}'.",s));
  }
  return Ok(());
}

pub fn fload(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"load");
  }
  match argv[0] {
    Object::String(ref s) => load(ret,s),
    _ => type_error("Type error in load(id): id is not a string.")
  }
}

pub fn iter(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"iter");
  }
  match argv[0] {
    Object::Range(ref r) => {
      let mut a = match r.a {
        Object::Int(a)=>a,
        _ => {return type_error("Type error in iter(a..b): a is not an integer.");}
      };
      let b = match r.b {
        Object::Int(b)=>b,
        _ => {return type_error("Type error in iter(a..b): b is not an integer.");}
      };
      let f: Box<FnMut(&mut Object, &Object, &[Object])->FnResult>
        = Box::new(move |ret: &mut Object, pself: &Object, argv: &[Object]| -> FnResult{
        if a<=b {
          a+=1;
          *ret = Object::Int(a-1);
        }else{
          *ret = Object::Empty;
        }
        Ok(())
      });
      *ret = Object::Function(Rc::new(Function{
        f: EnumFunction::Mut(RefCell::new(f)),
        argc: 0, argc_min: 0, argc_max: 0
      }));
      Ok(())
    },
    Object::List(ref a) => {
      let mut index: usize = 0;
      let a = a.clone();
      let f = Box::new(move |ret: &mut Object, pself: &Object, argv: &[Object]| -> FnResult{
        let a = a.borrow();
        if index == a.v.len() {
          *ret = Object::Empty;
        }else{
          *ret = a.v[index].clone();
          index+=1;
        }
        Ok(())
      });
      *ret = Object::Function(Rc::new(Function{
        f: EnumFunction::Mut(RefCell::new(f)),
        argc: 0, argc_min: 0, argc_max: 0
      }));
      Ok(())
    },
    _ => type_error("Type error in iter(x): x is not iterable.")
  }
}

pub fn record(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"record");    
  }
  match argv[0] {
    Object::Table(ref t) => {
      *ret = Object::Map(t.map.clone());
      Ok(())
    },
    _ => type_error("Type error in record(x): x is not a table.")
  }
}

pub fn fobject(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    0 => {
      *ret = Object::Table(Table::new(Object::Null));
      Ok(())
    },
    1 => {
      *ret = Object::Table(Table::new(argv[0].clone()));
      Ok(())    
    },
    2 => {
      match argv[1] {
        Object::Map(ref m) => {
          *ret = Object::Table(Rc::new(Table{
            prototype: argv[0].clone(),
            map: m.clone()
          }));
          Ok(())
        },
        _ => type_error("Type error in object(p,m): m is not a map.")
      }
    },
    n => argc_error(n,0,0,"object")
  }
}

pub fn flist(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
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
      *ret = List::new_object(v);
      Ok(())
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
      *ret = List::new_object(v);
      Ok(())
    },
    Object::List(ref a) => {
      *ret = Object::List(a.clone());
      Ok(())
    },
    _ => type_error("Type error in list(r): r is not a range.")
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
  gtab.insert("load",  Function::plain(fload,1,1));
  gtab.insert("iter",  Function::plain(iter,1,1));
  gtab.insert("record",Function::plain(record,1,1));
  gtab.insert("object",Function::plain(fobject,0,2));
  gtab.insert("list",  Function::plain(flist,1,1));

  let list_type = env.list.clone();
  ::list::init(&list_type);
  gtab.insert("List", Object::Table(list_type));
}

