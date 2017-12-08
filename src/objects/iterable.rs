
use std::rc::Rc;
use std::cell::RefCell;
use object::{
  Object, Table, List, U32String,
  FnResult, Function, EnumFunction,
  type_error, argc_error, value_error
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
      let f: Box<FnMut(&mut Env,&Object,&[Object])->FnResult> = match r.b {
        Object::Int(b) => {
          Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
            return if a<=b {
              a+=1;
              Ok(Object::Int(a-1))
            }else{
              Ok(Object::Empty)
            }
          })
        },
        Object::Null => {
          Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
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
      let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
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
    Object::Map(ref m) => {
      let mut index: usize = 0;
      let mut v: Vec<Object> = m.borrow().m.keys().cloned().collect();
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
    },
    Object::String(ref s) => {
      let mut index: usize = 0;
      let s = s.clone();
      let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult{
        if index == s.v.len() {
          return Ok(Object::Empty);
        }else{
          index+=1;
          return Ok(Object::String(Rc::new(U32String{
            v: vec![s.v[index-1]]
          })));
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

fn list_comprehension(env: &mut Env, i: &Object, f: &Object) -> FnResult {
  let mut v: Vec<Object> = Vec::new();
  loop{
    let x = try!(env.call(i,&Object::Null,&[]));
    if x == Object::Empty {break;}
    let y = try!(env.call(f,&Object::Null,&[x]));
    if y != Object::Null {
      v.push(y);
    }
  }
  Ok(List::new_object(v))
}

fn list(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let i = &try!(iter(pself));
  match argv.len() {
    0 => {
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
    },
    1 => {
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
        Object::Function(ref f) => {
          return list_comprehension(env,i,&argv[0]);
        },
        _ => return type_error("Type error in i.list(n): n is not an integer.")
      }    
    },
    n => {
      return argc_error(n,1,1,"list");
    }
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
    f: EnumFunction::Mut(RefCell::new(g)),
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
    f: EnumFunction::Mut(RefCell::new(g)),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  })))
}

fn each(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  let i = &try!(iter(pself));
  if argv.len() == 1 {
    loop{
      let y = try!(env.call(i,&Object::Null,&[]));
      if y == Object::Empty {
        break;
      }else{
        try!(env.call(&argv[0],&Object::Null,&[y]));
      }
    }
    return Ok(Object::Null);
  }else{
    return argc_error(argv.len(),1,1,"each");
  }
}

fn any(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"any");  
  }
  let i = &try!(iter(pself));
  let p = &argv[0];
  loop{
    let x = try!(env.call(i,&Object::Null,&[]));
    if x == Object::Empty {
      break;
    }else{
      let y = try!(env.call(p,&Object::Null,&[x]));
      if let Object::Bool(yb) = y {
        if yb {return Ok(Object::Bool(true));}
      }else{
        return type_error("Type error in i.any(p): return value of p is not of boolean type.");
      }
    }
  }
  return Ok(Object::Bool(false));
}

fn all(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"all");  
  }
  let i = &try!(iter(pself));
  let p = &argv[0];
  loop{
    let x = try!(env.call(i,&Object::Null,&[]));
    if x == Object::Empty {
      break;
    }else{
      let y = try!(env.call(p,&Object::Null,&[x]));
      if let Object::Bool(yb) = y {
        if !yb {return Ok(Object::Bool(false));}
      }else{
        return type_error("Type error in i.all(p): return value of p is not of boolean type.");
      }
    }
  }
  return Ok(Object::Bool(true));
}

fn count(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return argc_error(argv.len(),1,1,"count");  
  }
  let i = &try!(iter(pself));
  let p = &argv[0];
  let mut k: i32 = 0;
  loop{
    let x = try!(env.call(i,&Object::Null,&[]));
    if x == Object::Empty {
      break;
    }else{
      let y = try!(env.call(p,&Object::Null,&[x]));
      if let Object::Bool(yb) = y {
        if yb {k+=1;}
      }else{
        return type_error("Type error in i.count(p): return value of p is not of boolean type.");
      }
    }
  }
  return Ok(Object::Int(k));
}

fn chunks(pself: &Object, argv: &[Object]) -> FnResult{
  let n = match argv.len() {
    1 => {
      match argv[0] {
        Object::Int(x)=>{
          if x>=0 {x as usize}
          else {return value_error("Value error in a.chunks(n): n<0.")}
        },
        _ => return type_error("Type error in a.chunks(n): n is not an integer.")
      }
    },
    n => return argc_error(n,1,1,"chunks")
  };
  let i = try!(iter(pself));
  let mut empty = false;
  let g = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
    if empty {return Ok(Object::Empty);}
    let mut v: Vec<Object> = Vec::with_capacity(n);
    for _ in 0..n {
      let y = try!(env.call(&i,&Object::Null,&[]));
      if y==Object::Empty {empty=true; break;}
      v.push(y);
    }
    if v.len()==0 {
      Ok(Object::Empty)
    }else{
      Ok(List::new_object(v))
    }
  });
  Ok(Object::Function(Rc::new(Function{
    f: EnumFunction::Mut(RefCell::new(g)),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  })))
}

pub fn init(t: &Table){
  let mut m = t.map.borrow_mut();
  m.insert_fn_env  ("list",list,0,1);
  m.insert_fn_env  ("each",each,1,1);
  m.insert_fn_env  ("any",any,1,1);
  m.insert_fn_env  ("all",all,1,1);
  m.insert_fn_env  ("count",count,1,1);
  m.insert_fn_plain("map",map,1,1);
  m.insert_fn_plain("filter",filter,1,1);
  m.insert_fn_plain("chunks",chunks,1,1);
}

