
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

pub const BCSIZE: usize = 4;
pub const BCSIZEP4: usize = BCSIZE+4;

pub mod bc{
  pub const NULL: u8 = 00;
  pub const HALT: u8 = 01;
  pub const FALSE:u8 = 02;
  pub const TRUE: u8 = 03;
  pub const INT:  u8 = 04;
  pub const NEG:  u8 = 05;
  pub const ADD:  u8 = 06;
  pub const SUB:  u8 = 07;
  pub const MPY:  u8 = 08;
  pub const DIV:  u8 = 09;
  pub const IDIV: u8 = 10;
  pub const LIST: u8 = 11;
  pub const MAP:  u8 = 12;
  pub const EQ:   u8 = 13;
  pub const NE:   u8 = 14;
  pub const LOAD: u8 = 15;
  pub const STORE:u8 = 16;
}

pub struct U32String{
  pub v: Vec<char>
}

impl U32String{
  pub fn new_object(v: Vec<char>) -> Object{
    return Object::String(Rc::new(U32String{v: v}));
  }
  pub fn new_object_str(s: &str) -> Object{
    return Object::String(Rc::new(U32String{v: s.chars().collect()}));
  }
}

pub struct List{
  pub v: Vec<Object>
}

impl List{
  pub fn new_object(v: Vec<Object>) -> Object{
    return Object::List(Rc::new(RefCell::new(List{v: v})));
  }
}

pub struct Map{
  m: HashMap<Object,Object>
}

impl Map{
  pub fn new_object(m: HashMap<Object,Object>) -> Object{
    return Object::Map(Rc::new(RefCell::new(Map{m: m})));
  }
  pub fn new() -> Rc<RefCell<Map>>{
    return Rc::new(RefCell::new(Map{m: HashMap::new()}));
  }
}

pub enum Object{
  Null, Bool(bool), Int(i32), Float(f64),
  List(Rc<RefCell<List>>), String(Rc<U32String>),
  Map(Rc<RefCell<Map>>)
}

impl Object{
  pub fn clone(x: &Object) -> Object{
    match x {
      &Object::Null => {Object::Null},
      &Object::Bool(x) => {Object::Bool(x)},
      &Object::Int(x) => {Object::Int(x)},
      &Object::Float(x) => {Object::Float(x)},
      &Object::String(ref x) => {Object::String(x.clone())},
      &Object::List(ref x) => {Object::List(x.clone())},
      &Object::Map(ref x) => {Object::Map(x.clone())}
    }
  }
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
          _ => false
        }
      },
      &Object::String(ref x) => {
        match b {
          &Object::String(ref y) => {
            x.v==y.v
          },
          _ => false
        }
      },
      _ => false
    }
  }
}
impl Eq for Object{}

impl Hash for Object{
  fn hash<H: Hasher>(&self, state: &mut H){
    match self {
      &Object::Bool(x) => {x.hash(state);},
      &Object::Int(x) => {x.hash(state);},
      &Object::String(ref x) => {
        let s = &x.v;
        s.hash(state);
      },
      _ => panic!()
    }
  }
}

pub struct Module{
  pub program: Vec<u8>,
  pub data: Vec<Object>
}

fn list_to_string(a: &[Object]) -> String{
  let mut s = String::from("[");
  for i in 0..a.len() {
    if i!=0 {s.push_str(", ");}
    s.push_str(&object_to_string(&a[i]));
  }
  s.push_str("]");
  return s;
}

fn map_to_string(a: &HashMap<Object,Object>) -> String{
  let mut s = String::from("{");
  let mut first=true;
  for (key,value) in a {
    if first {first=false;} else{s.push_str(", ");}
    s.push_str(&object_to_string(&key));
    match value {
      &Object::Null => {},
      _ => {
        s.push_str(": ");
        s.push_str(&object_to_string(&value));
      }
    }
  }
  s.push_str("}");
  return s;
}

fn object_to_string(x: &Object) -> String{
  match x {
    &Object::Null => String::from("null"),
    &Object::Bool(b) => String::from(if b {"true"} else {"false"}),
    &Object::Int(i) => format!("{}",i),
    &Object::Float(f) => format!("{}",f),
    &Object::List(ref a) => {
      list_to_string(&a.borrow().v)
    },
    &Object::Map(ref a) => {
      map_to_string(&a.borrow().m)
    },
    &Object::String(ref a) => {
      let s: String = a.v.iter().cloned().collect();
      format!("\"{}\"",s)
    }
  }
}

fn operator_plus(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x+y);
          return sp-1;
        },
        _ => panic!()
      }
    },
    _ => panic!()
  }
}

fn operator_minus(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x-y);
          return sp-1;
        },
        _ => panic!()
      }
    },
    _ => panic!()
  }
}

fn operator_mpy(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x*y);
          return sp-1;
        },
        _ => panic!()
      }
    },
    _ => panic!()
  }
}

fn operator_div(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Float((x as f64)/(y as f64));
          return sp-1;
        },
        _ => panic!()
      }
    },
    _ => panic!()
  }
}

fn operator_idiv(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-2] = Object::Int(x/y);
          return sp-1;
        },
        _ => panic!()
      }
    },
    _ => panic!()
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

fn compose_i32(b1: u8, b2: u8, b3: u8, b4: u8) -> i32{
  return (b4 as i32)<<24 | (b3 as i32)<<16 | (b2 as i32)<<8 | (b1 as i32);
}

fn compose_u32(b1: u8, b2: u8, b3: u8, b4: u8) -> u32{
  return (b4 as u32)<<24 | (b3 as u32)<<16 | (b2 as u32)<<8 | (b1 as u32);
}

pub fn eval(module: &Module, a: &[u8], gtab: &Rc<RefCell<Map>>){
  let mut ip=0;
  let mut stack: Vec<Object> = Vec::new();
  for _ in 0..100 {
    stack.push(Object::Null);
  }
  let mut sp=0;
  loop{
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
        ip+=BCSIZEP4;
        stack[sp] = Object::Int(compose_i32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]));
        sp+=1;
      },
      bc::LOAD => {
        ip+=BCSIZEP4;
        let index = compose_u32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]);
        match gtab.borrow().m.get(&module.data[index as usize]) {
          Some(x) => {
            stack[sp] = Object::clone(x);
            sp+=1;
          },
          None => {
            panic!()
          }
        }
      },
      bc::STORE => {
        ip+=BCSIZEP4;
        let index = compose_u32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]);
        let key = Object::clone(&module.data[index as usize]);
        gtab.borrow_mut().m.insert(key,replace(&mut stack[sp-1],Object::Null));
        sp-=1;
      },
      bc::ADD => {
        sp = operator_plus(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::SUB => {
        sp = operator_minus(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::MPY => {
        sp = operator_mpy(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::DIV => {
        sp = operator_div(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::IDIV => {
        sp = operator_idiv(sp, &mut stack);
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
      bc::LIST => {
        ip+=BCSIZEP4;
        let size = compose_i32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]) as usize;
        sp = operator_list(sp,&mut stack,size);
      },
      bc::MAP => {
        ip+=BCSIZEP4;
        let size = compose_i32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]) as usize;
        sp = operator_map(sp,&mut stack,size);
      },
      bc::HALT => {
        break;
      },
      _ => {panic!()}
    }
  }
  if sp>0 {
    println!("{}",object_to_string(&stack[sp-1]));
  }
}
