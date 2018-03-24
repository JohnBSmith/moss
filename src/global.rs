
#![allow(unused_imports)]

use std::rc::Rc;
use std::cell::RefCell;
use vm::{object_to_string, object_to_repr, RTE, Env, op_lt};
use object::{Object, Map, Table, List, Range,
  FnResult, U32String, Function, EnumFunction,
  VARIADIC, new_module, Exception,
};
use rand::Rand;
use iterable::iter;
use std::collections::HashMap;
use std::str::FromStr;
use std::f64::NAN;
use system::{History};
use compiler::Value;
use long::Long;

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
    print!("{}",try!(argv[i].string(env)));
  }
  println!();
  return Ok(Object::Null);
}

pub fn put(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",try!(argv[i].string(env)));
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
    'f' => {format!("{:.*}",n,x)},  // fixed point
    'e' => {format!("{:.*e}",n,x)}, // lower exponential
    'E' => {format!("{:.*E}",n,x)}, // upper exponential
    't' => { // fixed point, trimmed zeroes
      let mut v: Vec<char> = format!("{:.*}",n,x).chars().collect();
      loop{
        let n = v.len();
        if n<2 {break;}
        if v[n-1]=='0' || v[n-1]=='.' {
          v.pop();
        }else{
          break;
        }
      }
      return Ok(U32String::new_object(v));
    },
    _ => {
      return env.value_error("Value error in str(x,fmt,precision): fmt should be one of 'f', 'e', 'E'.");
    }
  };
  return Ok(U32String::new_object_str(&s));
}

fn fstr(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  match argv.len() {
    1 => {
      let s = try!(argv[0].string(env));
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
  match argv.len() {
    1=>{}, n=>{return env.argc_error(n,1,1,"repr");}
  }
  let s = try!(argv[0].repr(env));
  return Ok(U32String::new_object_str(&s));
}

fn sgn(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"sgn");
  }
  match argv[0] {
    Object::Int(x) => {
      return Ok(Object::Int(if x<0 {-1} else {1}));
    },
    Object::Float(x) => {
      return Ok(Object::Float(if x<0.0 {-1.0} else {1.0}));
    },
    Object::Complex(z) => {
      return Ok(Object::Float(z.abs()));
    },
    Object::Interface(ref x) => {
      return x.sgn(env);
    },
    _ => {
      return env.type_error1(
        "Type error in sgn(x): x should be of type Int, Long, Float.",
        "x",&argv[0]
      );
    }
  }
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
      return match env.eval_string(&a,"",gtab,Value::Optional) {
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
  if !env.rte().read_access(id) {
    return env.std_exception(&format!(
      "Error in load(id): Could not open file id=='{}': permission denied.",id
    ));
  }
  let s = match ::system::read_module_file(id) {
    Ok(s)=>s,
    Err(e) => return env.std_exception(&e)
  };
  let module = new_module(id);
  env.rte().clear_at_exit(module.map.clone());
  match env.eval_string(&s,id,module.map.clone(),Value::None) {
    Ok(x) => {
      Ok(match x {
        Object::Null => Object::Table(Rc::new(module)),
        x => x
      })
    },
    Err(e) => Err(e)
  }
}

fn load(env: &mut Env, id: Rc<U32String>, hot_plug: bool) -> FnResult{
  if !hot_plug {
    let m = env.rte().module_table.borrow();
    if let Some(value) = m.m.get(&Object::String(id.clone())) {
      return Ok(value.clone());
    }
  }
  let s: String = id.v.iter().collect();
  let y = match &s[..] {
    "math"  => ::math::load_math(),
    "math/la" => ::la::load_la(env),
    "cmath" => ::math::load_cmath(),
    "sys"   => ::sys::load_sys(env.rte()),
    "sf"    => ::sf::load_sf(),    
    _ => {
      try!(load_file(env,&s))
      // return index_error(&format!("Could not load module '{}'.",s));
    }
  };
  if !hot_plug {
    let mut m = env.rte().module_table.borrow_mut();
    m.m.insert(Object::String(id),y.clone());
  }
  return Ok(y);
}

fn fload(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return env.argc_error(argv.len(),1,1,"load");
  }
  match argv[0] {
    Object::String(ref s) => load(env,s.clone(),false),
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
            map: m.clone(), extra: None
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

fn float_range_to_list(env: &mut Env, r: &Range) -> FnResult {
  let a = match r.a {
    Object::Int(x) => x as f64,
    Object::Float(x) => x,
    _ => return env.type_error1(
      "Type error in list(a..b): a is not of type Float.",
      "a",&r.a)
  };
  let b = match r.b {
    Object::Int(x) => x as f64,
    Object::Float(x) => x,
    _ => return env.type_error1(
      "Type error in list(a..b): b is not of type Float.",
      "b",&r.b)
  };
  let d = match r.step {
    Object::Null => 1.0,
    Object::Int(x) => x as f64,
    Object::Float(x) => x,
    _ => return env.type_error1(
      "Type error in list(a..b: d): d is not of type Float.",
      "d",&r.step)
  };

  let q = (b-a)/d;
  let n = if q<0.0 {0} else {(q+0.001) as usize+1};

  let mut v: Vec<Object> = Vec::with_capacity(n);
  for k in 0..n {
    v.push(Object::Float(a+k as f64*d));
  }
  return Ok(List::new_object(v));
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
        Object::Float(x) => {
          return float_range_to_list(env,r);
        },
        _ => return env.type_error1(
          "Type error in list(a..b): a is not an integer.",
          "a",&r.a)
      };
      let b = match r.b {
        Object::Int(x)=>x,
        Object::Float(x) => {
          return float_range_to_list(env,r);
        },
        _ => return env.type_error1(
          "Type error in list(a..b): b is not an integer.",
          "b",&r.b)
      };
      let d = match r.step {
        Object::Null => 1,
        Object::Int(x)=>x,
        Object::Float(x) => {
          return float_range_to_list(env,r);
        },
        _ => return env.type_error1(
          "Type error in list(a..b: d): d is not an integer.",
          "d",&r.step)
      };
      if d==0 {
        return env.value_error("Value error in list(a..b: d): d==0.");
      }
      let mut n = (b-a)/d+1;
      if n<0 {n=0;}
      let mut v: Vec<Object> = Vec::with_capacity(n as usize);
      let mut k = a;
      for i in 0..n {
        v.push(Object::Int(k));
        k+=d;
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
    Object::Function(ref f) => {
      return ::iterable::to_list(env,obj,&[]);
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
      "x", &argv[0])
  }
}

fn float(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {}, n => return env.argc_error(n,1,1,"float")
  }
  match argv[0] {
    Object::Int(n) => return Ok(Object::Float(n as f64)),
    Object::Float(x) => return Ok(Object::Float(x)),
    Object::Complex(z) => return Ok(Object::Float(
      if z.im==0.0 {z.re} else {NAN}
    )),
    Object::String(ref s) => {
      return match f64::from_str(&s.v.iter().collect::<String>()) {
        Ok(value) => Ok(Object::Float(value)),
        Err(_) => env.value_error("Value error: parse error in float(s).")
      }
    },
    _ => {}
  }
  if let Some(b) = Long::downcast(&argv[0]) {
    return Ok(Object::Float(b.as_f64()));
  }
  env.type_error1(
    "Type error in float(x): cannot convert x to float.",
    "x", &argv[0])
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
            h.append(&try!(x.string(env)));
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

fn fconst(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {}, n => {return env.argc_error(n,1,1,"const");}
  }
  match argv[0] {
    Object::List(ref a) => {
      let mut a = a.borrow_mut();
      a.frozen = true;
    },
    Object::Map(ref m) => {
      let mut m = m.borrow_mut();
      m.frozen = true;
    },
    Object::Table(ref t) => {
      let mut m = t.map.borrow_mut();
      m.frozen = true;
    },
    _ => {}
  }
  return Ok(argv[0].clone());
}

fn read(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    1 => {}, n => {return env.argc_error(n,1,1,"read");}
  }
  match argv[0] {
    Object::String(ref id) => {
      let id: String = id.v.iter().collect();

      if !env.rte().read_access(&id) {
        return env.std_exception(&format!(
          "Error in read(id): Could not open file id=='{}': permission denied.",id
        ));
      }
      
      return match ::system::read_file(&id) {
        Ok(s) => Ok(U32String::new_object_str(&s)),
        Err(e) => env.std_exception(&e)
      }
    },
    ref x => env.type_error1(
      "Type error in read(id): id is not a string.",
      "id", x)
  }
}

fn zip(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
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
  for k in 0..n {
    let mut t: Vec<Object> = Vec::with_capacity(argc);
    for i in 0..argc {
      t.push(v[i].borrow().v[k].clone());
    }
    vy.push(List::new_object(t));
  }
  return Ok(List::new_object(vy));
}

fn pow(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    3 => {}, n => return env.argc_error(n,3,3,"pow")
  }
  return ::long::pow_mod(env,&argv[0],&argv[1],&argv[2]);
}

fn min(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    2 => {}, n => return env.argc_error(n,2,2,"min")
  }
  let cond = try!(op_lt(env,&argv[0],&argv[1]));
  if let Object::Bool(cond) = cond {
    return Ok(argv[if cond {0} else {1}].clone());
  }else{
    return env.type_error("Type error in min(x,y): value of x<y is not a boolean.")
  }
}

fn max(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  match argv.len() {
    2 => {}, n => return env.argc_error(n,2,2,"max")
  }
  let cond = try!(op_lt(env,&argv[0],&argv[1]));
  if let Object::Bool(cond) = cond {
    return Ok(argv[if cond {1} else {0}].clone());
  }else{
    return env.type_error("Type error in max(x,y): value of x<y is not a boolean.")
  }
}

pub fn init_rte(rte: &RTE){
  let mut gtab = rte.gtab.borrow_mut();
  gtab.insert_fn_plain("print",print,0,VARIADIC);
  gtab.insert_fn_plain("put",put,0,VARIADIC);
  gtab.insert_fn_plain("str",fstr,1,1);
  gtab.insert_fn_plain("int",fint,1,1);
  gtab.insert_fn_plain("float",float,1,1);
  gtab.insert_fn_plain("repr",repr,1,1);
  gtab.insert_fn_plain("input",input,0,2);
  gtab.insert_fn_plain("sgn",sgn,1,1);
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
  gtab.insert_fn_plain("const",fconst,1,1);
  gtab.insert_fn_plain("read",read,1,1);
  gtab.insert_fn_plain("zip",zip,0,VARIADIC);
  gtab.insert_fn_plain("pow",pow,3,3);
  gtab.insert_fn_plain("min",min,2,2);
  gtab.insert_fn_plain("max",max,2,2);
  gtab.insert("empty", Object::Empty);

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

  let type_long = rte.type_long.clone();
  gtab.insert("Long", Object::Table(type_long));
  
  let type_type_error = rte.type_type_error.clone();
  gtab.insert("TypeError", Object::Table(type_type_error));
  
  let type_value_error = rte.type_value_error.clone();
  gtab.insert("ValueError", Object::Table(type_value_error));
  
  let type_index_error = rte.type_index_error.clone();
  gtab.insert("IndexError", Object::Table(type_index_error));
}

