
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;
use std::mem::transmute;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use object::{
  Object, Map, List, Function, EnumFunction, StandardFn,
  FnResult, Exception, type_error, argc_error, index_error
};
use complex::Complex64;

pub const BCSIZE: usize = 4;
pub const BCSIZEP4: usize = BCSIZE+4;
pub const BCSIZEP8: usize = BCSIZE+8;

pub mod bc{
  pub const NULL: u8 = 00;
  pub const HALT: u8 = 01;
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
  pub const DOT: u8 = 46;
  pub const DOT_SET: u8 = 47;
  pub const SWAP: u8 = 48;
  pub const DUP: u8 = 49;

  pub fn op_to_str(x: u8) -> &'static str {
    match x {
      NULL => "NULL",
      HALT => "HALT",
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
      _ => "unknown"
    }
  }
}

fn print_op(x: u8){
  println!("{}",bc::op_to_str(x));
}

fn print_stack(a: &[Object]){
  let s = list_to_string(a);
  println!("stack: {}",s);
}

impl PartialEq for Object{
  fn eq(&self, b: &Object) -> bool{
    match self {
      &Object::Null => {
        match b {
          &Object::Null => true,
          _ => false
        }
      },
      &Object::Bool(x) => {
        match b {
          &Object::Bool(y) => x==y,
          _ => false
        }
      },
      &Object::Int(x) => {
        match b {
          &Object::Int(y) => x==y,
          &Object::Float(y) => (x as f64)==y,
          &Object::Complex(y) => (x as f64)==y.re && y.im==0.0,
          _ => false
        }
      },
      &Object::Float(x) => {
        match b {
          &Object::Int(y) => x==(y as f64),
          &Object::Float(y) => x==y,
          &Object::Complex(y) => x==y.re && y.im==0.0,
          _ => false
        }
      },
      &Object::Complex(x) => {
        match b {
          &Object::Int(y) => x.re==(y as f64) && x.im==0.0,
          &Object::Float(y) => x.re==y && x.im==0.0,
          &Object::Complex(y) => x==y,
          _ => false
        }
      },
      &Object::String(ref x) => {
        match b {
          &Object::String(ref y) => x.v==y.v,
          _ => false
        }
      },
      &Object::List(ref x) => {
        match b {
          &Object::List(ref y) => {
            x.borrow().v == y.borrow().v
          },
          _ => false
        }
      },
      &Object::Map(ref x) => {
        match b {
          &Object::Map(ref y) => {
            x.borrow().m == y.borrow().m
          },
          _ => false
        }
      },
      &Object::Function(ref f) => {
        match b {
          &Object::Function(ref g) => {
            Rc::ptr_eq(f,g)
          },
          _ => false
        }
      },
      &Object::Range(ref x) => {
        match b {
          &Object::Range(ref y) => {
            x.a==y.a && x.b==y.b && x.step==y.step
          },
          _ => false
        }
      },
      _ => panic!()
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
      _ => panic!()
    }
  }
}

fn list_to_string(a: &[Object]) -> String{
  let mut s = String::from("[");
  for i in 0..a.len() {
    if i!=0 {s.push_str(", ");}
    s.push_str(&object_to_repr(&a[i]));
  }
  s.push_str("]");
  return s;
}

fn tuple_to_string(a: &[Object]) -> String{
  let mut s = String::from("(");
  for i in 0..a.len() {
    if i!=0 {s.push_str(", ");}
    s.push_str(&object_to_repr(&a[i]));
  }
  s.push_str(")");
  return s;
}

fn map_to_string(a: &HashMap<Object,Object>, left: &str, right: &str) -> String{
  let mut s = String::from(left);
  let mut first=true;
  for (key,value) in a {
    if first {first=false;} else{s.push_str(", ");}
    s.push_str(&object_to_repr(&key));
    match value {
      &Object::Null => {},
      _ => {
        s.push_str(": ");
        s.push_str(&object_to_repr(&value));
      }
    }
  }
  s.push_str(right);
  return s;
}

pub fn object_to_string(x: &Object) -> String{
  match *x {
    Object::Null => String::from("null"),
    Object::Bool(b) => String::from(if b {"true"} else {"false"}),
    Object::Int(i) => format!("{}",i),
    Object::Float(x) => format!("{}",x),
    Object::Complex(z) => format!("{}+{}i",z.re,z.im),
    Object::List(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => list_to_string(&a.v),
        Err(_) => String::from("[...]")        
      }
    },
    Object::Map(ref a) => {
      match a.try_borrow_mut() {
        Ok(a) => map_to_string(&a.m,"{","}"),
        Err(_) => String::from("{...}")        
      }
    },
    Object::String(ref a) => {
      let s: String = a.v.iter().cloned().collect();
      format!("{}",s)
    },
    Object::Function(ref a) => {
      format!("function")
    },
    Object::Range(ref r) => {
      format!("{}..{}",object_to_string(&r.a),object_to_string(&r.b))
    },
    Object::Tuple(ref t) => {
      tuple_to_string(&t)
    },
    Object::Table(ref t) => {
      match t.map.try_borrow_mut() {
        Ok(m) => map_to_string(&m.m,"table ", " end"),
        Err(_) => String::from("table ... end")
      }
    }
  }
}

pub fn object_to_repr(x: &Object) -> String{
  match *x {
    Object::String(ref a) => {
      let s: String = a.v.iter().cloned().collect();
      format!("\"{}\"",s)
    },
    _ => object_to_string(x)
  }
}

fn operator_neg(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-1] {
    Object::Int(x) => {
      stack[sp-1] = Object::Int(-x);
      Ok(())
    },
    Object::Float(x) => {
      stack[sp-1] = Object::Float(-x);
      Ok(())
    },
    _ => type_error("Type error in -a.")
  }
}

fn operator_plus(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = match x.checked_add(y) {
            Some(z) => Object::Int(z),
            None => panic!()
          };
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x as f64+y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x as f64+y);
          Ok(())
        },
        _ => type_error("Type error in a+b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x+(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x+y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x+y);
          Ok(())
        },
        _ => type_error("Type error in a+b.")
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Complex(x+y as f64);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Complex(x+y);
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(x+y);
          Ok(())
        },
        _ => type_error("Type error in a+b.")
      }
    },
    _ => type_error("Type error in a+b.")
  }
}

fn operator_minus(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a-b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a-b.")
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a-b.")
      }
    },
    _ => type_error("Type error in a-b.")
  }
}

fn operator_mpy(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = match x.checked_mul(y) {
            Some(z) => Object::Int(z),
            None => panic!()
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
        _ => type_error(&format!("Type error in a*b: a={}, b={}.",
          stack[sp-2].str(), stack[sp-1].str()
        ))
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
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
        _ => type_error(&format!("Type error in a*b: a={}, b={}.",
          stack[sp-2].str(), stack[sp-1].str()
        ))
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
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
        _ => type_error(&format!("Type error in a*b: a={}, b={}.",
          stack[sp-2].str(), stack[sp-1].str()
        ))
      }
    },
    _ => type_error(&format!("Type error in a*b: a={}, b={}.",
      stack[sp-2].str(), stack[sp-1].str()
    ))
  }
}

fn operator_div(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a/b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a/b.")
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a/b.")
      }
    },
    _ => type_error("Type error in a/b.")
  }
}

fn operator_idiv(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x/y);
          Ok(())
        },
        _ => type_error("Type error in a//b.")
      }
    },
    _ => type_error("Type error in a//b.")
  }
}

fn operator_pow(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x.pow(y as u32));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float((x as f64).powf(y));
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(y.expa(x as f64));
          Ok(())
        },
        _ => type_error("Type error in a^b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float(x.powi(y));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Float(x.powf(y));
          Ok(())
        },
        Object::Complex(y) => {
          stack[sp-2] = Object::Complex(y.expa(x));
          Ok(())
        },
        _ => type_error("Type error in a^b.")
      }
    },
    Object::Complex(x) => {
      match stack[sp-1] {
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
        _ => type_error("Type error in a^b.")
      }
    },
    _ => type_error("Type error in a^b.")
  }
}

fn operator_lt(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)<y);
          Ok(())
        },
        _ => type_error("Type error in a<b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x<y);
          Ok(())
        },
        _ => type_error("Type error in a<b.")
      }
    },
    _ => type_error("Type error in a<b.")
  }
}

fn operator_gt(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)>y);
          Ok(())
        },
        _ => type_error("Type error in a>b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x>y);
          Ok(())
        },
        _ => type_error("Type error in a>b.")
      }
    },
    _ => type_error("Type error in a>b.")
  }
}

fn operator_le(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<=y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)<=y);
          Ok(())
        },
        _ => type_error("Type error in a<=b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<=(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x<=y);
          Ok(())
        },
        _ => type_error("Type error in a<=b.")
      }
    },
    _ => type_error("Type error in a<=b.")
  }
}

fn operator_ge(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x<y);
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool((x as f64)>=y);
          Ok(())
        },
        _ => type_error("Type error in a>=b.")
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x>=(y as f64));
          Ok(())
        },
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x>=y);
          Ok(())
        },
        _ => type_error("Type error in a>=b.")
      }
    },
    _ => type_error("Type error in a>=b.")
  }
}

fn operator_is(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2] {
    Object::Bool(x) => {
      match stack[sp-1] {
        Object::Bool(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())
        }
      }
    },
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())
        }
      }
    },
    Object::Float(x) => {
      match stack[sp-1] {
        Object::Float(y) => {
          stack[sp-2] = Object::Bool(x==y);
          Ok(())
        },
        _ => {
          stack[sp-2] = Object::Bool(false);
          Ok(())       
        }
      }
    },
    _ => type_error("Type error in a is b.")
  }
}

fn operator_list(sp: usize, stack: &mut Vec<Object>, size: usize) -> usize{
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

fn operator_map(sp: usize, stack: &mut Vec<Object>, size: usize) -> usize{
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

fn operator_index(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2].clone() {
    Object::List(a) => {
      match stack[sp-1] {
        Object::Int(i) => {
          if i>=0 {
            stack[sp-2] = match a.borrow().v.get(i as usize) {
              Some(x) => x.clone(),
              None => {
                return index_error(&format!(
                  "Error in a[i]: i=={} is out of upper bound, size(a)=={}.",
                  i, a.borrow().v.len()
                ));
              }
            };
          }else{
            return index_error(&format!("Error in a[i]: i=={} is out of lower bound.",i));
          }
          Ok(())
        },
        _ => type_error("Type error in a[i]: i is not an integer.")
      }
    },
    Object::Map(m) => {
      match m.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-1] = Object::Null;
          stack[sp-2] = x.clone();
          Ok(())
        },
        None => index_error("Index error in m[key]: key not found.")
      }
    },
    _ => type_error("Type error in a[i]: a is not index-able.")
  }
}

fn index_assignment(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2].clone() {
    Object::List(a) => {
      match stack[sp-1] {
        Object::Int(i) => {
          if i>=0 {
            let mut a = a.borrow_mut();
            if a.frozen {
              return index_error("Error in a[i]: a is immutable.");
            }
            match a.v.get_mut(i as usize) {
              Some(x) => {
                *x = replace(&mut stack[sp-3],Object::Null);
                stack[sp-2] = Object::Null;
              },
              None => {
                return index_error("Error in a[i]: i is out of bounds.");
              }
            }
          }else{
            return index_error("Error in a[i]: i is out of bounds.");
          }
          Ok(())          
        },
        _ => type_error("Type error in a[i]=value: i is not an integer.")
      }    
    },
    Object::Map(m) => {
      let key = replace(&mut stack[sp-1],Object::Null);
      let value = replace(&mut stack[sp-3],Object::Null);
      let mut m = m.borrow_mut();
      if m.frozen {
        return type_error("Index error in m[key]=value: m is frozen.");
      }
      match m.m.insert(key,value) {
        Some(_) => {},
        None => {}
      }
      Ok(())
    },
    _ => type_error("Type error in a[i]=value: a is not index-able.")
  }
}

fn operator_dot(sp: usize, stack: &mut Vec<Object>) -> FnResult {
  match stack[sp-2].clone() {
    Object::Table(t) => {
      match t.map.borrow().m.get(&stack[sp-1]) {
        Some(x) => {
          stack[sp-2] = x.clone();
          stack[sp-1] = Object::Null;
          Ok(())
        },
        None => index_error("Index error x.m: x has not property m.")
      }
    },
    _ => type_error("Type error in x.m: x is not a table.")
  }
}

#[inline(never)]
fn global_variable_not_found(module: &Module, index: u32, gtab: &RefCell<Map>) -> FnResult {
  let mut s =  String::new();
  s.push_str(&format!("Error: variable '{}' not found.",object_to_string(&module.data[index as usize])));
  // println!("gtab: {}",object_to_repr(&Object::Map(gtab.clone())));
  // panic!()
  return index_error(&s);
}

#[inline(always)]
fn compose_i32(a: &[u8], ip: usize) -> i32{
  (a[ip+3] as i32)<<24 | (a[ip+2] as i32)<<16 | (a[ip+1] as i32)<<8 | (a[ip] as i32)
  // unsafe{*((a.as_ptr().offset(ip as isize)) as *const i32)}
}

#[inline(always)]
fn compose_u32(a: &[u8], ip: usize) -> u32{
  (a[ip+3] as u32)<<24 | (a[ip+2] as u32)<<16 | (a[ip+1] as u32)<<8 | (a[ip] as u32)
  // unsafe{*((a.as_ptr().offset(ip as isize)) as *const u32)}
}

#[inline(always)]
fn compose_u64(a: &[u8], ip: usize) -> u64{
  (a[ip+7] as u64)<<56 | (a[ip+6] as u64)<<48 | (a[ip+5] as u64)<<40 | (a[ip+4] as u64)<<32 |
  (a[ip+3] as u64)<<24 | (a[ip+2] as u64)<<16 | (a[ip+1] as u64)<< 8 | (a[ip] as u64)
  // unsafe{*((a.as_ptr().offset(ip as isize)) as *const u64)}
}

pub struct Module{
  pub program: Rc<Vec<u8>>,
  // pub program: Vec<u8>,

  // pub program: Rc<[u8]>,
  // Rc<[T]> is available in Rust version 1.21 onwards.

  pub data: Vec<Object>
}

struct Frame{
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

struct State{
  stack: Vec<Object>,
  sp: usize,
  frame_stack: Vec<Frame>
}

const STACK_SIZE: usize = 2000;
const FRAME_STACK_SIZE: usize = 200;

fn vm_loop(state: &mut State, module: Rc<Module>, gtab: Rc<RefCell<Map>>)
  -> FnResult
{
  let mut stack = &mut state.stack;
  let mut module = module;
  let mut gtab = gtab;
  // let mut a: &[u8] = unsafe{&*(&module.program as &[u8] as *const [u8])};
  let mut a = module.program.clone();
  let mut ip=0;
  let mut sp=state.sp;
  let mut bp=sp;
  let mut argv_ptr=0;
  let mut fnself = Rc::new(Function{
      f: EnumFunction::Plain(::global::fpanic),
      argc: 0, argc_min: 0, argc_max: 0
  });

  let mut ret = true;
  let mut catch = false;

  loop{
    // print_stack(&stack[0..10]);
    // print_op(a[ip]);
    // match unsafe{*a.get_unchecked(ip)} {
    match a[ip] {
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
        stack[sp] = Object::Int(compose_i32(&a,ip+BCSIZE));
        sp+=1;
        ip+=BCSIZEP4;
      },
      bc::FLOAT => {
        stack[sp] = Object::Float(unsafe{
          transmute::<u64,f64>(compose_u64(&a,ip+BCSIZE))
        });
        sp+=1;
        ip+=BCSIZEP8;
      },
      bc::IMAG => {
        stack[sp] = Object::Complex(Complex64{re: 0.0,
          im: unsafe{transmute::<u64,f64>(compose_u64(&a,ip+BCSIZE))}
        });
        sp+=1;
        ip+=BCSIZEP8;
      },
      bc::STR => {
        let index = compose_u32(&a,ip+BCSIZE);
        stack[sp] = module.data[index as usize].clone();
        sp+=1;
        ip+=BCSIZEP4;
      },
      bc::LOAD_ARG => {
        let index = compose_u32(&a,ip+BCSIZE) as usize;
        stack[sp] = stack[argv_ptr+index].clone();
        sp+=1;
        ip+=BCSIZEP4;
      },
      bc::LOAD_LOCAL => {
        let index = compose_u32(&a,ip+BCSIZE) as usize;
        stack[sp] = stack[bp+index].clone();
        sp+=1;
        ip+=BCSIZEP4;
      },
      bc::STORE_LOCAL => {
        sp-=1;
        let index = compose_u32(&a,ip+BCSIZE) as usize;
        stack[bp+index] = replace(&mut stack[sp],Object::Null);
        ip+=BCSIZEP4;
      },
      bc::FNSELF => {
        ip+=BCSIZE;
        stack[sp] = Object::Function(fnself.clone());
        sp+=1;
      },
      bc::NEG => {
        try!(operator_neg(sp, &mut stack));
        ip+=BCSIZE;
      },
      bc::ADD => {
        try!(operator_plus(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::SUB => {
        try!(operator_minus(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::MPY => {
        try!(operator_mpy(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::DIV => {
        try!(operator_div(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::IDIV => {
        try!(operator_idiv(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::POW => {
        try!(operator_pow(sp, &mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::EQ => {
        let value = stack[sp-2]==stack[sp-1];
        sp-=1;
        stack[sp] = Object::Null;
        stack[sp-1] = Object::Bool(value);
        ip+=BCSIZE;
      },
      bc::NE => {
        let value = stack[sp-2]!=stack[sp-1];
        sp-=1;
        stack[sp] = Object::Null;
        stack[sp-1] = Object::Bool(value);
        ip+=BCSIZE;
      },
      bc::LT => {
        try!(operator_lt(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::GT => {
        try!(operator_gt(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::LE => {
        try!(operator_le(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::GE => {
        try!(operator_ge(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::IS => {
        try!(operator_is(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;      
      },
      bc::LIST => {
        let size = compose_u32(&a,ip+BCSIZE) as usize;
        sp = operator_list(sp,&mut stack,size);
        ip+=BCSIZEP4;
      },
      bc::MAP => {
        let size = compose_u32(&a,ip+BCSIZE) as usize;
        sp = operator_map(sp,&mut stack,size);
        ip+=BCSIZEP4;
      },
      bc::JMP => {
        ip = (ip as i32+compose_i32(&a,ip+BCSIZE)) as usize;
      },
      bc::JZ => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>{sp-=1; x},
          _ => panic!()
        };
        if condition {
          ip+=BCSIZEP4;
        }else{
          ip = (ip as i32+compose_i32(&a,ip+BCSIZE)) as usize;
        }
      },
      bc::JNZ => {
        let condition = match stack[sp-1] {
          Object::Bool(x)=>{sp-=1; x},
          _ => panic!()
        };
        if condition {
          ip = (ip as i32+compose_i32(&a,ip+BCSIZE)) as usize;
        }else{
          ip+=BCSIZEP4;
        }
      },
      bc::CALL => {
        ip+=BCSIZEP4;
        let argc = compose_u32(&a,ip-4) as usize;
        let f = stack[sp-argc-2].clone();
        match f {
          Object::Function(ref f) => {
            match f.f {
              EnumFunction::Plain(fp) => {
                let mut y = Object::Null;
                match fp(&mut y, &stack[sp-argc-1], &stack[sp-argc..sp]) {
                  Ok(()) => {},
                  Err(e) => {
                    return Err(e);
                  }
                }
                sp-=argc+1;
                stack[sp-1]=y;
                continue;
              },
              EnumFunction::Std(ref sf) => {
                if argc != f.argc as usize {
                  state.sp = sp;
                  return argc_error(argc, f.argc_min, f.argc_max, "anonymous function");
                }
                state.frame_stack.push(Frame{
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
                ip = sf.address;
                argv_ptr = sp-argc-1;
                ret = false;
                catch = false;

                bp = sp;
                for _ in 0..sf.var_count {
                  stack[sp] = Object::Null;
                  sp+=1;
                }

                continue;
              }
            }
          },
          _ => {
            state.sp = sp;
            return type_error("Type error in f(...): f is not callable.");
          }
        }
      },
      bc::RET => {
        if ret {
          state.sp = sp;
          return Ok(());
        }
        let frame = state.frame_stack.pop().unwrap();
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
        let index = compose_u32(&a,ip+BCSIZE);
        match gtab.borrow().m.get(&module.data[index as usize]) {
          Some(x) => {
            stack[sp] = x.clone();
            sp+=1;
          },
          None => {
            return global_variable_not_found(&module,index,&gtab);
          }
        }
        ip+=BCSIZEP4;
      },
      bc::STORE => {
        let index = compose_u32(&a,ip+BCSIZE);
        let key = module.data[index as usize].clone();
        gtab.borrow_mut().m.insert(key,replace(&mut stack[sp-1],Object::Null));
        sp-=1;
        ip+=BCSIZEP4;
      },
      bc::STORE_ARG => {
        sp-=1;
        let index = compose_u32(&a,ip+BCSIZE) as usize;
        stack[argv_ptr+index] = replace(&mut stack[sp],Object::Null);
        ip+=BCSIZEP4;
      },
      bc::STORE_CONTEXT => {
        panic!()
      },
      bc::LOAD_CONTEXT => {
        let index = compose_u32(&a,ip+BCSIZE) as usize;
        match fnself.f {
          EnumFunction::Std(ref sf) => {
            stack[sp] = sf.context.borrow().v[index].clone();
          },
          _ => panic!()
        }
        sp+=1;
        ip+=BCSIZEP4;
      },
      bc::FN => {
        ip+=BCSIZE+16;
        let address = (ip as i32-20+compose_i32(&a,ip-16)) as usize;
        // println!("fn [ip = {}]",address);
        let argc_min = compose_u32(&a,ip-12);
        let argc_max = compose_u32(&a,ip-8);
        let var_count = compose_u32(&a,ip-4);
        let context = match replace(&mut stack[sp-2],Object::Null) {
          Object::List(a) => a,
          Object::Null => Rc::new(RefCell::new(List::new())),
          _ => panic!()
        };
        sp-=1;
        stack[sp-1] = Function::new(StandardFn{
          address: address,
          module: module.clone(),
          gtab: gtab.clone(),
          var_count: var_count,
          context: context
        },argc_min,argc_max);
      },
      bc::GET_INDEX => {
        try!(operator_index(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::SET_INDEX => {
        try!(index_assignment(sp,&mut stack));
        sp-=3;
        ip+=BCSIZE;
      },
      bc::DOT => {
        try!(operator_dot(sp,&mut stack));
        sp-=1;
        ip+=BCSIZE;
      },
      bc::POP => {
        sp-=1;
        stack[sp] = Object::Null;
        ip+=BCSIZE;
      },
      bc::HALT => {
        state.sp=sp;
        return Ok(());
      },
      _ => {panic!()}
    }
  }
}

fn list_from_slice(a: &[Object]) -> Object {
  let n = a.len();
  let mut v: Vec<Object> = Vec::with_capacity(n);
  for i in 0..n {
    v.push(a[i].clone());
  }
  return List::new_object(v);
}

pub fn eval(module: Rc<Module>, gtab: Rc<RefCell<Map>>, command_line: bool)
  -> Result<Object,Box<Exception>>
{
  let mut stack: Vec<Object> = Vec::with_capacity(STACK_SIZE);
  for _ in 0..STACK_SIZE {
    stack.push(Object::Null);
  }
  let mut frame_stack: Vec<Frame> = Vec::with_capacity(FRAME_STACK_SIZE);

  let mut state = State{stack, frame_stack, sp: 0};
  try!(vm_loop(&mut state, module, gtab));

  let stack = state.stack;
  let sp = state.sp;
  if command_line {
    for i in 0..sp {
      match stack[i] {
        Object::Null => {},
        _ => {
          println!("{}",object_to_repr(&stack[i]));
        }
      }
    }
    return Ok(Object::Null);
  }else{
    if sp==0 {
      return Ok(Object::Null);
    }else if sp==1 {
      return Ok(stack[0].clone());
    }else{
      return Ok(list_from_slice(&stack[0..sp]));
    }
  }
}
