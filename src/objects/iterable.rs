
use std::rc::Rc;
use std::cell::RefCell;
use object::{
  Object, Table, List,
  FnResult, Function, EnumFunction,
  type_error, argc_error,
};
use vm::Env;

pub fn iter(x: &Object) -> FnResult{
  match *x {
    Object::Function(ref f) => {
      Ok(Object::Function(f.clone()))
    },
    Object::Range(ref r) => {
      let mut a = match r.a {
        Object::Int(a)=>a,
        _ => {return type_error("Type error in iter(a..b): a is not an integer.");}
      };
      let f: Box<FnMut(&Object,&[Object])->FnResult> = match r.b {
        Object::Int(b) => {
          Box::new(move |pself: &Object, argv: &[Object]| -> FnResult{
            return if a<=b {
              a+=1;
              Ok(Object::Int(a-1))
            }else{
              Ok(Object::Empty)
            }
          })
        },
        Object::Null => {
          Box::new(move |pself: &Object, argv: &[Object]| -> FnResult{
            a+=1; Ok(Object::Int(a-1))
          })
        },
        _ => {return type_error("Type error in iter(a..b): b is not an integer.");}
      };
      return Ok(Object::Function(Rc::new(Function{
        f: EnumFunction::Mut(RefCell::new(f)),
        argc: 0, argc_min: 0, argc_max: 0,
        id: Object::Null
      })));
    },
    Object::List(ref a) => {
      let mut index: usize = 0;
      let a = a.clone();
      let f = Box::new(move |pself: &Object, argv: &[Object]| -> FnResult{
        let a = a.borrow();
        if index == a.v.len() {
          return Ok(Object::Empty);
        }else{
          index+=1;
          return Ok(a.v[index-1].clone());

        }
      });
      Ok(Object::Function(Rc::new(Function{
        f: EnumFunction::Mut(RefCell::new(f)),
        argc: 0, argc_min: 0, argc_max: 0,
        id: Object::Null
      })))
    },
    _ => type_error("Type error in iter(x): x is not iterable.")
  }
}

fn list(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let i = &try!(iter(pself));
  if argv.len() == 1 {
    match argv[0] {
      Object::Int(n) => {
        let mut v: Vec<Object> = Vec::new();
        for _ in 0..n {
          let y = try!(env.call(i,&Object::Null,&[]));
          if y == Object::Empty {
            break;
          }else{
            v.push(y);
          }
        }
        return Ok(List::new_object(v));
      },
      _ => return type_error("Type error in i.list(n): n is not an integer.")
    }
  }else if argv.len() == 0 {
    let mut v: Vec<Object> = Vec::new();
    loop{
      let y = try!(env.call(i,&Object::Null,&[]));
      if y == Object::Empty {
        break;
      }else{
        v.push(y);
      }
    }
    return Ok(List::new_object(v));  
  }else{
    return argc_error(argv.len(),1,1,"list");
  }
}

fn map(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"map");
  }
  let i = try!(iter(pself));
  let f = argv[0].clone();
  let g = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
    let x = try!(env.call(&i,&Object::Null,&[]));
    return if x == Object::Empty {
      Ok(x)
    }else{
      let y = try!(env.call(&f,&Object::Null,&[x]));
      Ok(y)
    };
  });
  Ok(Object::Function(Rc::new(Function{
    f: EnumFunction::Env(RefCell::new(g)),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  })))
}

fn filter(pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"filter");
  }
  let i = try!(iter(pself));
  let f = argv[0].clone();
  let g = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
    loop{
      let x = try!(env.call(&i,&Object::Null,&[]));
      if x == Object::Empty {
        return Ok(x);
      }else{
        let y = try!(env.call(&f,&Object::Null,&[x.clone()]));
        match y {
          Object::Bool(u) => {
            if u {return Ok(x);}
          },
          _ => return type_error("Type error in i.filter(p): return value of p is not of boolean type.")
        }
      }
    }
  });
  Ok(Object::Function(Rc::new(Function{
    f: EnumFunction::Env(RefCell::new(g)),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  })))
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_env  ("list",list,0,1);
  m.insert_fn_plain("map",map,1,1);
  m.insert_fn_plain("filter",filter,1,1);
}

