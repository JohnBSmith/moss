
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use vm::{object_to_string, object_to_repr, RTE, Env};
use object::{Object, Map, Table, List,
  FnResult, U32String, Function, EnumFunction,
  VARIADIC, new_module, Exception,
};
use rand::Rand;
use iterable::iter;
use std::collections::HashMap;
use system::{History};

pub fn type_name(x: &Object) -> String {
  loop{
  return match *x {
    Object::Null => "null",
    Object::Bool(x) => "Bool",
    Object::Int(x) => "Int",
    Object::Float(x) => "Float",
    Object::Complex(x) => "Complex",
    Object::List(ref x) => "List",
    Object::String(ref x) => "String",
    Object::Map(ref x) => "Map",
    Object::Function(ref x) => "Function",
    Object::Range(ref x) => "Range",
    Object::Tuple(ref x) => "Tuple",
    Object::Empty => "Empty",
    _ => {break;}
  }.to_string();
  }
  match *x {
    Object::Table(ref x) => "Table object".to_string(),
    Object::Interface(ref x) => x.type_name(),
    _ => "type_name: error".to_string()
  }
}

pub fn fpanic(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  panic!()
}

pub fn print(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[i]));
  }
  println!();
  return Ok(Object::Null);
}

pub fn put(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[0]));
  }
  return Ok(Object::Null);
}

fn float_to_string(env: &Env, x: &Object, fmt: &Object, precision: &Object) -> FnResult {
  let n = match *precision {
    Object::Int(n) => if n<0 {0} else {n as usize},
    _ => return env.type_error("Type error in str(x,fmt,precision): precision is not an integer.")
  };
  let fmt = match *fmt {
    Object::String(ref s) => &s.v,
    _ => return env.type_error("Type error in str(x,fmt,precision): fmt is not a string.")
  };
  let x = match *x {
    Object::Int(n) => n as f64,
    Object::Float(f) => f,
    _ => return env.type_error("Type error in str(x,fmt,precision): x should be of type Float or Int.")
  };
  if fmt.len() != 1 {
    return env.value_error("Value error in str(x,fmt,precision): size(fmt)!=1.");
  }
  let s = match fmt[0] {
    'f' => {format!("{:.*}",n,x)},
    'e' => {format!("{:.*e}",n,x)},
    'E' => {format!("{:.*E}",n,x)},
    _ => {
      return env.value_error("Value error in str(x,fmt,precision): fmt should be one of 'f', 'e', 'E'.");
    }
  };
  return Ok(U32String::new_object_str(&s));
}

fn fstr(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    1 => {
      let s = object_to_string(&argv[0]);
      return Ok(U32String::new_object_str(&s));
    },
    3 => {
      return float_to_string(env,&argv[0],&argv[1],&argv[2]);
    },
    n => {
      return env.argc_error(n,1,1,"str");
    }
  }
}

fn repr(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"repr");
  }
  let s = object_to_repr(&argv[0]);
  return Ok(U32String::new_object_str(&s));
}

fn abs(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"abs");
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
    Object::Interface(ref x) => {
      return x.abs(env);
    },
    _ => {
      return env.type_error1(
        "Type error in abs(x): x should be of type Int, Long, Float, Complex.",
        "x",&argv[0]
      );
    }
  }
}

fn eval(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  let gtab = match argv.len() {
    1 => env.rte().pgtab.borrow().clone(),
    2 => {
      match argv[1] {
        Object::Map(ref m) => m.clone(),
        _ => return env.type_error2(
          "Type error in eval(s,m): m is not a map.",
          "s","m",&argv[0],&argv[1])
      }
    },
    n => {
      return env.argc_error(n,1,1,"eval")
    }
  };
  match argv[0] {
    Object::String(ref s) => {
      let a: String = s.v.iter().collect();
      return match env.eval_string(&a,"",gtab) {
        Ok(x) => {Ok(x)},
        Err(e) => Err(e)
      }
    },
    _ => {
      return env.type_error1(
        "Type error in eval(s): s is not a string.",
        "s", &argv[0]
      );
    }
  }
}

fn size(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"size");
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
    _ => env.type_error1(
      "Type error in size(a): cannot determine the size of a.",
      "a",&argv[0])
  }
}

fn load_file(env: &mut Env, id: &str) -> FnResult {
  let s = match ::system::read_file(id) {
    Ok(s)=>s,
    Err(()) => return env.std_exception(&format!("Error: Could not load file '{}.moss'.",id))
  };
  let module = new_module(id);
  env.rte().clear_at_exit(module.map.clone());
  match env.eval_string(&s,id,module.map.clone()) {
    Ok(x) => {
      Ok(Object::Table(Rc::new(module)))
    },
    Err(e) => Err(e)
  }
}

fn load(env: &mut Env, s: &U32String) -> FnResult{
  let s: String = s.v.iter().collect();
  if s=="math" {
    return Ok(::math::load_math());
  }else if s=="cmath" {
    return Ok(::math::load_cmath());
  }else if s=="sys" {
    return Ok(::sys::load_sys(env.rte()));
  }else if s=="la" {
    return Ok(::la::load_la());
  }else if s=="sf" {
    return Ok(::sf::load_sf());
  }else{
    return load_file(env,&s);
    // return index_error(&format!("Could not load module '{}'.",s));
  }
}

fn fload(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"load");
  }
  match argv[0] {
    Object::String(ref s) => load(env,s),
    _ => env.type_error1(
      "Type error in load(id): id is not a string.",
      "id",&argv[0])
  }
}

fn fiter(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"iter");
  }
  return iter(env,&argv[0]);
}

fn record(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len()!=1 {
    return env.argc_error(argv.len(),1,1,"record");    
  }
  match argv[0] {
    Object::Table(ref t) => {
      Ok(Object::Map(t.map.clone()))
    },
    _ => env.type_error1(
      "Type error in record(x): x is not a table.",
      "x",&argv[0])
  }
}

fn fobject(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
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
        _ => env.type_error1(
          "Type error in object(p,m): m is not a map.",
          "m",&argv[1])
      }
    },
    n => env.argc_error(n,0,0,"object")
  }
}

fn ftype(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len()!=1 {
    return env.argc_error(argv.len(),1,1,"type");
  }
  match argv[0] {
    Object::Null => Ok(Object::Null),
    Object::Bool(x) => Ok(Object::Table(env.rte().type_bool.clone())),
    Object::Int(x) => Ok(Object::Table(env.rte().type_int.clone())),
    Object::Float(x) => Ok(Object::Table(env.rte().type_float.clone())),
    Object::Complex(x) => Ok(Object::Table(env.rte().type_complex.clone())),
    Object::String(ref s) => Ok(Object::Table(env.rte().type_string.clone())),
    Object::List(ref a) => Ok(Object::Table(env.rte().type_list.clone())),
    Object::Map(ref m) => Ok(Object::Table(env.rte().type_map.clone())),
    Object::Table(ref t) => {
      Ok(t.prototype.clone())
    },
    _ => Ok(Object::Null)
  }
}

pub fn list(env: &mut Env, obj: &Object) -> FnResult {
  match *obj {
    Object::Int(n) => {
      if n<0 {
        return env.value_error("Value error in list(n): n<0.");
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
        _ => return env.type_error1(
          "Type error in list(a..b): a is not an integer.",
          "a",&r.a)
      };
      let b = match r.b {
        Object::Int(x)=>x,
        _ => return env.type_error1(
          "Type error in list(a..b): b is not an integer.",
          "b",&r.b)
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
    Object::Map(ref m) => {
      let v: Vec<Object> = m.borrow().m.keys().cloned().collect();
      Ok(List::new_object(v))
    },
    _ => env.type_error1(
      "Type error in list(x): cannot convert x into a list.",
      "x",obj)
  }
}

pub fn flist(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    1 => {}, n => return env.argc_error(n,1,1,"list")
  }
  return list(env,&argv[0]);
}

fn set(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {
      let i = &try!(iter(env,&argv[0]));
      let mut m: HashMap<Object,Object> = HashMap::new();
      loop {
        let y = try!(env.call(i,&Object::Null,&[]));
        if y == Object::Empty {break;}
        m.insert(y,Object::Null);
      }
      return Ok(Object::Map(Rc::new(RefCell::new(Map{
        m: m, frozen: false
      }))));
    },
    n => return env.argc_error(n,1,1,"set")
  }
}

fn copy(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()!=1 {
    return env.argc_error(argv.len(),1,1,"copy");
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

fn frand(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()==1 {
    match argv[0] {
      Object::Range(ref r) => {
        let a = match r.a {
          Object::Int(x)=>x,
          _ => return env.type_error1(
            "Type error in rand(a..b): a is not an integer.",
            "a",&r.a)
        };
        let b = match r.b {
          Object::Int(x)=>x,
          _ => return env.type_error1(
            "Type error in rand(a..b): b is not an integer.",
            "b",&r.b)
        };
        let mut rng = Rand::new(0);
        let f = Box::new(move |env: &mut Env, pself: &Object, argv: &[Object]| -> FnResult {
          Ok(Object::Int(rng.rand_range(a,b)))
        });
        return Ok(Function::mutable(f,0,0));
      },
      _ => return env.type_error1(
        "Type error in rand(r): r is not a range.",
        "r",&argv[0])
    }
  }else{
    return env.argc_error(argv.len(),1,1,"rand");
  }
}

fn fgtab(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()==1 {
    match argv[0] {
      Object::Function(ref f) => {
        if let EnumFunction::Std(ref fp) = f.f {
          Ok(Object::Map(fp.gtab.clone()))
        }else{
          env.type_error("Type error in gtab(f): f is not a function from Moss source code.")
        }
      },
      _ => env.type_error1(
        "Type error in gtab(f): f is not a function.",
        "f",&argv[0])
    }
  }else{
    Ok(Object::Map(env.rte().pgtab.borrow().clone()))
  }
}

fn stoi(a: &[char]) -> i32 {
  let mut y = 0;
  for x in a {
    y = 10*y+(*x as i32)-('0' as i32);
  }
  return y;
}

fn fint(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {}, n => return env.argc_error(n,1,1,"int")
  }
  match argv[0] {
    Object::Int(n) => Ok(Object::Int(n)),
    Object::String(ref s) => Ok(Object::Int(stoi(&s.v))),
    _ => env.type_error1(
      "Type error in int(x): cannot convert x to int.",
      "x",&argv[0])
  }
}

fn input(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    0 => {
      let s = match ::system::getline("") {
        Ok(s)=>s, Err(e) => return env.std_exception("Error in input(): could not obtain input.")
      };
      Ok(U32String::new_object_str(&s))
    },
    1|2 => {
      let prompt = match argv[0] {
        Object::String(ref s) => {
          s.v.iter().cloned().collect::<String>()
        },
        _ => return env.type_error1(
          "Type error in input(prompt): prompt is not a string.",
          "prompt",&argv[0])
      };
      let s = if argv.len()==2 {
        if let Object::List(ref a) = argv[1] {
          let mut h = History::new();
          for x in &a.borrow().v {
            h.append(&x.to_string());
          }
          match ::system::getline_history(&prompt,&h) {
            Ok(s)=>s, Err(e) => return env.std_exception("Error in input(): could not obtain input.")
          }
        }else{
          return env.type_error1(
            "Type error in input(prompt,history): history is not a list.",
            "history",&argv[1])
        }
      }else{
        match ::system::getline(&prompt) {
          Ok(s)=>s, Err(e) => return env.std_exception("Error in input(): could not obtain input.")
        }
      };
      Ok(U32String::new_object_str(&s))
    },
    n => {
      env.argc_error(n,0,2,"input")
    }
  }
}

fn bench(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  return Ok(Object::Null);
}

pub fn init_rte(rte: &RTE){
  let mut gtab = rte.gtab.borrow_mut();
  gtab.insert_fn_plain("print",print,0,VARIADIC);
  gtab.insert_fn_plain("put",put,0,VARIADIC);
  gtab.insert_fn_plain("str",fstr,1,1);
  gtab.insert_fn_plain("int",fint,1,1);
  gtab.insert_fn_plain("repr",repr,1,1);
  gtab.insert_fn_plain("input",input,0,2);
  gtab.insert_fn_plain("abs",abs,1,1);
  gtab.insert_fn_plain("eval",eval,1,1);
  gtab.insert_fn_plain("size",size,1,1);
  gtab.insert_fn_plain("load",fload,1,1);
  gtab.insert_fn_plain("iter",fiter,1,1);
  gtab.insert_fn_plain("record",record,1,1);
  gtab.insert_fn_plain("object",fobject,0,2);
  gtab.insert_fn_plain("type",ftype,1,1);
  gtab.insert_fn_plain("list",flist,1,1);
  gtab.insert_fn_plain("set",set,1,1);
  gtab.insert_fn_plain("copy",copy,1,1);
  gtab.insert_fn_plain("rand",frand,1,1);
  gtab.insert_fn_plain("gtab",fgtab,0,0);
  gtab.insert("empty", Object::Empty);

  gtab.insert_fn_plain("bench",bench,0,0);

  let type_bool = rte.type_bool.clone();
  gtab.insert("Bool", Object::Table(type_bool));
  
  let type_int = rte.type_int.clone();
  gtab.insert("Int", Object::Table(type_int));
  
  let type_float = rte.type_float.clone();
  gtab.insert("Float", Object::Table(type_float));
  
  let type_complex = rte.type_complex.clone();
  gtab.insert("Complex", Object::Table(type_complex));

  let type_string = rte.type_string.clone();
  ::string::init(&type_string);
  gtab.insert("String", Object::Table(type_string));

  let type_list = rte.type_list.clone();
  ::list::init(&type_list);
  gtab.insert("List", Object::Table(type_list));

  let type_map = rte.type_map.clone();
  ::map::init(&type_map);
  gtab.insert("Map", Object::Table(type_map));

  let type_function = rte.type_function.clone();
  ::function::init(&type_function);
  gtab.insert("Function", Object::Table(type_function));

  let type_iterable = rte.type_iterable.clone();
  ::iterable::init(&type_iterable);
  gtab.insert("Iterable", Object::Table(type_iterable));
}

