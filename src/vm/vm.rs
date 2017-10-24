
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
}

pub enum Object{
  Null, Bool(bool), Int(i32), Float(f64)
}

fn object_to_string(x: &Object) -> String{
  match x {
    &Object::Null => String::from("null"),
    &Object::Bool(b) => String::from(if b {"true"} else {"false"}),
    &Object::Int(i) => format!("{}",i),
    _ => panic!()
  }
}

pub fn operator_plus(sp: usize, stack: &mut Vec<Object>) -> usize{
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
