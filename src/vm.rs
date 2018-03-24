
use std::rc::Rc;
use std::cell::{Cell,RefCell};
use std::mem::replace;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs::File;
use std::io::Read;
use std::fmt::Write;

use object::{
  Object, Map, List, Function, EnumFunction, StandardFn,
  FnResult, OperatorResult, Exception, Table, Range, U32String,
  VARIADIC
};
use complex::Complex64;
use long::Long;
use format::u32string_format;
use global::type_name;

// use ::Interpreter;
use system;
use compiler;

// byte code size
// byte code+argument size
// byte code+argument+argument size
pub const BCSIZE: usize = 1;
pub const BCASIZE: usize = 2;
pub const BCAASIZE: usize = 3;

pub mod bc{
  pub const NULL: u8 = 00;
  pub const OF: u8 = 01;
  pub const FALSE:u8 = 02;
  pub const TRUE: u8 = 03;
  pub const INT:  u8 = 04;
  pub const FLOAT:u8 = 05;
  pub const IMAG: u8 = 06;
  pub const NEG:  u8 = 07;
  pub const ADD:  u8 = 08;
  pub const SUB:  u8 = 09;
  pub const MPY:  u8 = 10;
  pub const DIV:  u8 = 11;
  pub const IDIV: u8 = 12;
  pub const POW:  u8 = 13;
  pub const EQ:   u8 = 14;
  pub const NE:   u8 = 15;
  pub const IS:   u8 = 16;
  pub const ISNOT:u8 = 17;
  pub const IN:   u8 = 18;
  pub const NOTIN:u8 = 19; 
  pub const LT:   u8 = 20;
  pub const GT:   u8 = 21;
  pub const LE:   u8 = 22;
  pub const GE:   u8 = 23;
  pub const LIST: u8 = 24;
  pub const MAP:  u8 = 25;
  pub const LOAD: u8 = 26;
  pub const STORE:u8 = 27;
  pub const JMP:  u8 = 28;
  pub const JZ:   u8 = 29;
  pub const JNZ:  u8 = 30;
  pub const CALL: u8 = 31;
  pub const RET:  u8 = 32;
  pub const STR:  u8 = 33;
  pub const FN:   u8 = 34;
  pub const FNSEP:u8 = 35;
  pub const POP:  u8 = 36;
  pub const LOAD_LOCAL: u8 = 37;
  pub const LOAD_ARG: u8 = 38;
  pub const LOAD_CONTEXT: u8 = 39;
  pub const STORE_LOCAL: u8 = 40;
  pub const STORE_ARG: u8 = 41;
  pub const STORE_CONTEXT: u8 = 42;
  pub const FNSELF: u8 = 43;
  pub const GET_INDEX: u8 = 44;
  pub const SET_INDEX: u8 = 45;
  pub const DOT:  u8 = 46;
  pub const DOT_SET: u8 = 47;
  pub const SWAP: u8 = 48;
  pub const DUP:  u8 = 49;
  pub const DUP_DOT_SWAP: u8 = 50;
  pub const AND:  u8 = 51;
  pub const OR:   u8 = 52;
  pub const NOT:  u8 = 53;
  pub const NEXT: u8 = 54;
  pub const RANGE:u8 = 55;
  pub const MOD:  u8 = 56;
  pub const ELSE: u8 = 57;
  pub const YIELD:u8 = 58;
  pub const EMPTY:u8 = 59;
  pub const TABLE:u8 = 60;
  pub const GET:  u8 = 61;
  pub const BAND: u8 = 62;
  pub const BOR:  u8 = 63;
  pub const AOP:  u8 = 64;
  pub const RAISE:u8 = 65;
  pub const AOP_INDEX:u8 = 66;
  pub const OP:  u8 = 67;
  pub const TRY:  u8 = 68;
  pub const TRYEND:u8 = 69;
  pub const GETEXC:u8 = 70;
  pub const CRAISE:u8 = 71;
  pub const HALT: u8 = 72;
  pub const LONG: u8 = 73;

  pub fn op_to_str(x: u8) -> &'static str {
    match x {
      NULL => "NULL",
      OF => "OF",
      FALSE => "FALSE",
      TRUE => "TRUE",
      INT => "INT",
      FLOAT => "FLOAT",
      IMAG => "IMAG",
      NEG => "NEG",
      ADD => "ADD",
      SUB => "SUB",
      MPY => "MPY",
      DIV => "DIV",
      IDIV => "IDIV",
      MOD => "MOD",
      POW => "POW",
      EQ => "EQ",
      NE => "NE",
      IS => "IS",
      ISNOT => "ISNOT",
      IN => "IN",
      NOTIN => "NOTIN",
      LT => "LT",
      GT => "GT",
      LE => "LE",
      GE => "GE",
      LIST => "LIST",
      MAP => "MAP",
      LOAD => "LOAD",
      STORE => "STORE",
      JMP => "JMP",
      JZ => "JZ",
      JNZ => "JNZ",
      CALL => "CALL",
      RET => "RET",
      STR => "STR",
      FN => "FN",
      FNSEP => "FNSEP",
      POP => "POP",
      LOAD_LOCAL => "LOAD_LOCAL",
      LOAD_ARG => "LOAD_ARG",
      LOAD_CONTEXT => "LOAD_CONTEXT",
      STORE_LOCAL => "STORE_LOCAL",
      STORE_ARG => "STORE_ARG",
      STORE_CONTEXT => "STORE_CONTEXT",
      FNSELF => "FNSELF",
      GET_INDEX => "GET_INDEX",
      SET_INDEX => "SET_INDEX",
      DOT => "DOT",
      DOT_SET => "DOT_SET",
      SWAP => "SWAP",
      DUP => "DUP",
      DUP_DOT_SWAP => "DUP_DOT_SWAP",
      AND => "AND",
      OR => "OR",
      NOT => "NOT",
      NEXT => "NEXT",
      ELSE => "ELSE",
      YIELD => "YIELD",
      EMPTY => "EMPTY",
      TABLE => "TABLE",
      GET => "GET",
      BAND => "BAND",
      BOR => "BOR",
      AOP => "AOP",
      RAISE => "RAISE",
      AOP_INDEX => "AOP_INDEX",
      OP => "OP",
      TRY => "TRY",
      TRYEND => "TRYEND",
      GETEXC => "GETEXC",
      CRAISE => "CRAISE",
      HALT => "HALT",
      _ => "unknown"
    }
  }
}

fn print_op(x: u32){
  println!("{}",bc::op_to_str(x as u8));
}

fn print_stack(env: &mut Env, a: &[Object]){
  let s = match list_to_string(env,a) {Ok(s)=>s, Err(_)=>panic!()};
  println!("stack: {}",s);
}

impl PartialEq for Object{
  fn eq(&self, b: &Object) -> bool{
    'r: loop{
    match *self {
      Object::Null => {
        return match *b {
          Object::Null => true,
          _ => false
        };
      },
      Object::Bool(x) => {
        return match *b {
          Object::Bool(y) => x==y,
          _ => false
        };
      },
      Object::Int(x) => {
        return match *b {
          Object::Int(y) => x==y,
          Object::Float(y) => (x as f64)==y,
          Object::Complex(y) => (x as f64)==y.re && y.im==0.0,
          _ => {break 'r;}
        };
      },
      Object::Float(x) => {
        return match *b {
          Object::Int(y) => x==(y as f64),
          Object::Float(y) => x==y,
          Object::Complex(y) => x==y.re && y.im==0.0,
          _ => {break 'r;}
        };
      },
      Object::Complex(x) => {
        return match *b {
          Object::Int(y) => x.re==(y as f64) && x.im==0.0,
          Object::Float(y) => x.re==y && x.im==0.0,
          Object::Complex(y) => x==y,
          _ => {break 'r;}
        };
      },
      Object::String(ref x) => {
        return match *b {
          Object::String(ref y) => x.v==y.v,
          _ => false
        };
      },
      Object::List(ref x) => {
        return match *b {
          Object::List(ref y) => {
            x.borrow().v == y.borrow().v
          },
          _ => false
        };
      },
      Object::Map(ref x) => {
        return match *b {
          Object::Map(ref y) => {
            x.borrow().m == y.borrow().m
          },
          _ => false
        };
      },
      Object::Function(ref f) => {
        return match *b {
          Object::Function(ref g) => {
            Rc::ptr_eq(f,g)
          },
          _ => false
        };
      },
      Object::Range(ref x) => {
        return match *b {
          Object::Range(ref y) => {
            x.a==y.a && x.b==y.b && x.step==y.step
          },
          _ => false
        };
      },
      Object::Empty => {
        return match *b {
          Object::Empty => true,
          _ => false
        };
      },
      _ => {}
    }
    return match *self {
      Object::Table(ref x) => {
        return match *b {
          Object::Table(ref y) => Rc::ptr_eq(x,y),
          _ => false
        }
      },
      Object::Interface(ref x) => {
        return x.eq_plain(b);
      },
      _ => false
    };
    } // 'r
    return match *b {
      Object::Interface(ref x) => {
        return x.req_plain(self);
      },
      _ => false
    }
  }
}
impl Eq for Object{}

impl Hash for Object{
  fn hash<H: Hasher>(&self, state: &mut H){
    match *self {
      Object::Null => {},
      Object::Bool(x) => {x.hash(state);},
      Object::Int(x) => {x.hash(state);},
      Object::String(ref x) => {
        let s = &x.v;
        s.hash(state);
      },
      Object::List(ref a) => {
        let mut a = a.borrow_mut();
        a.frozen = true;
        a.v.hash(state);
      },
      Object::Map(ref m) => {
        let mut m = m.borrow_mut();
        m.frozen = true;
        let mut hash: u64 = 0;
        for (key,value) in &m.m {
          let mut hstate = DefaultHasher::new();
          key.hash(&mut hstate);
          value.hash(&mut hstate);
          hash = hash.wrapping_add(hstate.finish());
        }
        state.write_u64(hash);
      },
      Object::Table(ref t) => {
        let p: &Table = t;
        (p as *const _).hash(state);
      },
      Object::Interface(ref x) => {
        state.write_u64(x.hash());
      },
      _ => panic!()
    }
  }
}

fn list_to_string(env: &mut Env, a: &[Object]) -> Result<String,Box<Exception>> {
  let mut s = String::from("[");
  for i in 0..a.len() {
    if i!=0 {s.push_str(", ");}
    s.push_str(&try!(object_to_repr(env,&a[i])));
  }
  s.push_str("]");
  return Ok(s);
}

/*
fn tuple_to_string(env: &mut Env, a: &[Object]) -> Result<String,Box<Exception>> {
  let mut s = String::from("(");
  for i in 0..a.len() {
    if i!=0 {s.push_str(", ");}
    s.push_str(&try!(object_to_repr(env,&a[i])));
  }
  s.push_str(")");
  return Ok(s);
}
*/

fn map_to_string(env: &mut Env, a: &HashMap<Object,Object>,
  left: &str, right: &str
) -> Result<String,Box<Exception>>
{
  let mut s = String::from(left);
  let mut first=true;
  for (key,value) in a {
    if first {first=false;} else{s.push_str(", ");}
    s.push_str(&try!(object_to_repr(env,&key)));
    match value {
      &Object::Null => {},
      _ => {
        s.push_str(": ");
        s.push_str(&try!(object_to_repr(env,&value)));
      }
    }
  }
  s.push_str(right);
  return Ok(s);
}

fn float_to_string(x: f64) -> String {
  if x==0.0 {
    "0".to_string()
  }else if x.abs()>1E14 {
    format!("{:e}",x)
  }else if x.abs()<0.0001 {
    format!("{:e}",x)
  }else{
    format!("{}",x)
  }
}

fn is_digit(c: char) -> bool {
  ('0' as u32)<=(c as u32) && (c as u32)<=('9' as u32)
}

fn float_to_string_explicit(x: f64) -> String {
  let mut s = if x==0.0 {
    "0".to_string()
  }else if x.abs()>1E14 {
    format!("{:e}",x)
  }else if x.abs()<0.0001 {
    format!("{:e}",x)
  }else{
    format!("{}",x)
  };
  if s.chars().all(|c| c=='-' || is_digit(c)) {
    s.push_str(".0");
  }
  return s;
}

fn complex_to_string(z: Complex64) -> String {
  if z.im<0.0 {
    format!("{}{}i",float_to_string(z.re),float_to_string(z.im))
  }else{
    format!("{}+{}i",float_to_string(z.re),float_to_string(z.im))
  }
}

fn table_to_string(env: &mut Env, t: &Rc<Table>) -> Result<String,Box<Exception>> {
  if let Object::Table(ref pt) = t.prototype {
    if let Some(f) = pt.get(&env.rte().key_string) {
      let s = try!(env.call(&f,&Object::Table(t.clone()),&[]));
      return s.string(env);
    }
  }
  let left = if t.prototype == Object::Null {
    "table{"
  }else{
    "table(...){"
  };
  match t.map.try_borrow_mut() {
    Ok(m) => map_to_string(env,&m.m,left,"}"),
    Err(_) => Ok(format!("{}{}",left,"...}"))
  }
}

fn list_to_string_plain(a: &[Object]) -> String {
  let mut s = "[".to_string();
  for i in 0..a.len() {
    if i != 0 {s.push_str(", ");}
    s.push_str(&object_to_repr_plain(&a[i]));
  }
  s.push_str("]");
  return s;
}

fn map_to_string_plain(a: &HashMap<Object,Object>,
  left: &str, right: &str
) -> String
{
  let mut s = left.to_string();
  let mut first=true;
  for (key,value) in a {
    if first {first=false;} else{s.push_str(", ");}
    s.push_str(&object_to_repr_plain(&key));
    match value {
      &Object::Null => {},
      _ => {
        s.push_str(": ");
        s.push_str(&object_to_repr_plain(&value));
      }
    }
  }
  s.push_str(right);
  return s;
}

pub fn object_to_string_plain(x: &Object) -> String {
  match *x {
    Object::Null => "null".to_string(),
    Object::Bool(x) => (if x {"true"} else {"false"}).to_string(),
    Object::Int(x) => x.to_string(),
    Object::Float(x) => float_to_string_explicit(x),
    Object::Complex(x) => complex_to_string(x),
    Object::String(ref a) => a.v.iter().collect(),
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => list_to_string_plain(&a.v),
        Err(_) => "[...]".to_string()
      }    
    },
    Object::Map(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => map_to_string_plain(&a.m,"{","}"),
        Err(_) => "{...}".to_string()
      }
    },
    Object::Function(_) => "function".to_string(),
    Object::Range(_) => "range".to_string(),
    // Object::Tuple(_) => "tuple".to_string(),
    Object::Table(ref t) => {
      match t.map.try_borrow_mut() {
        Ok(m) => map_to_string_plain(&m.m,"table{","}"),
        Err(_) => "table{...}".to_string()
      }
    },
    Object::Empty => "empty".to_string(),
    Object::Interface(_) => "interface object".to_string()
  }
}

pub fn object_to_repr_plain(x: &Object) -> String {
  if let Object::String(ref s) = *x {
    string_to_repr(s)
  }else{
    object_to_string_plain(x)
  }
}

pub fn object_to_string(env: &mut Env, x: &Object) -> Result<String,Box<Exception>> {
  Ok(match *x {
    Object::Null => "null".to_string(),
    Object::Bool(b) => (if b {"true"} else {"false"}).to_string(),
    Object::Int(i) => format!("{}",i),
    Object::Float(x) => float_to_string_explicit(x),
    Object::Complex(z) => complex_to_string(z),
    Object::String(ref a) => a.v.iter().collect(),
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => {return list_to_string(env,&a.v);},
        Err(_) => "[...]".to_string()
      }
    },
    Object::Map(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => {return map_to_string(env,&a.m,"{","}");},
        Err(_) => "{...}".to_string()
      }
    },
    Object::Function(ref f) => {
      match f.id {
        Object::Null => format!("function"),
        Object::Int(x) => {
          let line = (x as u32) & 0xffff;
          let col = (x as u32)>>16;
          if let EnumFunction::Std(ref f) = f.f {
            format!("function ({}:{}:{})",&f.module.id,line,col)
          }else{
            format!("function ({}:{})",line,col)
          }
        },
        _ => format!("function {}",try!(object_to_string(env,&f.id)))
      }
    },
    Object::Range(ref r) => {
      match r.step {
        Object::Null => {
          format!("{}..{}",try!(object_to_string(env,&r.a)),try!(object_to_string(env,&r.b)))
        },
        ref step => {
          format!("{}..{}: {}",
            try!(object_to_string(env,&r.a)),
            try!(object_to_string(env,&r.b)),
            try!(object_to_string(env,step))
          )
        }
      }
    },
    Object::Table(ref t) => {
      return table_to_string(env,&t);
    },
    Object::Empty => {
      "empty".to_string()
    },
    Object::Interface(ref t) => {
      return t.to_string(env);
    }
  })
}

fn string_to_repr(s: &U32String) -> String{
  let mut buffer = "\"".to_string();
  for c in &s.v {
    if *c=='\n' {
      buffer.push_str("\\n");
    }else if *c=='\t' {
      buffer.push_str("\\t");
    }else if *c=='\\' {
      buffer.push_str("\\b");
    }else if *c=='"' {
      buffer.push_str("\\d");
    }else{
      buffer.push(*c);
    }
  }
  buffer.push('"');
  return buffer;
}

pub fn object_to_repr(env: &mut Env, x: &Object) -> Result<String,Box<Exception>> {
  match *x {
    Object::String(ref a) => {
      Ok(string_to_repr(a))
    },
    _ => object_to_string(env,x)
  }
}

fn function_id_to_string(env: &mut Env, f: &Function) -> String {
  match f.id {
    Object::Null => format!("function"),
    Object::Int(x) => {
      let line = (x as u32) & 0xffff;
      let col = (x as u32)>>16;
      if let EnumFunction::Std(ref f) = f.f {
        format!("function ({}:{}:{})",&f.module.id,line,col)
      }else{
        format!("function ({}:{})",line,col)
      }
    },
    _ => format!("{}",match object_to_string(env,&f.id) {
      Ok(value)=>value,
      Err(e) => "[could not show: Exception in str(f.id)]".to_string()
    })
  }
}

pub fn op_neg(env: &mut Env, x: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  try!(::vm::operator_neg(env.env,env.sp+1,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_add(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_plus(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_sub(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_minus(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_mpy(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_mpy(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_div(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_div(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_lt(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_lt(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
}

pub fn op_eq(env: &mut Env, x: &Object, y: &Object) -> FnResult {
  /*
  env.stack[env.sp] = x.clone();
  env.stack[env.sp+1] = y.clone();
  try!(::vm::operator_eq(env.env,env.sp+2,env.stack));
  return Ok(replace(&mut env.stack[env.sp],Object::Null));
  */
  return Ok(Object::Bool(x==y));
}

fn operator_neg(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  match stack[sp-1] {
    Object::Int(x) => {
      stack[sp-1] = Object::Int(-x);
      return Ok(());
    },
    Object::Float(x) => {
      stack[sp-1] = Object::Float(-x);
      return Ok(());
    },
    _ => {}
  }
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Table(t) => {
      match t.get(&env.rte.key_neg) {
        Some(ref f) => {
          match (Env{env,sp,stack}).call(f,&Object::Table(t),&[]) {
            Ok(y) => {stack[sp-1] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          Err(env.type_error1_plain(sp,stack,"Type error in -x.","x",&Object::Table(t)))
        }
      }
    },
    Object::Interface(x) => {
      match x.neg(&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error1_plain(sp,stack,"Type error in -x.","x",&Object::Interface(x)))
          }else{
            stack[sp-1] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    x => Err(env.type_error1_plain(sp,stack,"Type error in -x.","x",&x))
  }
}

fn list_add(a: &List, b: &List) -> Object {
  let mut v: Vec<Object> = Vec::with_capacity(a.v.len()+b.v.len());
  for x in &a.v {
    v.push(x.clone());
  }
  for x in &b.v {
    v.push(x.clone());
  }
  return List::new_object(v);
}

fn string_add(a: &U32String, b: &U32String) -> Object {
  let mut v: Vec<char> = Vec::with_capacity(a.v.len()+b.v.len());
  for c in &a.v {
    v.push(*c);
  }
  for c in &b.v {
    v.push(*c);
  }
  return U32String::new_object(v);
}

fn operator_plus(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = match x.checked_add(y) {
            Some(z) => Object::Int(z),
            None => {Long::add_int_int(x,y)}
          };
          return Ok(());
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x as f64+y);
          return Ok(());
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x as f64+y);
          return Ok(());
        },
        _ => {break 'r;}
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x+(y as f64));
          return Ok(());
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x+y);
          return Ok(());
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x+y);
          return Ok(());
        },
        _ => {break 'r;}
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(x+y as f64);
          return Ok(());
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(x+y);
          return Ok(());
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x+y);
          return Ok(());
        },
        _ => {break 'r;}
      }
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::String(a) => {
      match stack[sp-1].clone() {
        Object::String(b) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = string_add(&a,&b);
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::List(a) => {
      match stack[sp-1].clone() {
        Object::List(b) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = list_add(&*a.borrow(),&*b.borrow());
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_plus) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = stack[sp-1].clone();
      match a.add(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            break 'r;
          }else{
            stack[sp-1] = Object::Null;
            stack[sp-2] = y;
            Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.radd(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x+y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }      
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rplus) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x+y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x+y.","x","y",&x,&a
      ))
    }
  }
}

fn operator_minus(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = match x.checked_sub(y) {
            Some(z) => Object::Int(z),
            None => panic!()
          };
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x as f64-y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x as f64-y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x-(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x-y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x-y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Complex(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(x-y as f64);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(x-y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x-y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Map(a) => {
      match stack[sp-1].clone() {
        Object::Map(b) => {
          let mut m: HashMap<Object,Object> = HashMap::new();
          let b = &b.borrow().m;
          for (key,value) in &a.borrow().m {
            if !b.contains_key(&key) {
              m.insert(key.clone(),value.clone());
            }
          }
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Map(Rc::new(RefCell::new(
            Map{m: m, frozen: false}
          )));
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_minus) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = stack[sp-1].clone();
      match a.sub(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            break 'r;
          }else{
            stack[sp-1] = Object::Null;
            stack[sp-2] = y;
            Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.rsub(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x-y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }      
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rminus) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x-y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x-y.","x","y",&x,&a
      ))
    }
  }
}

fn operator_mpy(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'list: loop{
  'string: loop{
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = match x.checked_mul(y) {
            Some(z) => Object::Int(z),
            None => Long::mpy_int_int(x,y)
          };
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float((x as f64)*y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x as f64*y);
          Ok(())
        },
        Object::String(_) => {
          break 'string;
        },
        Object::List(_) => {
          break 'list;
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x*(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x*y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x*y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Complex(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(y as f64*x);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(y*x);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x*y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::String(s) => {
      let n = match stack[sp-1] {
        Object::Int(i) => i,
        _ => {break 'r;}
      };
      stack[sp-2] = ::string::duplicate(&s.v,n);
      Ok(())
    },
    Object::List(a) => {
      let n = match stack[sp-1] {
        Object::Int(x) => if x<0 {0 as usize} else {x as usize},
        _ => {break 'r;}
      };
      stack[sp-2] = ::list::duplicate(&a,n);
      Ok(())
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_mpy) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = stack[sp-1].clone();
      match a.mpy(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            break 'r;
          }else{
            stack[sp-1] = Object::Null;
            stack[sp-2] = y;
            Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.rmpy(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x*y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }      
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rmpy) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x*y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x*y.","x","y",&x,&a
      ))
    }
  };

  } // 'string
  let n = match stack[sp-2] {
    Object::Int(i) => i,
    _ => unreachable!()
  };
  let s = match replace(&mut stack[sp-1],Object::Null) {
    Object::String(s) => s,
    _ => unreachable!()
  };
  stack[sp-2] = ::string::duplicate(&s.v,n);
  return Ok(());

  } // 'list
  let n = match stack[sp-2] {
    Object::Int(x) => if x<0 {0 as usize} else {x as usize},
    _ => unreachable!()
  };
  let a = match replace(&mut stack[sp-1],Object::Null) {
    Object::List(a) => a,
    _ => unreachable!()
  };
  stack[sp-2] = ::list::duplicate(&a,n);
  return Ok(());
}

fn operator_div(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float((x as f64 )/(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float((x as f64)/y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x as f64/y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x/(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x/y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x/y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Complex(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(x/y as f64);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(x/y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x/y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Table(a) => {
      match a.get(&env.rte.key_div) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = stack[sp-1].clone();
      match a.div(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            break 'r;
          }else{
            stack[sp-1] = Object::Null;
            stack[sp-2] = y;
            Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.rdiv(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x/y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }      
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rdiv) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x/y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x/y.","x","y",&x,&a
      ))
    }
  }
}

fn operator_idiv(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          if y==0 {
            return Err(env.value_error_plain("Value error in a//b: b==0."));
          }
          stack[sp-2] = Object::Int(x/y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match a.idiv(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          stack[sp-2] = y;
          Ok(())
        },
        Err(e) => Err(e)
      }    
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.ridiv(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x//y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_ridiv) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x//y.","x","y",&x,&Object::Table(a)
          ))
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x//y.","x","y",&x,&a
      ))
    }
  }
}

fn operator_mod(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x%y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Table(a) => {
      match a.get(&env.rte.key_mod) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match a.imod(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {stack[sp-2]=y; Ok(())},
        Err(e) => Err(e)
      }    
    },
    Object::String(s) => {
      let a = replace(&mut stack[sp-1],Object::Null);
      match u32string_format(&mut Env{env: env, sp: sp, stack: stack},&s,&a) {
        Ok(y) => {stack[sp-2]=y; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.rimod(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {
          if env.is_unimplemented(&y) {
            Err(env.type_error2_plain(sp,stack,
              "Type error in x%y.","x","y",&b,&Object::Interface(a)
            ))            
          }else{
            stack[sp-2] = y; Ok(())
          }
        },
        Err(e) => Err(e)
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rmod) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x%y.","x","y",&x,&Object::Table(a)
          ))
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x%y.","x","y",&x,&a
      ))
    }
  }
}

fn checked_pow(mut base: i32, mut exp: u32) -> Option<i32> {
  if exp == 0 {return Some(1);}
  let mut acc: i32 = 1;
  loop {
    if (exp & 1) == 1 {
      acc = match acc.checked_mul(base) {Some(x)=>x, None=>return None};
    }
    exp /= 2;
    if exp == 0 {break;}
    base = match base.checked_mul(base) {Some(x)=>x, None=>return None};
  }
  return Some(acc);
}

fn operator_pow(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          if y<0 {
            stack[sp-2] = Object::Float((x as f64).powf(y as f64));
          }else{
            stack[sp-2] = match checked_pow(x,y as u32) {
              Some(z) => Object::Int(z),
              None => Long::pow_int_uint(x,y as u32)
            };
          }
          Ok(())
        },
        Object::Float(y) => {
          let z = (x as f64).powf(y);
          if z.is_nan() {
            stack[sp-2] = Object::Complex(Complex64{re: x as f64, im: 0.0}.powf(y));
          }else{
            stack[sp-2] = Object::Float(z);
          }
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(y.expa(x as f64));
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x.powi(y));
          Ok(())
        },
        Object::Float(y) => {
          let z = x.powf(y);
          if z.is_nan() {
            stack[sp-2] = Object::Complex(Complex64{re: x, im: 0.0}.powf(y));          
          }else{
            stack[sp-2] = Object::Float(z);
          }
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(y.expa(x));
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Complex(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(x.powf(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(x.powf(y));
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x.pow(y));
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Function(f) => {
      let n = replace(&mut stack[sp-1],Object::Null);
      match ::function::iterate(&mut Env{env,sp,stack},&Object::Function(f),&n) {
        Ok(y) => {stack[sp-2] = y; Ok(())},
        Err(e) => Err(e)
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_pow) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match a.pow(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {stack[sp-2] = y; Ok(())},
        Err(e) => Err(e)
      }     
    },
    _ => {break 'r;}
  };
  } // 'r
  match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(a) => {
      let b = replace(&mut stack[sp-2],Object::Null);
      match a.rpow(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(y) => {stack[sp-2] = y; Ok(())},
        Err(e) => Err(e)
      }      
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rpow) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x^y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x^y.","x","y",&x,&a
      ))
    }
  }
}

fn operator_band(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2].clone() {
    Object::Map(ref a) => {
      return match stack[sp-1].clone() {
        Object::Map(ref b) => {
          let mut m: HashMap<Object,Object> = HashMap::new();
          let b = &b.borrow().m;
          for (key,value) in &a.borrow().m {
            if b.contains_key(key) {
              m.insert(key.clone(),value.clone());
            }
          }
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Map(Rc::new(RefCell::new(
            Map{m: m, frozen: false}
          )));
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  break 'r;
  } // 'r
  let x = stack[sp-2].clone();
  let y = stack[sp-1].clone();
  return Err(env.type_error2_plain(sp,stack,
    "Type error in x&y.","x","y",&x,&y
  ));
}

fn operator_bor(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2].clone() {
    Object::Map(ref a) => {
      return match stack[sp-1].clone() {
        Object::Map(ref b) => {
          let mut m: HashMap<Object,Object> = HashMap::new();
          for (key,value) in &a.borrow().m {
            m.insert(key.clone(),value.clone());
          }
          for (key,value) in &b.borrow().m {
            m.insert(key.clone(),value.clone());
          }
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Map(Rc::new(RefCell::new(
            Map{m: m, frozen: false}
          )));
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  break 'r;
  } // 'r
  let x = stack[sp-2].clone();
  let y = stack[sp-1].clone();
  return Err(env.type_error2_plain(sp,stack,
    "Type error in x|y.","x","y",&x,&y
  ));
}

fn operator_eq(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Null => {
      return match stack[sp-1] {
        Object::Null => {
          stack[sp-2] = Object::Bool(true);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Bool(x) => {
      return match stack[sp-1] {
        Object::Bool(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)==y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Bool((x as f64)==y.re && y.im==0.0);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x==(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Bool(x==y.re && y.im==0.0);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Empty => {
      return match stack[sp-1] {
        Object::Empty => {
          stack[sp-2] = Object::Bool(true);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::String(x) => {
      match stack[sp-1].clone() {
        Object::String(y) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Bool(x.v==y.v);
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::List(x) => {
      match stack[sp-1].clone() {
        Object::List(y) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Bool(x.borrow().v == y.borrow().v);
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Map(x) => {
      match stack[sp-1].clone() {
        Object::Map(y) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Bool(x.borrow().m == y.borrow().m);
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Function(ref f) => {
      match stack[sp-1].clone() {
        Object::Function(ref g) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Bool(Rc::ptr_eq(f,g));
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Range(x) => {
      match stack[sp-1].clone() {
        Object::Range(y) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = Object::Bool(x.a==y.a && x.b==y.b && x.step==y.step);
          Ok(())
        },
        _ => {break 'r;}
      }
    },
    Object::Interface(x) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match x.eq(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(x) => {
      let a = replace(&mut stack[sp-2],Object::Null);
      match x.req(&a,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    a => {
      stack[sp-2] = Object::Bool(false);
      return Ok(());
    }
  };
}

fn operator_lt(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)<y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x<y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Interface(x) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match x.lt(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_lt) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          let y = replace(&mut stack[sp-1],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[y]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {break 'r;}
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(x) => {
      let a = replace(&mut stack[sp-2],Object::Null);
      match x.rlt(&a,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    Object::Table(a) => {
      match a.get(&env.rte.key_rlt) {
        Some(ref f) => {
          let x = replace(&mut stack[sp-2],Object::Null);
          match (Env{env,sp,stack}).call(f,&x,&[Object::Table(a)]) {
            Ok(y) => {stack[sp-2] = y; Ok(())},
            Err(e) => Err(e)
          }
        },
        None => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in x<y.","x","y",&x,&Object::Table(a)
          ))   
        }
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x<y.","x","y",&x,&a
      ))
    }
  };
}

fn operator_gt(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)>y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x>y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Interface(x) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match x.gt(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(x) => {
      let a = replace(&mut stack[sp-2],Object::Null);
      match x.rgt(&a,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x>y.","x","y",&x,&a
      ))
    }
  };
}

fn operator_le(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<=y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)<=y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<=(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x<=y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Interface(x) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match x.le(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(x) => {
      let a = replace(&mut stack[sp-2],Object::Null);
      match x.rle(&a,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x<=y.","x","y",&x,&a
      ))
    }
  };
}

fn operator_ge(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  'r: loop{
  match stack[sp-2] {
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>=y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)>=y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>=(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x>=y);
          Ok(())
        },
        _ => {break 'r;}
      };
    },
    _ => {}
  }
  return match stack[sp-2].clone() {
    Object::Interface(x) => {
      let b = replace(&mut stack[sp-1],Object::Null);
      match x.le(&b,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {break 'r;}
  };
  } // 'r
  return match replace(&mut stack[sp-1],Object::Null) {
    Object::Interface(x) => {
      let a = replace(&mut stack[sp-2],Object::Null);
      match x.rge(&a,&mut Env{env: env, sp: sp, stack: stack}) {
        Ok(value) => {stack[sp-2] = value; Ok(())},
        Err(e) => Err(e)
      }
    },
    a => {
      let x = stack[sp-2].clone();
      Err(env.type_error2_plain(sp,stack,
        "Type error in x>=y.","x","y",&x,&a
      ))
    }
  };
}

fn operator_is(sp: usize, stack: &mut [Object]) -> OperatorResult {
  match stack[sp-2] {
    Object::Null => {
      return match stack[sp-1] {
        Object::Null => {
          stack[sp-2] = Object::Bool(true);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())        
        }
      };
    },
    Object::Bool(x) => {
      return match stack[sp-1] {
        Object::Bool(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())
        }
      };
    },
    Object::Int(x) => {
      return match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())
        }
      };
    },
    Object::Float(x) => {
      return match stack[sp-1] {
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())
        }
      };
    },
    _ => {}
  }
  match replace(&mut stack[sp-2],Object::Null) {
    Object::Table(ref a) => {
      match replace(&mut stack[sp-1],Object::Null) {
        Object::Table(ref b) => {
          stack[sp-2] = Object::Bool(Rc::ptr_eq(a,b));
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())          
        }
      }
    },
    _ => {
      stack[sp-1] = Object::Null;
      stack[sp-2] = Object::Bool(false);
      Ok(())
    }
  }
}

fn operator_of(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  let type_obj = replace(&mut stack[sp-1],Object::Null);
  let value: bool;
  'ret: loop{
  match stack[sp-2] {
    Object::Null => {
      value = match type_obj {
        Object::Null => true,
        _ => false
      };
      break 'ret;
    },
    Object::Bool(x) => {
      value = match type_obj {
        Object::Table(ref t) => Rc::ptr_eq(t,&env.rte.type_bool),
        _ => false
      };
      break 'ret;
    },
    Object::Int(x) => {
      value = match type_obj {
        Object::Table(ref t) => Rc::ptr_eq(t,&env.rte.type_int),
        _ => false
      };
      break 'ret;
    },
    Object::Float(x) => {
      value = match type_obj {
        Object::Table(ref t) => Rc::ptr_eq(t,&env.rte.type_float),
        _ => false
      };
      break 'ret;
    },
    Object::Complex(x) => {
      value = match type_obj {
        Object::Table(ref t) => Rc::ptr_eq(t,&env.rte.type_complex),
        _ => false
      };
      break 'ret;
    },
    _ => {}
  }
  match replace(&mut stack[sp-2],Object::Null) {
    Object::String(x) => {
      value = match type_obj {
        Object::Table(ref t) => {
          Rc::ptr_eq(t,&env.rte.type_string) ||
          Rc::ptr_eq(t,&env.rte.type_iterable)
        },
        _ => false
      };
    },
    Object::List(x) => {
      value = match type_obj {
        Object::Table(ref t) => {
          Rc::ptr_eq(t,&env.rte.type_list) ||
          Rc::ptr_eq(t,&env.rte.type_iterable)
        },
        _ => false
      };
    },
    Object::Function(x) => {
      value = match type_obj {
        Object::Table(ref t) => {
          Rc::ptr_eq(t,&env.rte.type_function) ||
          Rc::ptr_eq(t,&env.rte.type_iterable)
        },
        _ => false
      };
    },
    Object::Table(x) => {
      let t = match type_obj {
        Object::Table(t)=>t,
        _ => {
          value = false;
          break 'ret;
        }
      };
      let mut p = &x.prototype;
      loop{
        match *p {
          Object::Table(ref pt) => {
            if Rc::ptr_eq(pt,&t) {
              value = true;
              break 'ret;
            }else{
              p = &pt.prototype;
            }
          },
          _ => {
            value = false;
            break 'ret;
          }
        }
      }
    },
    Object::Interface(x) => {
      value = x.is_instance_of(&type_obj,&env.rte);
    },
    _ => {value = false;}
  }
  break 'ret;
  } // 'ret
  stack[sp-2] = Object::Bool(value);
  return Ok(());
}

fn operator_in(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  let key = replace(&mut stack[sp-2],Object::Null);
  match replace(&mut stack[sp-1],Object::Null) {
    Object::List(a) => {
      for x in &a.borrow().v {
        if key==*x {
          stack[sp-2] = Object::Bool(true);
          return Ok(())
        };
      }
      stack[sp-2] = Object::Bool(false);
      return Ok(());
    },
    Object::String(s) => {
      let c = match key {
        Object::String(cs) => {
          if cs.v.len()==1 {cs.v[0]}
          else{
            return Err(env.value_error_plain("Value error in 'c in s': size(c)!=1."));
          }
        },
        _ => {
          return Err(env.type_error1_plain(sp,stack,
            "Type error in 'c in s': s is a string, but c is not.", "c", &key
          ));
        }
      };
      for x in &s.v {
        if c==*x {
          stack[sp-2] = Object::Bool(true);
          return Ok(());
        }
      }
      stack[sp-2] = Object::Bool(false);
      return Ok(());
    },
    Object::Map(m) => {
      if m.borrow().m.contains_key(&key) {
        stack[sp-2] = Object::Bool(true);
      }else{
        stack[sp-2] = Object::Bool(false);
      }
      return Ok(());
    },
    a => Err(env.type_error1_plain(sp,stack,
      "Type error in 'x in a': expected a to be of type List, String or Map.", "a", &a
    ))
  }
}

fn operator_range(sp: usize, stack: &mut [Object]) -> OperatorResult {
  let r = Object::Range(Rc::new(Range{
    a: replace(&mut stack[sp-3],Object::Null),
    b: replace(&mut stack[sp-2],Object::Null),
    step: replace(&mut stack[sp-1],Object::Null)
  }));
  stack[sp-3] = r;
  Ok(())
}

fn operator_list(sp: usize, stack: &mut [Object], size: usize) -> usize{
  let mut sp = sp;
  let mut v: Vec<Object> = Vec::new();
  for i in 0..size {
    v.push(replace(&mut stack[sp-size+i],Object::Null));
  }
  sp-=size;
  stack[sp] = List::new_object(v);
  sp+=1;
  return sp;
}

fn operator_map(sp: usize, stack: &mut [Object], size: usize) -> usize{
  let mut sp = sp;
  let mut m: HashMap<Object,Object> = HashMap::new();
  let mut i=0;
  while i<size {
    m.insert(
      replace(&mut stack[sp-size+i],Object::Null),
      replace(&mut stack[sp-size+i+1],Object::Null)
    );
    i+=2;
  }
  sp-=size;
  stack[sp] = Map::new_object(m);
  sp+=1;
  return sp;
}

fn operator_index(env: &mut EnvPart, argc: usize,
  sp: usize, stack: &mut [Object]
) -> OperatorResult
{
  if argc != 1 {
    return match stack[sp-1-argc].clone() {
      Object::Interface(x) => {
        let (s1,s2) = stack.split_at_mut(sp);
        let mut env = Env{sp: 0, stack: s2, env: env};
        match x.index(&s1[sp-argc..sp],&mut env) {
          Ok(value) => {
            s1[sp-1-argc] = value;
            for x in &mut s1[sp-argc..sp] {
              *x = Object::Null;
            }
            Ok(())
          },
          Err(e) => Err(e)
        }
      },
      Object::Function(f) => {
        let (s1,s2) = stack.split_at_mut(sp);
        let mut env = Env{sp: 0, stack: s2, env: env};
        match ::list::map_fn(&mut env,&Object::Function(f),&s1[sp-argc..sp]) {
          Ok(value) => {
            s1[sp-1-argc] = value;
            for x in &mut s1[sp-argc..sp] {
              *x = Object::Null;
            }
            Ok(())
          },
          Err(e) => Err(e)
        }        
      },
      _ => {
        return Err(env.type_error_plain("Type error in a[...]: got more than one index."));
      }
    }
  }
  match stack[sp-2].clone() {
    Object::List(a) => {
      let a = a.borrow();
      if let Object::Int(i) = stack[sp-1] {
        let index = if i<0 {
          let iplus = i+(a.v.len() as i32);
          if iplus<0 {
            return Err(env.index_error_plain(&format!("Error in a[i]: i=={} is out of lower bound.",i)));          
          }else{
            iplus as usize
          }
        }else{
          i as usize
        };
        stack[sp-2] = match a.v.get(index) {
          Some(x) => x.clone(),
          None => {
            return Err(env.index_error_plain(&format!(
              "Error in a[i]: i=={} is out of upper bound, size(a)=={}.",
              i, a.v.len()
            )));
          }
        };
        return Ok(());
      }
      match replace(&mut stack[sp-1],Object::Null) {
        Object::Range(r) => {
          let n = a.v.len() as i32;
          let i = match r.a {
            Object::Int(x) => if x<0 {x+n} else {x},
            Object::Null => 0,
            _ => return Err(env.type_error1_plain(sp,stack,
              "Type error in a[i..j]: i is not an integer.",
              "i",&r.a))
          };
          let j = match r.b {
            Object::Int(x) => if x< -1 {x+n} else {x},
            Object::Null => n-1,
            _ => return Err(env.type_error1_plain(sp,stack,
              "Type error in a[i..j]: j is not an integer.",
              "j",&r.b))
          };
          let mut v: Vec<Object> = Vec::new();
          for k in i..j+1 {
            if 0<=k && k<n {
              v.push(a.v[k as usize].clone());
            }
          }
          stack[sp-2] = List::new_object(v);
          Ok(())
        },
        i => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in a[i]: i is not an integer.",
            "a","i",&x,&i))
        }
      }
    },
    Object::String(s) => {
      if let Object::Int(i) = stack[sp-1] {
        let index = if i<0 {
          let iplus = i+(s.v.len() as i32);
          if iplus<0 {
            return Err(env.index_error_plain(&format!("Error in s[i]: i=={} is out of lower bound.",i)));          
          }else{
            iplus as usize
          }
        }else{
          i as usize
        };
        stack[sp-2] = match s.v.get(index) {
          Some(c) => U32String::new_object_char(*c),
          None => {
            return Err(env.index_error_plain(&format!(
              "Error in s[i]: i=={} is out of upper bound, size(s)=={}.",
              i, s.v.len()
            )));
          }
        };
        return Ok(());
      }
      match replace(&mut stack[sp-1],Object::Null) {
        Object::Range(r) => {
          let n = s.v.len() as i32;
          let i = match r.a {
            Object::Int(x) => if x<0 {x+n} else {x},
            Object::Null => 0,
            _ => return Err(env.type_error1_plain(sp,stack,
              "Type error in s[i..j]: i is not an integer.",
              "i",&r.a))
          };
          let j = match r.b {
            Object::Int(x) => if x< -1 {x+n} else{x},
            Object::Null => n-1,
            _ => return Err(env.type_error1_plain(sp,stack,
              "Type error in s[i..j]: j is not an integer.",
              "j",&r.b))
          };
          let mut v: Vec<char> = Vec::new();
          for k in i..j+1 {
            if 0<=k && k<n {
              v.push(s.v[k as usize]);
            }
          }
          stack[sp-2] = U32String::new_object(v);
          Ok(())
        },
        i => {
          let x = stack[sp-2].clone();
          Err(env.type_error2_plain(sp,stack,
            "Type error in s[i]: i is not an integer.",
            "s","i",&x,&i))
        }
      }
    },
    Object::Map(m) => {
      match m.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = x.clone();
          Ok(())
        },
        None => {
          let key = try!(stack[sp-1].clone().repr(&mut Env{env,sp,stack}));
          Err(env.index_error_plain(&format!("Index error in m[{}]: not found.",key)))
        }
      }
    },
    Object::Interface(x) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      match x.index(&[key],&mut Env{env,sp,stack}) {
        Ok(value) => {
          stack[sp-2] = value;
          Ok(())
        },
        Err(e) => Err(e)
      }
    },
    Object::Function(f) => {
      let a = replace(&mut stack[sp-1],Object::Null);
      match ::list::map_fn(&mut Env{env,sp,stack},&Object::Function(f),&[a]) {
        Ok(value) => {
          stack[sp-2] = value;
          Ok(())
        },
        Err(e) => Err(e)
      }    
    },
    a => Err(env.type_error1_plain(sp,stack,
      "Type error in a[i]: a is not index-able.",
      "a",&a))
  }
}

fn index_assignment(env: &mut EnvPart, argc: usize,
  sp: usize, stack: &mut [Object]
) -> OperatorResult
{
  if argc != 1 {
    return match stack[sp-1-argc].clone() {
      Object::Interface(x) => {
        let (s1,s2) = stack.split_at_mut(sp);
        let mut env = Env{sp: 0, stack: s2, env: env};
        match x.set_index(&s1[sp-argc..sp],&s1[sp-argc-2],&mut env) {
          Ok(value) => {
            for x in &mut s1[sp-argc-2..sp] {
              *x = Object::Null;
            }
            Ok(())
          },
          Err(e) => Err(e)
        }
      },
      _ => {
        return Err(env.type_error_plain("Type error in a[...]=x: got more than one index."));
      }
    }
  }
  match stack[sp-2].clone() {
    Object::List(a) => {
      match stack[sp-1] {
        Object::Int(i) => {
          let mut a = a.borrow_mut();
          if a.frozen {
            return Err(env.value_error_plain("Value error in a[i]: a is immutable."));
          }
          let index = if i<0 {
            let iplus = i+(a.v.len() as i32);
            if iplus<0 {
              return Err(env.index_error_plain(&format!("Error in a[i]: i=={} is out of lower bound.",i)));          
            }else{
              iplus as usize
            }
          }else{
            i as usize
          };
          match a.v.get_mut(index) {
            Some(x) => {
              *x = replace(&mut stack[sp-3],Object::Null);
              stack[sp-2] = Object::Null;
            },
            None => {
              return Err(env.index_error_plain(&format!(
                "Error in a[i]: i=={} is out of upper bound.", i
              )));
            }
          }
          Ok(())          
        },
        _ => Err(env.type_error_plain("Type error in a[i]=value: i is not an integer."))
      }
    },
    Object::Map(m) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      let value = replace(&mut stack[sp-3],Object::Null);
      let mut m = m.borrow_mut();
      if m.frozen {
        return Err(env.value_error_plain("Value error in m[key]=value: m is frozen."));
      }
      match m.m.insert(key,value) {
        Some(_) => {},
        None => {}
      }
      Ok(())
    },
    Object::Interface(x) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      let value = replace(&mut stack[sp-3],Object::Null);
      return match x.set_index(&[key],&value,&mut Env{env,sp,stack}) {
        Ok(y) => {
          stack[sp-2] = Object::Null;
          Ok(())
        },
        Err(e) => Err(e)
      };
    },
    a => Err(env.type_error1_plain(sp,stack,
      "Type error in a[i]=value: a is not index-able.",
      "a",&a
    ))
  }
}

fn table_get(t: &Table, key: &Object) -> Option<Object> {
  let mut p = t;
  loop{
    if let Some(y) = p.map.borrow().m.get(key) {
      return Some(y.clone());
    }else{
      match p.prototype {
        Object::Table(ref pt) => {p = pt;},
        Object::List(ref a) => {
          for x in &a.borrow().v {
            if let Object::Table(ref pt) = *x {
              if let Some(y) = table_get(pt,key) {
                return Some(y.clone());
              }
            }
          }
          return None;
        },
        _ => return None
      }
    }
  }
}

fn operator_dot(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  match stack[sp-2].clone() {
    Object::Table(t) => {
      if let Some(x) = table_get(&t,&stack[sp-1]) {
        stack[sp-2] = x;
        stack[sp-1] = Object::Null;
        return Ok(());
      }
    },
    Object::List(a) => {
      match env.rte.type_list.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          return Ok(());
        },
        None => {
          match env.rte.type_iterable.map.borrow().m.get(&stack[sp-1]) {
            Some(x) => {
              stack[sp-2] = x.clone();
              stack[sp-1] = Object::Null;
              return Ok(());
            },
            None => {}
          }
        }
      }
    },
    Object::Map(a) => {
      match env.rte.type_map.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          return Ok(());
        },
        None => {
          match env.rte.type_iterable.map.borrow().m.get(&stack[sp-1]) {
            Some(x) => {
              stack[sp-2] = x.clone();
              stack[sp-1] = Object::Null;
              return Ok(());
            },
            None => {}
          }
        }
      }
    },
    Object::Function(a) => {
      match env.rte.type_function.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          return Ok(());
        },
        None => {
          match env.rte.type_iterable.map.borrow().m.get(&stack[sp-1]) {
            Some(x) => {
              stack[sp-2] = x.clone();
              stack[sp-1] = Object::Null;
              return Ok(());
            },
            None => {}
          }        
        }
      }
    },
    Object::String(a) => {
      match env.rte.type_string.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          return Ok(());
        },
        None => {
          match env.rte.type_iterable.map.borrow().m.get(&stack[sp-1]) {
            Some(x) => {
              stack[sp-2] = x.clone();
              stack[sp-1] = Object::Null;
              return Ok(());
            },
            None => {}
          }
        }
      }
    },
    Object::Range(_) => {
      match env.rte.type_iterable.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          return Ok(());
        },
        None => {}
      }      
    },
    Object::Interface(x) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      match x.get(&key,&mut Env{env, sp, stack}) {
        Ok(value) => {
          stack[sp-2] = value;
          return Ok(());
        },
        Err(e) => {
          return Err(e);
        }
      }
    },
    x => {
      let key = stack[sp-1].clone();
      return Err(env.type_error1_plain(sp,stack,
        &format!("Type error in t.{}: t is not a table.",key),
        "x",&x
      ))
    }
  }
  let key = try!(stack[sp-1].clone().string(&mut Env{env,stack,sp}));
  return Err(env.index_error_plain(&format!(
    "Index error in t.{0}: '{0}' not in property chain.", key
  )));
}

fn operator_dot_set(env: &mut EnvPart, sp: usize, stack: &mut [Object]) -> OperatorResult {
  match stack[sp-2].clone() {
    Object::Table(t) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      let value = replace(&mut stack[sp-3],Object::Null);
      let mut m = t.map.borrow_mut();
      if m.frozen {
        return Err(env.value_error_plain("Value error in a.x=value: a is frozen."));
      }
      match m.m.insert(key,value) {
        Some(_) => {},
        None => {}
      }
      Ok(())
    },
    a => Err(env.type_error1_plain(sp,stack,
      "Type error in a.x: a is not a table.",
      "a",&a
    ))
  }
}

fn operator_get(env: &mut EnvPart,
  sp: usize, stack: &mut [Object], index: u32
) -> OperatorResult
{
  'error: loop{
    stack[sp] = if let Object::List(ref a) = stack[sp-1] {
      a.borrow().v[index as usize].clone()
    }else{
      break 'error;
    };
    return Ok(());
  }
  let a = stack[sp-1].clone();
  return Err(env.type_error1_plain(sp,stack,
    "Type error in x,y = a: a is not a list.","a",&a
  ));
}

fn operate(op: u32, env: &mut EnvPart, sp: usize, stack: &mut [Object],
  p: &mut Object, x: Object
) -> OperatorResult {
  stack[sp] = p.clone();
  stack[sp+1] = x;
  match op as u8 {
    bc::ADD  => {try!(operator_plus (env,sp+2,stack));},
    bc::SUB  => {try!(operator_minus(env,sp+2,stack));},
    bc::MPY  => {try!(operator_mpy  (env,sp+2,stack));},
    bc::DIV  => {try!(operator_div  (env,sp+2,stack));},
    bc::IDIV => {try!(operator_idiv (env,sp+2,stack));},
    bc::BAND => {try!(operator_band (env,sp+2,stack));},
    bc::BOR  => {try!(operator_bor  (env,sp+2,stack));},
    _ => {panic!();}
  }
  *p = replace(&mut stack[sp],Object::Null);
  return Ok(());
}

fn compound_assignment(key_op: u32, op: u32,
  env: &mut EnvPart, sp: usize, stack: &mut [Object]
) -> OperatorResult
{
  match key_op as u8 {
    bc::AOP_INDEX => {
      match replace(&mut stack[sp-3],Object::Null) {
        Object::List(a) => {
          let i = match stack[sp-2] {
            Object::Int(x) => x,
            _ => {
              return Err(env.type_error_plain("Type error in a[i]: i is not an integer."));
            }
          };
          let mut a = a.borrow_mut();
          if a.frozen {
            return Err(env.value_error_plain("Value error in assignment to a[i]: a is frozen."));            
          }
          let index = if i<0 {
            let iplus = i+(a.v.len() as i32);
            if iplus<0 {
              return Err(env.index_error_plain(&format!(
                "Index error in assignment to a[i]: i=={} is out of lower bound.",i
              )));
            }else{
              iplus as usize
            }
          }else{
            i as usize
          };
          let p = match a.v.get_mut(index) {
            Some(value) => value,
            None => {
              return Err(env.index_error_plain(&format!(
                "Index error in assignment to a[i]: i=={} is out of upper bound.", i
              )));
            }
          };

          let x = replace(&mut stack[sp-1],Object::Null);
          return operate(op,env,sp,stack,p,x);
        },
        Object::Map(m) => {
          let key = replace(&mut stack[sp-2],Object::Null);
          let mut m = m.borrow_mut();
          if m.frozen {
            return Err(env.value_error_plain("Value error in assignment to m[key]: m is frozen."));            
          }
          let p = match m.m.get_mut(&key) {
            Some(value)=>value,
            None => {
              return Err(env.index_error_plain("Index error in m[key]: key is not in m."));
            }
          };
          let x = replace(&mut stack[sp-1],Object::Null);
          return operate(op,env,sp,stack,p,x);
        },
        _ => {
          return Err(env.type_error_plain("Type error in a[i]: a is not a list."));
        }
      }
    },
    bc::DOT => {
      match replace(&mut stack[sp-3],Object::Null) {
        Object::Table(t) => {
          let key = replace(&mut stack[sp-2],Object::Null);
          let mut m = t.map.borrow_mut();
          if m.frozen {
            return Err(env.value_error_plain("Value error in assignment to t.(key): t is frozen."));            
          }
          let p = match m.m.get_mut(&key) {
            Some(value)=>value,
            None => {
              return Err(env.index_error_plain("Index error in assignment to t.(key): key is not in t."));
            }
          };
          let x = replace(&mut stack[sp-1],Object::Null);
          return operate(op,env,sp,stack,p,x);
        },
        _ => {
          return Err(env.type_error_plain("Type error in assignment to t.(key): t is not a table."));
        }
      }    
    },
    _ => panic!()
  }
}

#[inline(never)]
fn global_variable_not_found(env: &mut Env, module: &Module,
  index: u32, gtab: &RefCell<Map>
) -> OperatorResult {
  let mut s =  String::new();
  s.push_str(&format!("Error: variable '{}' not found.",try!(object_to_string(env,&module.data[index as usize]))));
  // println!("gtab: {}",object_to_repr(&Object::Map(gtab.clone())));
  // panic!()
  return Err(env.env.index_error_plain(&s));
}

#[inline(never)]
fn non_boolean_condition(env: &mut Env, condition: &Object) -> OperatorResult {
  Err(env.env.type_error1_plain(env.sp,env.stack,
    "Type error: condition is not of type bool.",
    "condition", condition
  ))
}

#[inline(never)]
fn mut_fn_aliasing(env: &mut Env, f: &Function) -> Box<Exception> {
  let s = match f.id.string(env) {
    Ok(value)=>value,
    Err(e)=>{return e;}
  };
  env.env.std_exception_plain(&format!(
    "Memory error: function '{}' is already borrowed.",s
  ))
}

fn get_line_col(a: &[u32], ip: usize) -> (usize,usize) {
  // let line = (a[ip+2] as usize)<<8 | (a[ip+1] as usize);
  // let col = a[ip+3] as usize;
  let line = ((a[ip]>>8) & 0xffff) as usize;
  let col = (a[ip]>>24) as usize;
  return (line,col);
}

#[inline(always)]
fn load_i32(a: &[u32], ip: usize) -> i32{
  // unsafe{*a.get_unchecked(ip) as i32}
  a[ip] as i32
}

#[inline(always)]
fn load_u32(a: &[u32], ip: usize) -> u32{
  // unsafe{*a.get_unchecked(ip)}
  a[ip]
}

#[inline(always)]
fn load_u64(a: &[u32], ip: usize) -> u64{
  // unsafe{*((a.as_ptr().offset(ip as isize)) as *const u64)}
  (a[ip+1] as u64)<<32 | (a[ip] as u64)
}

fn new_table(prototype: Object, map: Object) -> Object {
  match map {
    Object::Map(map) => {
      Object::Table(Rc::new(Table{prototype, map, extra: None}))
    },
    _ => panic!()
  }
}

// Runtime environment: globally accessible information.
pub struct RTE{
  pub type_bool: Rc<Table>,
  pub type_int: Rc<Table>,
  pub type_float: Rc<Table>,
  pub type_complex: Rc<Table>,
  pub type_long: Rc<Table>,
  pub type_string: Rc<Table>,
  pub type_list: Rc<Table>,
  pub type_map: Rc<Table>,
  pub type_function: Rc<Table>,
  pub type_iterable: Rc<Table>,
  pub type_std_exception: Rc<Table>,
  pub type_type_error: Rc<Table>,
  pub type_value_error: Rc<Table>,
  pub type_index_error: Rc<Table>,
  pub unimplemented: Rc<Table>,
  pub argv: RefCell<Option<Rc<RefCell<List>>>>,
  pub gtab: Rc<RefCell<Map>>,
  pub pgtab: RefCell<Rc<RefCell<Map>>>,
  pub delay: RefCell<Vec<Rc<RefCell<Map>>>>,
  pub module_table: Rc<RefCell<Map>>,
  pub interface_types: RefCell<Vec<Rc<Table>>>,

  pub key_string: Object,
  pub key_neg: Object,
  pub key_plus: Object,
  pub key_rplus: Object,
  pub key_minus: Object,
  pub key_rminus: Object,
  pub key_mpy: Object,
  pub key_rmpy: Object,
  pub key_div: Object,
  pub key_rdiv: Object,
  pub key_idiv: Object,
  pub key_ridiv: Object,
  pub key_mod: Object,
  pub key_rmod: Object,
  pub key_pow: Object,
  pub key_rpow: Object,
  pub key_lt: Object,
  pub key_rlt: Object,
  pub key_le: Object,
  pub key_rle: Object
}

impl RTE{
  pub fn new() -> Rc<RTE>{
    Rc::new(RTE{
      type_bool: Table::new(Object::Null),
      type_int: Table::new(Object::Null),
      type_float: Table::new(Object::Null),
      type_complex: Table::new(Object::Null),
      type_string: Table::new(Object::Null),
      type_list: Table::new(Object::Null),
      type_map:  Table::new(Object::Null),
      type_function: Table::new(Object::Null),
      type_iterable: Table::new(Object::Null),
      type_long: Table::new(Object::Null),
      type_std_exception: Table::new(Object::Null),
      type_type_error: Table::new(Object::Null),
      type_value_error: Table::new(Object::Null),
      type_index_error: Table::new(Object::Null),
      unimplemented: Table::new(Object::Null),
      argv: RefCell::new(None),
      gtab: Map::new(),
      pgtab: RefCell::new(Map::new()),
      delay: RefCell::new(Vec::new()),
      module_table: Map::new(),
      interface_types: RefCell::new(Vec::new()),

      key_string: U32String::new_object_str("string"),
      key_neg:    U32String::new_object_str("neg"),
      key_plus:   U32String::new_object_str("plus"),
      key_rplus:  U32String::new_object_str("rplus"),
      key_minus:  U32String::new_object_str("minus"),
      key_rminus: U32String::new_object_str("rminus"),
      key_mpy:    U32String::new_object_str("mpy"),
      key_rmpy:   U32String::new_object_str("rmpy"),
      key_div:    U32String::new_object_str("div"),
      key_rdiv:   U32String::new_object_str("rdiv"),
      key_idiv:    U32String::new_object_str("div"),
      key_ridiv:   U32String::new_object_str("rdiv"),
      key_mod:    U32String::new_object_str("mod"),
      key_rmod:   U32String::new_object_str("rmod"),
      key_pow:    U32String::new_object_str("pow"),
      key_rpow:   U32String::new_object_str("rpow"),
      key_lt:     U32String::new_object_str("lt"),
      key_rlt:     U32String::new_object_str("rlt"),
      key_le:     U32String::new_object_str("le"),
      key_rle:     U32String::new_object_str("rle")
    })
  }
  pub fn clear_at_exit(&self, gtab: Rc<RefCell<Map>>){
    // Prevent a memory leak induced by a circular reference
    // of a function to its gtab (the gtab may contain this
    // function). The gtab may also contain itself.
    self.delay.borrow_mut().push(gtab);
  }
  pub fn read_access(&self, id: &str) -> bool {
    return true;
  }
}

pub struct Module{
  // pub program: Rc<Vec<u32>>,
  // pub program: Vec<u32>,

  pub program: Rc<[u32]>,
  // Rc<[T]> is available in Rust version 1.21 onwards.

  pub data: Vec<Object>,
  pub rte: Rc<RTE>,
  pub gtab: Rc<RefCell<Map>>,
  pub id: String
}

pub struct Frame{
  ip: usize,
  base_pointer: usize,
  f: Rc<Function>,
  module: Rc<Module>,
  gtab: Rc<RefCell<Map>>,
  argc: usize,
  argv_ptr: usize,
  var_count: usize,
  ret: bool,
  catch: bool
}

pub struct State{
  pub stack: Vec<Object>,
  pub sp: usize,
  pub env: EnvPart
}

fn vm_loop(
  state: &mut Env,
  mut ip: usize,
  mut argv_ptr: usize,
  mut bp: usize,
  mut module: Rc<Module>,
  mut gtab: Rc<RefCell<Map>>,
  mut fnself: Rc<Function>
) -> OperatorResult
{
  let mut stack: &mut [Object] = state.stack;
  // let mut a: &[u8] = unsafe{&*(&module.program as &[u8] as *const [u8])};
  let mut env = &mut state.env;
  let mut a = module.program.clone();
  let mut sp=state.sp;

  let mut exception: OperatorResult = Ok(());
  let mut ret = true;
  let mut catch = false;

  // print_stack(&stack[0..10]);

  'main: loop{ // loop
  loop{ // try
    // print_stack(&stack[0..10]);
    // print_op(a[ip]);
    // match unsafe{*a.get_unchecked(ip) as u8} {
    match a[ip] as u8 {
      bc::NULL => {
        stack[sp] = Object::Null;
        sp+=1;
        ip+=BCSIZE;
      },
      bc::TRUE => {
        ip+=BCSIZE;
        stack[sp] = Object::Bool(true);
        sp+=1;
      },
      bc::FALSE => {
        ip+=BCSIZE;
        stack[sp] = Object::Bool(false);
        sp+=1;
      },
      bc::INT => {
        stack[sp] = Object::Int(load_i32(&a,ip+BCSIZE));
        sp+=1;
        ip+=BCASIZE;
      },
      bc::FLOAT => {
        stack[sp] = Object::Float(f64::from_bits(
          load_u64(&a,ip+BCSIZE)
        ));
        sp+=1;
        ip+=BCAASIZE;
      },
      bc::IMAG => {
        stack[sp] = Object::Complex(Complex64{re: 0.0,
          im: f64::from_bits(load_u64(&a,ip+BCSIZE))
        });
        sp+=1;
        ip+=BCAASIZE;
      },
      bc::STR => {
        let index = load_u32(&a,ip+BCSIZE);
        stack[sp] = module.data[index as usize].clone();
        sp+=1;
        ip+=BCASIZE;
      },
      bc::LOAD_ARG => {
        let index = load_u32(&a,ip+BCSIZE) as usize;
        stack[sp] = stack[argv_ptr+index].clone();
        sp+=1;
        ip+=BCASIZE;
      },
      bc::LOAD_LOCAL => {
        let index = load_u32(&a,ip+BCSIZE) as usize;
        stack[sp] = stack[bp+index].clone();
        sp+=1;
        ip+=BCASIZE;
      },
      bc::STORE_LOCAL => {
        sp-=1;
        let index = load_u32(&a,ip+BCSIZE) as usize;
        stack[bp+index] = replace(&mut stack[sp],Object::Null);
        ip+=BCASIZE;
      },
      bc::FNSELF => {
        ip+=BCSIZE;
        stack[sp] = Object::Function(fnself.clone());
        sp+=1;
      },
      bc::NEG => {
        match operator_neg(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        ip+=BCSIZE;
      },
      bc::ADD => {
        match operator_plus(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::SUB => {
        match operator_minus(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::MPY => {
        match operator_mpy(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::DIV => {
        match operator_div(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::IDIV => {
        match operator_idiv(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::MOD => {
        match operator_mod(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::POW => {
        match operator_pow(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::BAND => {
        match operator_band(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::BOR => {
        match operator_bor(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::EQ => {
        match operator_eq(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::NE => {
        match operator_eq(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        if let Object::Bool(value) = stack[sp-2] {
          stack[sp-2] = Object::Bool(!value);
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::LT => {
        match operator_lt(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::GT => {
        match operator_gt(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::LE => {
        match operator_le(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::GE => {
        match operator_ge(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::IS => {
        match operator_is(sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;      
      },
      bc::IN => {
        match operator_in(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::OF => {
        match operator_of(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;      
      },
      bc::NOT => {
        let x = match stack[sp-1] {
          Object::Bool(x)=>x,
          _ => {
            exception = Err(env.type_error_plain("Type error in not a: a is not a boolean."));
            break;
          }
        };
        stack[sp-1] = Object::Bool(!x);
        ip+=BCSIZE;
      },
      bc::AND => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>x,
          _ => {
            exception = Err(env.type_error_plain("Type error in a and b: a is not a boolean."));
            break;
          }
        };
        if condition {
          sp-=1;
          ip+=BCASIZE;
        }else{
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }
      },
      bc::OR => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>x,
          _ => {
            exception = Err(env.type_error_plain("Type error in a or b: a is not a boolean."));
            break;
          }
        };
        if condition {
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }else{
          sp-=1;
          ip+=BCASIZE;
        }      
      },
      bc::LIST => {
        let size = load_u32(&a,ip+BCSIZE) as usize;
        sp = operator_list(sp,&mut stack,size);
        ip+=BCASIZE;
      },
      bc::MAP => {
        let size = load_u32(&a,ip+BCSIZE) as usize;
        sp = operator_map(sp,&mut stack,size);
        ip+=BCASIZE;
      },
      bc::RANGE => {
        match operator_range(sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=2;
        ip+=BCSIZE;          
      },
      bc::JMP => {
        ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
      },
      bc::JZ => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>{sp-=1; x},
          _ => {
            let value = stack[sp-1].clone();
            exception = non_boolean_condition(&mut Env{env,sp,stack},&value);
            break;
          }
        };
        if condition {
          ip+=BCASIZE;
        }else{
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }
      },
      bc::JNZ => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>{sp-=1; x},
          _ => {
            let value = stack[sp-1].clone();
            exception = non_boolean_condition(&mut Env{env,sp,stack},&value);
            break;
          }
        };
        if condition {
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }else{
          ip+=BCASIZE;
        }
      },
      bc::CALL => {
        ip+=BCASIZE;
        let mut argc = load_u32(&a,ip-1) as usize;
        let fobj = stack[sp-argc-2].clone();
        match fobj {
          Object::Function(ref f) => {
            match f.f {
              EnumFunction::Std(ref sf) => {
                if argc != f.argc as usize {
                  if f.argc_min as usize <= argc && f.argc_max == VARIADIC {
                    let n = argc-f.argc_min as usize;
                    let mut v: Vec<Object> = Vec::with_capacity(n);
                    for x in &mut stack[sp-n..sp] {
                      v.push(replace(x,Object::Null));
                    }
                    argc=argc-n+1;
                    sp = sp-n+1;
                    stack[sp-1] = List::new_object(v);
                  }else if f.argc_min as usize <= argc && argc <= f.argc_max as usize {
                    while argc != f.argc_max as usize {
                      stack[sp] = Object::Null;
                      sp+=1;
                      argc+=1;
                    }
                  }else{
                    let s = try!(fobj.string(&mut Env{env,stack,sp}));
                    exception = Err(env.argc_error_plain(argc, f.argc_min, f.argc_max, &s));
                    break;
                  }
                }
                env.frame_stack.push(Frame{
                  ip: ip, base_pointer: bp,
                  f: replace(&mut fnself,(*f).clone()),
                  module: replace(&mut module,sf.module.clone()),
                  gtab: replace(&mut gtab,sf.gtab.clone()),
                  argc: argc,
                  argv_ptr: argv_ptr,
                  var_count: sf.var_count as usize,
                  ret: ret, catch: catch
                });
                // a = unsafe{&*(&module.program as &[u8] as *const [u8])};
                a = module.program.clone();
                ip = sf.address.get();
                argv_ptr = sp-argc-1;
                ret = false;
                catch = false;

                bp = sp;
                for _ in 0..sf.var_count {
                  stack[sp] = Object::Null;
                  sp+=1;
                }

                continue;
              },
              EnumFunction::Plain(ref fp) => {
                let y = {
                  let (s1,s2) = stack.split_at_mut(sp);
                  let mut env = Env{sp: 0, stack: s2, env: env};
                  match fp(&mut env, &s1[sp-argc-1], &s1[sp-argc..sp]) {
                    Ok(y) => y, Err(e) => {exception = Err(e); break;}
                  }
                };
                sp-=argc+1;
                stack[sp-1]=y;
                continue;
              },
              EnumFunction::Mut(ref fp) => {
                let y = {
                  let (s1,s2) = stack.split_at_mut(sp);
                  let mut env = Env{sp: 0, stack: s2, env: env};
                  let pf = &mut *match fp.try_borrow_mut() {
                    Ok(f)=>f, Err(e) => {
                      exception = Err(mut_fn_aliasing(&mut env,f));
                      break;
                    }
                  };
                  match pf(&mut env, &s1[sp-argc-1], &s1[sp-argc..sp]) {
                    Ok(y) => y, Err(e) => {exception = Err(e); break;}
                  }
                };
                sp-=argc+1;
                stack[sp-1]=y;
                continue;
              }
            }
          },
          _ => {
            match object_call(env,&fobj,argc,sp,stack) {
              Ok(()) => {}, Err(e) => {exception = Err(e); break;}
            }
            sp-=argc+1;
            continue;
          }
        }
      },
      bc::RET => {
        if ret {
          state.sp = sp;
          return Ok(());
        }
        let frame = env.frame_stack.pop().unwrap();
        module = frame.module;
        ip = frame.ip;
        argv_ptr = frame.argv_ptr;
        bp = frame.base_pointer;
        // a = unsafe{&*(&module.program as &[u8] as *const [u8])};
        a = module.program.clone();
        gtab = frame.gtab;
        fnself = frame.f;
        ret = frame.ret;
        catch = frame.catch;

        let y = replace(&mut stack[sp-1],Object::Null);
        let n = frame.argc+2+frame.var_count;
        sp-=n;
        for x in stack[sp..sp+n].iter_mut() {
          *x = Object::Null;
        }

        stack[sp-1] = y;
      },
      bc::LOAD => {
        let index = load_u32(&a,ip+BCSIZE);
        let key = &module.data[index as usize];
        match gtab.borrow().m.get(key) {
          Some(x) => {
            stack[sp] = x.clone();
            sp+=1;
          },
          None => {
            match module.gtab.borrow().m.get(key) {
              Some(x) => {
                stack[sp] = x.clone();
                sp+=1;
              },
              None => {
                exception = global_variable_not_found(&mut Env{env,stack,sp},&module,index,&gtab);
                break;
              }
            }
          }
        }
        ip+=BCASIZE;
      },
      bc::STORE => {
        let index = load_u32(&a,ip+BCSIZE);
        let key = module.data[index as usize].clone();
        gtab.borrow_mut().m.insert(key,replace(&mut stack[sp-1],Object::Null));
        sp-=1;
        ip+=BCASIZE;
      },
      bc::STORE_ARG => {
        sp-=1;
        let index = load_u32(&a,ip+BCSIZE) as usize;
        stack[argv_ptr+index] = replace(&mut stack[sp],Object::Null);
        ip+=BCASIZE;
      },
      bc::STORE_CONTEXT => {
        let index = load_u32(&a,ip+BCSIZE) as usize;
        match fnself.f {
          EnumFunction::Std(ref sf) => {
            sf.context.borrow_mut().v[index] = replace(&mut stack[sp-1],Object::Null);
          },
          _ => panic!()
        }
        sp-=1;
        ip+=BCASIZE;
      },
      bc::LOAD_CONTEXT => {
        let index = load_u32(&a,ip+BCSIZE) as usize;
        match fnself.f {
          EnumFunction::Std(ref sf) => {
            stack[sp] = sf.context.borrow().v[index].clone();
          },
          _ => panic!()
        }
        sp+=1;
        ip+=BCASIZE;
      },
      bc::FN => {
        ip+=BCSIZE+4;
        let address = (ip as i32-5+load_i32(&a,ip-4)) as usize;
        // println!("fn [ip = {}]",address);
        let argc_min = load_u32(&a,ip-3);
        let argc_max = load_u32(&a,ip-2);
        let var_count = load_u32(&a,ip-1);
        let context = match replace(&mut stack[sp-2],Object::Null) {
          Object::List(a) => a,
          Object::Null => Rc::new(RefCell::new(List::new())),
          _ => panic!()
        };
        sp-=1;
        let id = replace(&mut stack[sp],Object::Null);
        stack[sp-1] = Function::new(StandardFn{
          address: Cell::new(address),
          module: module.clone(),
          gtab: gtab.clone(),
          var_count: var_count,
          context: context
        },id,argc_min,argc_max);
      },
      bc::GET_INDEX => {
        let argc = load_u32(&a,ip+BCSIZE) as usize;
        match operator_index(env, argc, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=argc;
        ip+=BCASIZE;
      },
      bc::SET_INDEX => {
        let argc = load_u32(&a,ip+BCSIZE) as usize;
        match index_assignment(env, argc, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=argc+2;
        ip+=BCASIZE;
      },
      bc::DOT => {
        match operator_dot(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=1;
        ip+=BCSIZE;
      },
      bc::DOT_SET => {
        match operator_dot_set(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=3;
        ip+=BCSIZE;
      },
      bc::DUP_DOT_SWAP => {
        let x = stack[sp-2].clone();
        match operator_dot(env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        stack[sp-1] = x;
        ip+=BCSIZE;
      },
      bc::POP => {
        sp-=1;
        stack[sp] = Object::Null;
        ip+=BCSIZE;
      },
      bc::NEXT => {
        let y = {
          let x = stack[sp-1].clone();
          let mut env = Env{sp: sp, stack: stack, env: env};
          match env.iter_next(&x) {
            Ok(y)=>y, Err(e)=>{exception=Err(e); break;}
          }
        };
        if y==Object::Empty {
          sp-=1;
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }else{
          stack[sp-1] = y;
          ip+=BCASIZE;
        }
      },
      bc::YIELD => {
        match fnself.f {
          EnumFunction::Std(ref sf) => {
            sf.address.set(ip+BCSIZE);
          },
          _ => panic!()
        }
        if ret {
          state.sp = sp;
          return Ok(());
        }
        let frame = env.frame_stack.pop().unwrap();
        module = frame.module;
        ip = frame.ip;
        argv_ptr = frame.argv_ptr;
        bp = frame.base_pointer;
        a = module.program.clone();
        gtab = frame.gtab;
        fnself = frame.f;
        ret = frame.ret;
        catch = frame.catch;

        let y = replace(&mut stack[sp-1],Object::Null);
        let n = frame.argc+2+frame.var_count;
        sp-=n;
        for x in stack[sp..sp+n].iter_mut() {
          *x = Object::Null;
        }

        stack[sp-1] = y;
      },
      bc::ELSE => {
        if stack[sp-1]==Object::Null {
          sp-=1;
          ip+=BCASIZE;
        }else{
          ip = (ip as i32+load_i32(&a,ip+BCSIZE)) as usize;
        }      
      },
      bc::EMPTY => {
        stack[sp] = Object::Empty;
        sp+=1;
        ip+=BCSIZE;        
      },
      bc::TABLE => {
        sp-=1;
        stack[sp-1] = new_table(
          replace(&mut stack[sp],Object::Null),
          replace(&mut stack[sp-1],Object::Null)
        );
        ip+=BCSIZE;
      },
      bc::GET => {
        let index = load_u32(&a,ip+BCSIZE);
        match operator_get(env,sp,stack,index) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        };
        sp+=1;
        ip+=BCASIZE;
      },
      bc::RAISE => {
        sp-=1;
        exception = Err(Exception::raise(
          replace(&mut stack[sp],Object::Null)
        ));
        break;
      },
      bc::AOP => {
        match compound_assignment(a[ip+1],a[ip+2],env, sp, &mut stack) {
          Ok(())=>{}, Err(e)=>{exception=Err(e); break;}
        }
        sp-=3;
        ip+=BCAASIZE;
      },
      bc::HALT => {
        state.sp=sp;
        return Ok(());
      },
      bc::OP => {
        ip+=BCSIZE;
        let op = a[ip] as u8;
        if op == bc::TRY {
          catch = true;
          env.catch_stack.push(CatchFrame{
            sp: sp, ip: (ip as i32+load_i32(&a,ip+BCSIZE)) as usize
          });
          ip+=BCASIZE;
        }else if op == bc::TRYEND {
          catch = false;
          env.catch_stack.pop();
          ip+=BCSIZE;
        }else if op == bc::GETEXC {
          if let Err(ref e) = exception {
            stack[sp] = e.value.clone();
            sp+=1;
          }else{
            panic!();
          }
          ip+=BCSIZE;
        }else if op == bc::CRAISE {
          catch = false;
          env.catch_stack.pop();
          break;
        }else{
          panic!();
        }
      },
      bc::LONG => {
        let index = load_u32(&a,ip+BCSIZE);
        stack[sp] = match Long::to_long(&module.data[index as usize]) {
          Ok(x) => x,
          Err(()) => panic!()
        };
        sp+=1;
        ip+=BCASIZE;
      },
      _ => {panic!()}
    }
  }

  // catch:
  if catch {
    let cframe = env.catch_stack.last().unwrap();
    ip = cframe.ip;
    for i in cframe.sp..sp {
      stack[i] = Object::Null;
    }
    sp = cframe.sp;
  }else{
    state.sp = sp;
    if let Err(ref mut e) = exception {
      let (line,col) = get_line_col(&a,ip);
      e.set_spot(line,col,&module.id);

      loop{
        let frame = match env.frame_stack.pop() {
          Some(x)=>x,
          None=>{break;}
        };
        module = frame.module;
        a = module.program.clone();
        let (line,col) = get_line_col(&a,frame.ip-BCASIZE);
        let fids = function_id_to_string(&mut Env{env,sp,stack},&*fnself);
        e.push_clm(line,col,&module.id,&fids);
        fnself = frame.f;
        if frame.catch {
          let cframe = env.catch_stack.last().unwrap();
          ip = cframe.ip;
          for i in cframe.sp..sp {
            stack[i] = Object::Null;
          }
          sp = cframe.sp;
          argv_ptr = frame.argv_ptr;
          bp = frame.base_pointer;
          gtab = frame.gtab;
          ret = frame.ret;
          catch = true;

          continue 'main;
        }
      }
    }else{
      panic!();
    }
    return exception;
  }

  }//goto loop
}

fn list_from_slice(a: &[Object]) -> Object {
  let n = a.len();
  let mut v: Vec<Object> = Vec::with_capacity(n);
  for i in 0..n {
    v.push(a[i].clone());
  }
  return List::new_object(v);
}

pub fn eval(env: &mut Env,
  module: Rc<Module>, gtab: Rc<RefCell<Map>>, command_line: bool)
  -> Result<Object,Box<Exception>>
{
  let fnself = Rc::new(Function{
    f: EnumFunction::Plain(::global::fpanic),
    argc: 0, argc_min: 0, argc_max: 0,
    id: Object::Null
  });
  {
    let mut pgtab = env.rte().pgtab.borrow_mut();
    *pgtab = gtab.clone();
  }

  let bp = env.sp;
  match vm_loop(env, 0, bp, bp, module, gtab.clone(), fnself) {
    Ok(())=>{},
    Err(e)=>{
      for i in bp..env.sp {
        env.stack[i] = Object::Null;
      }
      env.sp = bp;
      return Err(e);
    }
  }

  let ref mut stack = env.stack;
  let sp = env.sp;

  let y = if command_line {
    let env = &mut Env{env: env.env, sp, stack};
    for i in bp..sp {
      match env.stack[i].clone() {
        Object::Null => {},
        x => {println!("{}",try!(x.repr(env)));}
      }
    }
    Object::Null
  }else{
    if sp==bp {
      Object::Null
    }else if sp==bp+1 {
      stack[bp].clone()
    }else{
      list_from_slice(&stack[bp..sp])
    }
  };
  for i in bp..sp {
    stack[i] = Object::Null;
  }
  env.sp = bp;
  return Ok(y);
}

#[inline(never)]
fn object_call(env: &mut EnvPart, f: &Object,
  argc: usize, sp: usize, stack: &mut [Object]
) -> OperatorResult
{
  match *f {
    Object::Map(ref m) => {
      let argv = &mut stack[sp-argc-2..sp];
      if argv.len()!=3 {
        return Err(env.argc_error_plain(argv.len()-2,1,1,"sloppy index"));
      }
      if argv[1]!=Object::Null {
        argv[1] = Object::Null;
      }
      let key = replace(&mut argv[2],Object::Null);
      argv[0] = match m.borrow().m.get(&key) {
        Some(x) => x.clone(),
        None => Object::Null
      };
      Ok(())
    },
    _ => Err(env.type_error1_plain(sp,stack,
      "Type error in f(...): f is not callable.",
      "f", f))
  }
}

fn bounded_repr(env: &mut Env, x: &Object) -> Result<String,Box<Exception>> {
  let s = try!(x.repr(env));
  if s.len()>32 {
    return Ok(s[0..32].to_string()+"... ");
  }else{
    return Ok(s);
  }
}

fn exception_value_to_string(env: &mut Env, x: &Object) -> String {
  let value = if let Object::Table(ref t) = *x {
    let m = &t.map.borrow().m;
    let key = U32String::new_object_str("value");
    if let Some(value) = m.get(&key) {value.clone()} else{x.clone()}
  }else{x.clone()};
  return match value.string(env) {
    Ok(s) => {s}, Err(e) => {
      format!("{}\n[^Another exception occured in str(exception.value).]",
        exception_to_string(env,&e))
    }
  }
}

fn exception_to_string(env: &mut Env, e: &Exception) -> String {
  let mut s = String::new();
  if let Some(ref traceback) = e.traceback {
    for x in traceback.v.iter().rev() {
      match x.string(env) {
        Ok(x) => {writeln!(&mut s,"  in {}",x).unwrap();},
        Err(e) => {
          write!(&mut s,"{}",exception_to_string(env,&e)).unwrap();
          writeln!(&mut s,"[^Another exception occured in str(exception.traceback[k]).]").unwrap();
        }
      };
    }
  }
  if let Some(ref spot) = e.spot {
    writeln!(&mut s,"Line {}, col {} ({}):",spot.line,spot.col,&spot.module).unwrap();
  }
  write!(&mut s,"{}",exception_value_to_string(env,&e.value)).unwrap();
  return s;
}

pub fn get_env(state: &mut State) -> Env {
  return Env{
    sp: state.sp, stack: &mut state.stack,
    env: &mut state.env,
  };
}

pub fn stack_clear(a: &mut [Object]) {
  for x in a {
    *x = Object::Null;
  }
}

pub struct CatchFrame{
  ip: usize,
  sp: usize
}

pub struct EnvPart{
  frame_stack: Vec<Frame>,
  catch_stack: Vec<CatchFrame>,
  rte: Rc<RTE>,
}
impl EnvPart{
  pub fn new(frame_stack: Vec<Frame>, rte: Rc<RTE>) -> Self {
    Self{frame_stack, catch_stack: Vec::new(), rte}
  }
  pub fn is_unimplemented(&self, x: &Object) -> bool {
    if let Object::Table(ref t) = *x {
      return Rc::ptr_eq(t,&self.rte.unimplemented);
    }else{
      return false;
    }
  }

  pub fn std_exception_plain(&self, s: &str) -> Box<Exception> {
    Exception::new(s,Object::Table(self.rte.type_std_exception.clone()))
  }
  pub fn type_error_plain(&self, s: &str) -> Box<Exception> {
    Exception::new(s,Object::Table(self.rte.type_type_error.clone()))
  }
  pub fn value_error_plain(&self, s: &str) -> Box<Exception> {
    Exception::new(s,Object::Table(self.rte.type_value_error.clone()))
  }
  pub fn index_error_plain(&self, s: &str) -> Box<Exception> {
    Exception::new(s,Object::Table(self.rte.type_index_error.clone()))
  }

  pub fn argc_error_plain(&self, argc: usize, min: u32, max: u32, id: &str) -> Box<Exception> {
    let t = Object::Table(self.rte.type_std_exception.clone());
    if min==max {
      if min==0 {
        Exception::new(&format!("Error in {}: expected no argument, but got {}.",id,argc),t)
      }else if min==1 {
        Exception::new(&format!("Error in {}: expected 1 argument, but got {}.",id,argc),t)
      }else{
        Exception::new(&format!("Error in {}: expected {} arguments, but got {}.",id,min,argc),t)
      }
    }else{
      Exception::new(&format!("Error in {}: expected {}..{} arguments, but got {}.",id,min,max,argc),t)
    }
  }
  
  #[inline(never)]
  pub fn type_error1_plain(&mut self, sp: usize, stack: &mut [Object],
    s: &str, sx: &str, x: &Object
  ) -> Box<Exception>
  {
    let mut buffer = s.to_string();
    write!(buffer,"\nNote:\n").unwrap();
    {
      let env = &mut Env{env: self, sp, stack};
      let bs = match bounded_repr(env,x) {Ok(value)=>value, Err(e)=>return e};
      write!(buffer,"  {0}: {1}, {0} = {2}.",sx,&type_name(x),&bs).unwrap();
    }
    return self.type_error_plain(&buffer);
  }
  
  #[inline(never)]
  pub fn type_error2_plain(&mut self, sp: usize, stack: &mut [Object],
    s: &str, sx: &str, sy: &str, x: &Object, y: &Object
  ) -> Box<Exception>
  {
    let mut buffer = s.to_string();
    write!(buffer,"\nNote:\n").unwrap();
    {
      let env = &mut Env{env: self, sp, stack};
      let bsx = match bounded_repr(env,x) {Ok(value)=>value, Err(e)=>return e};
      let bsy = match bounded_repr(env,y) {Ok(value)=>value, Err(e)=>return e};
      write!(buffer,"  {0}: {1}, {0} = {2},\n",sx,&type_name(x),&bsx).unwrap();
      write!(buffer,"  {0}: {1}, {0} = {2}.",sy,&type_name(y),&bsy).unwrap();
    }
    return self.type_error_plain(&buffer);
  }
}

fn call_object(env: &mut Env,
  f: &Object, pself: &Object, argv: &[Object]
) -> FnResult
{
  match *f {
    Object::Map(ref m) => {
      match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"sloppy index")
      }
      return Ok(match m.borrow().m.get(&argv[0]) {
        Some(x) => x.clone(),
        None => Object::Null
      });
    },
    _ => env.type_error1(
      "Type error in f(...): f is not callable.",
      "f", f)
  }
}

// Calling environment of a function call
pub struct Env<'a>{
  pub sp: usize,
  stack: &'a mut [Object],
  env: &'a mut EnvPart
}

impl<'a> Env<'a>{
  pub fn call(&mut self, fobj: &Object,
    pself: &Object, argv: &[Object]
  ) -> FnResult
  {
    match *fobj {
      Object::Function(ref f) => {
        match f.f {
          EnumFunction::Std(ref fp) => {
            let sp = self.sp;
            self.stack[self.sp] = pself.clone();
            self.sp+=1;
            for x in argv {
              self.stack[self.sp] = x.clone();
              self.sp+=1;
            }
            let argc = argv.len();
            if argc != f.argc as usize {
              if f.argc_min as usize <= argc && f.argc_max == VARIADIC {
                let n = argc-f.argc_min as usize;
                let mut v: Vec<Object> = Vec::with_capacity(n);
                for x in &mut self.stack[self.sp-n..self.sp] {
                  v.push(replace(x,Object::Null));
                }
                self.sp = self.sp-n+1;
                self.stack[self.sp-1] = List::new_object(v);
              }else if f.argc_min as usize <= argc && argc <= f.argc_max as usize {
                for _ in argc..f.argc_max as usize {
                  self.stack[self.sp] = Object::Null;
                  self.sp+=1;
                }
              }else{
                stack_clear(&mut self.stack[sp..self.sp]);
                self.sp = sp;
                let s = try!(fobj.string(self));
                return self.argc_error(argc, f.argc_min, f.argc_max, &s);
              }
            }
            let bp = self.sp;
            for _ in 0..fp.var_count {
              self.stack[self.sp] = Object::Null;
              self.sp+=1;
            }
            try!(vm_loop(self,fp.address.get(),sp,bp,fp.module.clone(),fp.gtab.clone(),f.clone()));
            let y = replace(&mut self.stack[self.sp-1],Object::Null);
            for x in &mut self.stack[sp..self.sp-1] {
              *x = Object::Null;
            }
            self.sp = sp;
            return Ok(y);
          },
          EnumFunction::Plain(fp) => {
            return fp(self,pself,argv);
          },
          EnumFunction::Mut(ref fp) => {
            let pf = &mut *match fp.try_borrow_mut() {
              Ok(f)=>f, Err(e)=> return Err(mut_fn_aliasing(self,f))
            };
            return pf(self,pself,argv);
          }
        }
      },
      _ => {
        return call_object(self,fobj,pself,argv);
      }
    }
  }
  fn iter_next(&mut self, f: &Object) -> FnResult {
    self.call(f,&Object::Null,&[])
  }
  pub fn rte(&self) -> &Rc<RTE> {
    &self.env.rte
  }

  pub fn eval_string(&mut self, s: &str, id: &str, gtab: Rc<RefCell<Map>>, value: compiler::Value)
    -> Result<Object,Box<Exception>>
  {
    let mut history = system::History::new();
    match compiler::scan(s,1,id,false) {
      Ok(v) => {
        match compiler::compile(v,false,value,&mut history,id,self.rte()) {
          Ok(module) => {
            return eval(self,module.clone(),gtab.clone(),false);
          },
          Err(e) => {compiler::print_error(&e);}
        };
      },
      Err(error) => {
        compiler::print_error(&error);
      }
    }
    return Ok(Object::Null);
  }

  pub fn command_line_session(&mut self, gtab: Rc<RefCell<Map>>){
    let mut history = system::History::new();
    loop{
      let mut input = String::new();
      match system::getline_history("> ",&history) {
        Ok(s) => {
          if s=="" {continue;}
          history.append(&s);
          input=s;
        },
        Err(error) => {println!("Error: {}", error);},
      };
      if input=="quit" {break}
      match compiler::scan(&input,1,"command line",false) {
        Ok(v) => {
          // compiler::print_vtoken(&v);
          match compiler::compile(
            v,true,compiler::Value::Optional,&mut history,"command line",self.rte()
          ){
            Ok(module) => {
              match eval(self,module.clone(),gtab.clone(),true) {
                Ok(x) => {}, Err(e) => {
                  println!("{}",exception_to_string(self,&e));
                }
              }
            },
            Err(e) => {compiler::print_error(&e);}
          };
        },
        Err(error) => {
          compiler::print_error(&error);
        }
      }
    }
  }

  pub fn eval(&mut self, s: &str) -> Object {
    let gtab = Map::new();
    return match self.eval_string(s,"",gtab,compiler::Value::Optional) {
      Ok(x) => x,
      Err(e) => {
        println!("{}",self.exception_to_string(&e));
        panic!();
      }
    };
  }

  pub fn eval_env(&mut self, s: &str, gtab: Rc<RefCell<Map>>) -> Object {
    return match self.eval_string(s,"",gtab,compiler::Value::Optional) {
      Ok(x) => x,
      Err(e) => e.value
    };    
  }

  pub fn eval_file(&mut self, id: &str, gtab: Rc<RefCell<Map>>){
    let mut path: String = String::from(id);
    path += ".moss";
    let mut f = match File::open(&path) {
      Ok(f) => f,
      Err(e) => {
        match File::open(id) {
          Ok(f) => f,
          Err(e) => {
            println!("File '{}' not found.",id);
            return;
          }
        }
      }
    };
    let mut s = String::new();
    f.read_to_string(&mut s).expect("something went wrong reading the file");

    match self.eval_string(&s,id,gtab,compiler::Value::Optional) {
      Ok(x) => {}, Err(e) => {
        println!("{}",exception_to_string(self,&e));
      }
    }
  }

  #[inline(never)]
  pub fn std_exception(&self, s: &str) -> FnResult {
    Err(self.env.std_exception_plain(s))
  }

  #[inline(never)]
  pub fn type_error(&self, s: &str) -> FnResult {
    Err(self.env.type_error_plain(s))
  }

  #[inline(never)]
  pub fn value_error(&self, s: &str) -> FnResult {
    Err(self.env.value_error_plain(s))
  }

  #[inline(never)]
  pub fn index_error(&self, s: &str) -> FnResult {
    Err(self.env.index_error_plain(s))
  }

  #[inline(never)]
  pub fn argc_error(&self,
    argc: usize, min: u32, max: u32, id: &str
  ) -> FnResult {
    Err(self.env.argc_error_plain(argc,min,max,id))
  }

  #[inline(never)]
  pub fn type_error1(&mut self, s: &str, sx: &str, x: &Object) -> FnResult {
    return Err(self.env.type_error1_plain(self.sp,self.stack,s,sx,x));
  }

  #[inline(never)]
  pub fn type_error2(&mut self,
    s: &str, sx: &str, sy: &str, x: &Object, y: &Object
  ) -> FnResult
  {
    return Err(self.env.type_error2_plain(self.sp,self.stack,s,sx,sy,x,y));
  }
  
  pub fn exception_to_string(&mut self, e: &Exception) -> String {
    exception_to_string(self,e)
  }
  
  pub fn print_type_and_value(&mut self, x: &Object) {
    let svalue = match x.string(self) {
      Ok(s) => s, Err(e) => {
        panic!(self.exception_to_string(&e));
      }
    };
    let stype = type_name(x);
    println!("Type: {}, value: {}",stype,svalue);
  }
}

pub fn sys_call(env: &mut Env, pself: &Object, argv: &[Object]) -> FnResult {
  if argv.len()<2 {
    return env.argc_error(argv.len(),2,VARIADIC,"sys.call");
  }
  let n = match argv[0] {
    Object::Int(x)=>{
      if x<0 {panic!();}
      x as usize
    },
    _ => return env.type_error1(
      "Type error in sys.call(n,f): n is not an integer.",
      "n",&argv[0])
  };
  let mut v: Vec<Object> = vec![Object::Null; n];
  let mut calling_env = Env{sp: 0, stack: &mut v, env: env.env};
  calling_env.call(&argv[1],&Object::Null,&argv[2..])
}

