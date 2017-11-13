
use std::rc::Rc;
use complex::Complex64;
use object::{Object, FnResult, Function,
  type_error, argc_error, new_module
};

fn re(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"re");
  }
  match argv[0] {
    Object::Complex(z) => {
      *ret = Object::Float(z.re);
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x);
      Ok(())
    },
    Object::Int(x) => {
      *ret = Object::Int(x);
      Ok(())
    },
    _ => type_error("Type error in re(z): z is not a number.")
  }
}

fn im(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"im");
  }
  match argv[0] {
    Object::Complex(z) => {
      *ret = Object::Float(z.im);
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x);
      Ok(())
    },
    Object::Int(x) => {
      *ret = Object::Int(x);
      Ok(())
    },
    _ => type_error("Type error in im(z): z is not a number.")
  }
}

fn conj(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"conj");
  }
  match argv[0] {
    Object::Complex(z) => {
      *ret = Object::Complex(z.conj());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x);
      Ok(())
    },
    Object::Int(x) => {
      *ret = Object::Int(x);
      Ok(())
    },
    _ => type_error("Type error in conj(z): z is not a number.")
  }
}

fn csqrt(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"csqrt");
  }
  match argv[0] {
    Object::Complex(z) => {
      *ret = Object::Complex(z.sqrt());
      Ok(())
    },
    Object::Float(x) => {
      *ret = if x<0.0 {
        Object::Complex(Complex64{re: x, im: 0.0}.sqrt())
      }else{
        Object::Float(x.sqrt())
      };
      Ok(())
    },
    Object::Int(x) => {
      *ret = if x<0 {
        Object::Complex(Complex64{re: x as f64, im: 0.0}.sqrt())
      }else{
        Object::Float((x as f64).sqrt())
      };
      Ok(())
    },
    _ => type_error("Type error in csqrt(z): z is not a number.")
  }
}

pub fn load_cmath() -> Object {
  let cmath = new_module("cmath");
  {
    let mut m = cmath.map.borrow_mut();
    m.insert("re",Function::plain(re,1,1));
    m.insert("im",Function::plain(im,1,1));
    m.insert("conj",Function::plain(conj,1,1));
    m.insert("sqrt",Function::plain(csqrt,1,1));
  }
  return Object::Table(Rc::new(cmath));
}

