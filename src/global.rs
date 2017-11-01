
use vm::{object_to_string};
use moss::{Object, FnResult, U32String, type_error, argc_error};

pub fn print(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[i]));
  }
  println!();
  return Ok(());
}

pub fn put(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  for i in 0..argv.len() {
    print!("{}",object_to_string(&argv[0]));
  }
  return Ok(());
}

pub fn fstr(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"abs");
  }
  let s = object_to_string(&argv[0]);
  *ret = U32String::new_object_str(&s);
  return Ok(());
}

pub fn abs(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"abs");
  }
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
    _ => {
      return type_error("Type error in abs(x): x is not an int, float, complex.");
    }
  }
}
