
use std::rc::Rc;
use std::cell::RefCell;
use std::mem::replace;

pub const BCSIZE: usize = 4;
pub const BCSIZEP4: usize = BCSIZE+4;

pub mod bc{
  pub const HALT: u8 = 0;
  pub const INT:  u8 = 1;
  pub const NEG:  u8 = 2;
  pub const ADD:  u8 = 3;
  pub const SUB:  u8 = 4;
  pub const MPY:  u8 = 5;
  pub const DIV:  u8 = 6;
  pub const IDIV: u8 = 7;
  pub const LIST: u8 = 8;
}

pub struct List{
  v: Vec<Object>
}

impl List{
  pub fn new_object(v: Vec<Object>) -> Object{
    return Object::List(Rc::new(RefCell::new(List{v: v})));
  }
}

pub enum Object{
  Null, Bool(bool), Int(i32), Float(f64),
  List(Rc<RefCell<List>>)
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

fn object_to_string(x: &Object) -> String{
  match x {
    &Object::Null => String::from("null"),
    &Object::Bool(b) => String::from(if b {"true"} else {"false"}),
    &Object::Int(i) => format!("{}",i),
    &Object::List(ref a) => {
      list_to_string(&a.borrow().v)
    }
    _ => panic!()
  }
}

fn operator_plus(sp: usize, stack: &mut Vec<Object>) -> usize{
  match stack[sp-2] {
    Object::Int(x) => {
      match stack[sp-1] {
        Object::Int(y) => {
          stack[sp-1] = Object::Int(x+y);
          return sp;
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
          stack[sp-1] = Object::Int(x-y);
          return sp;
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

fn compose_i32(b1: u8, b2: u8, b3: u8, b4: u8) -> i32{
  return (b4 as i32)<<24 | (b3 as i32)<<16 | (b2 as i32)<<8 | (b1 as i32);
}

pub fn eval(a: &[u8]){
  let mut ip=0;
  let mut stack: Vec<Object> = Vec::new();
  for _ in 0..100 {
    stack.push(Object::Null);
  }
  let mut sp=0;
  loop{
    match a[ip] {
      bc::INT => {
        ip+=BCSIZEP4;
        stack[sp] = Object::Int(compose_i32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]));
        sp+=1;
      },
      bc::ADD => {
        sp = operator_plus(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::SUB => {
        sp = operator_minus(sp, &mut stack);
        ip+=BCSIZE;
      },
      bc::LIST => {
        ip+=BCSIZEP4;
        let size = compose_i32(a[ip-4],a[ip-3],a[ip-2],a[ip-1]) as usize;
        sp = operator_list(sp,&mut stack,size);
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
