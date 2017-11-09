
use vm::{object_to_string, object_to_repr};
use object::{Object, FnResult, U32String, type_error, argc_error};

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
    return argc_error(argv.len(),1,1,"str");
  }
  let s = object_to_string(&argv[0]);
  *ret = U32String::new_object_str(&s);
  return Ok(());
}

pub fn repr(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"repr");
  }
  let s = object_to_repr(&argv[0]);
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

pub fn fpanic(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  panic!()
}

pub fn eval(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"eval");
  }
  match argv[0] {
    Object::String(ref s) => {
      let a: String = s.v.iter().collect();
      return match ::eval_string(&a,"") {
        Ok(x) => {*ret=x; Ok(())},
        Err(e) => Err(e)
      }
    },
    _ => {
      return type_error("Type error in eval(s): s is not a string.");
    }
  }
}

pub fn size(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"size");
  }
  match argv[0] {
    Object::List(ref a) => {
      *ret = Object::Int(a.borrow().v.len() as i32);
      Ok(())
    },
    Object::Map(ref m) => {
      *ret = Object::Int(m.borrow().m.len() as i32);
      Ok(())
    },
    Object::String(ref s) => {
      *ret = Object::Int(s.v.len() as i32);
      Ok(())
    }
    _ => type_error("Type error in size(a): cannot determine the size of a.")
  }
}
