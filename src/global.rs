
use vm::object_to_string;
use moss::{Object, FnResult};

pub fn print(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  println!("{}",object_to_string(&argv[0]));
  return Ok(());
}

pub fn put(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  print!("{}",object_to_string(&argv[0]));
  return Ok(());
}

pub fn abs(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
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
    _ => panic!()
  }
}
