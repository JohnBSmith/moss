
#![allow(unused_imports)]

use std::f64::consts::{PI,E,LOG10_E};
use std::rc::Rc;
use complex::Complex64;
use object::{Object, FnResult, Function,
  type_error, argc_error, new_module
};

fn lanczos_gamma(x: f64) -> f64 {
  let p=[
    0.99999999999980993, 676.5203681218851, -1259.1392167224028,
    771.32342877765313, -176.61502916214059, 12.507343278686905,
    -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7
  ];
  let x = x-1.0;
  let mut y = p[0];
  y+=p[1]/(x+1.0); y+=p[2]/(x+2.0);
  y+=p[3]/(x+3.0); y+=p[4]/(x+4.0);
  y+=p[5]/(x+5.0); y+=p[6]/(x+6.0);
  y+=p[7]/(x+7.0); y+=p[8]/(x+8.0);
  let t=x+7.5;
  return (2.0*PI).sqrt()*t.powf(x+0.5)*(-t).exp()*y;
}

pub fn gamma(x: f64) -> f64 {
  if x<0.5 {
    return PI/(x*PI).sin()/lanczos_gamma(1.0-x);
  }else{
    return lanczos_gamma(x);
  }
}

fn lanczos_cgamma(z: Complex64) -> Complex64 {
  let p=[
    0.99999999999980993, 676.5203681218851, -1259.1392167224028,
    771.32342877765313, -176.61502916214059, 12.507343278686905,
    -0.13857109526572012, 9.9843695780195716e-6, 1.5056327351493116e-7
  ];
  let z = z-1.0;
  let mut y = Complex64{re: p[0], im: 0.0};
  y=y+p[1]/(z+1.0); y=y+p[2]/(z+2.0);
  y=y+p[3]/(z+3.0); y=y+p[4]/(z+4.0);
  y=y+p[5]/(z+5.0); y=y+p[6]/(z+6.0);
  y=y+p[7]/(z+7.0); y=y+p[8]/(z+8.0);
  let t=z+7.5;
  return (2.0*PI).sqrt()*t.pow(z+0.5)*(-t).exp()*y;
}

pub fn cgamma(z: Complex64) -> Complex64 {
  if z.re<0.5 {
    return PI/(PI*z).sin()/lanczos_cgamma(1.0-z);
  }else{
    return lanczos_cgamma(z);
  }
}

fn floor(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"floor");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).floor()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.floor()))
    },
    _ => type_error("Type error in floor(x): x is not a number.")
  }
}

fn ceil(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"ceil");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).ceil()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.ceil()))
    },
    _ => type_error("Type error in ceil(x): x is not a number.")
  }
}

fn sqrt(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sqrt");
  }
  match argv[0] {
    Object::Float(x) => {
      Ok(Object::Float(x.sqrt()))
    },
    Object::Int(x) => {
      Ok(Object::Float((x as f64).sqrt()))
    },
    _ => type_error("Type error in sqrt(x): x is not a number.")
  }
}

fn exp(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"exp");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).exp()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.exp()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.exp()))
    },
    _ => type_error("Type error in exp(x): x is not a number.")
  }
}

fn ln(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"ln");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).ln()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.ln()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.ln()))
    },
    _ => type_error("Type error in ln(x): x is not a number.")
  }
}

fn lg(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"lg");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float(LOG10_E*(x as f64).ln()))
    },
    Object::Float(x) => {
      Ok(Object::Float(LOG10_E*x.ln()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(LOG10_E*z.ln()))
    },
    _ => type_error("Type error in lg(x): x is not a number.")
  }
}


fn sin(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sin");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).sin()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.sin()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.sin()))
    },
    _ => type_error("Type error in sin(x): x is not a number.")
  }
}

fn cos(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"cos");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).cos()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.cos()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.cos()))
    },
    _ => type_error("Type error in cos(x): x is not a number.")
  }
}

fn tan(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"tan");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).tan()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.tan()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.tan()))
    },
    _ => type_error("Type error in tan(x): x is not a number.")
  }
}

fn sinh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sinh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).sinh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.sinh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.sinh()))
    },
    _ => type_error("Type error in sinh(x): x is not a number.")
  }
}

fn cosh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"cosh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).cosh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.cosh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.cosh()))
    },
    _ => type_error("Type error in cosh(x): x is not a number.")
  }
}

fn tanh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"tanh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).tanh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.tanh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.tanh()))
    },
    _ => type_error("Type error in tanh(x): x is not a number.")
  }
}

fn asin(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"asin");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).asin()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.asin()))
    },
    _ => type_error("Type error in asin(x): x is not a number.")
  }
}

fn acos(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"acos");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).acos()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.acos()))
    },
    _ => type_error("Type error in acos(x): x is not a number.")
  }
}

fn atan(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"atan");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).atan()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.atan()))
    },
    _ => type_error("Type error in atan(x): x is not a number.")
  }
}

fn asinh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"asinh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).asinh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.asinh()))
    },
    _ => type_error("Type error in asinh(x): x is not a number.")
  }
}

fn acosh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"acosh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).acosh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.acosh()))
    },
    _ => type_error("Type error in acosh(x): x is not a number.")
  }
}

fn atanh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"atanh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float((x as f64).atanh()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x.atanh()))
    },
    _ => type_error("Type error in atanh(x): x is not a number.")
  }
}

fn fgamma(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"gamma");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Float(gamma(x as f64)))
    },
    Object::Float(x) => {
      Ok(Object::Float(gamma(x)))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(cgamma(z)))
    },
    _ => type_error("Type error in gamma(x): x is not a number.")
  }
}

fn hypot(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),2,2,"hypot");
  }
  match argv[0] {
    Object::Float(x) => {
      match argv[1] {
        Object::Float(y) => {
          Ok(Object::Float(x.hypot(y)))
        },
        Object::Int(y) => {
          Ok(Object::Float(x.hypot(y as f64)))
        },
        _ => type_error("Type error in hypot(x,y): y is not a float.")
      }
    },
    Object::Int(x) => {
      match argv[1] {
        Object::Float(y) => {
          Ok(Object::Float((x as f64).hypot(y)))
        },
        Object::Int(y) => {
          Ok(Object::Float((x as f64).hypot(y as f64)))
        },
        _ => type_error("Type error in hypot(x,y): y is not a float.")
      }
    },
    _ => type_error("Type error in hypot(x,y): x is not a float.")
  }
}

fn atan2(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),2,2,"atan2");
  }
  match argv[0] {
    Object::Float(y) => {
      match argv[1] {
        Object::Float(x) => {
          Ok(Object::Float(y.atan2(x)))
        },
        Object::Int(x) => {
          Ok(Object::Float(y.atan2(x as f64)))
        },
        _ => type_error("Type error in atan2(y,x): x is not a float.")
      }
    },
    Object::Int(y) => {
      match argv[1] {
        Object::Float(x) => {
          Ok(Object::Float((y as f64).atan2(x)))
        },
        Object::Int(x) => {
          Ok(Object::Float((y as f64).atan2(x as f64)))
        },
        _ => type_error("Type error in atan2(y,x): x is not a float.")
      }
    },
    _ => type_error("Type error in atan2(y,x): y is not a float.")
  }
}

fn re(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"re");
  }
  match argv[0] {
    Object::Complex(z) => {
      Ok(Object::Float(z.re))
    },
    Object::Float(x) => {
      Ok(Object::Float(x))
    },
    Object::Int(x) => {
      Ok(Object::Int(x))
    },
    _ => type_error("Type error in re(z): z is not a number.")
  }
}

fn im(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"im");
  }
  match argv[0] {
    Object::Complex(z) => {
      Ok(Object::Float(z.im))
    },
    Object::Float(x) => {
      Ok(Object::Float(x))
    },
    Object::Int(x) => {
      Ok(Object::Int(x))
    },
    _ => type_error("Type error in im(z): z is not a number.")
  }
}

fn conj(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"conj");
  }
  match argv[0] {
    Object::Complex(z) => {
      Ok(Object::Complex(z.conj()))
    },
    Object::Float(x) => {
      Ok(Object::Float(x))
    },
    Object::Int(x) => {
      Ok(Object::Int(x))
    },
    _ => type_error("Type error in conj(z): z is not a number.")
  }
}

fn csqrt(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"csqrt");
  }
  match argv[0] {
    Object::Complex(z) => {
      Ok(Object::Complex(z.sqrt()))
    },
    Object::Float(x) => {
      if x<0.0 {
        Ok(Object::Complex(Complex64{re: x, im: 0.0}.sqrt()))
      }else{
        Ok(Object::Float(x.sqrt()))
      }
    },
    Object::Int(x) => {
      if x<0 {
        Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.sqrt()))
      }else{
        Ok(Object::Float((x as f64).sqrt()))
      }
    },
    _ => type_error("Type error in csqrt(z): z is not a number.")
  }
}

fn cln(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"ln");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.ln()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.ln()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.ln()))
    },
    _ => type_error("Type error in ln(z): z is not a number.")
  }
}

fn casin(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"asin");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.asin()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.asin()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.asin()))
    },
    _ => type_error("Type error in asin(z): z is not a number.")
  }
}

fn cacos(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"acos");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.acos()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.acos()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.acos()))
    },
    _ => type_error("Type error in acos(z): z is not a number.")
  }
}

fn catan(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"atan");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.atan()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.atan()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.atan()))
    },
    _ => type_error("Type error in atan(z): z is not a number.")
  }
}

fn casinh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"asinh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.asinh()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.asinh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.asinh()))
    },
    _ => type_error("Type error in asinh(z): z is not a number.")
  }
}

fn cacosh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"acosh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.acosh()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.acosh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.acosh()))
    },
    _ => type_error("Type error in acosh(z): z is not a number.")
  }
}

fn catanh(pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"atanh");
  }
  match argv[0] {
    Object::Int(x) => {
      Ok(Object::Complex(Complex64{re: x as f64, im: 0.0}.atanh()))
    },
    Object::Float(x) => {
      Ok(Object::Complex(Complex64{re: x, im: 0.0}.atanh()))
    },
    Object::Complex(z) => {
      Ok(Object::Complex(z.atanh()))
    },
    _ => type_error("Type error in atanh(z): z is not a number.")
  }
}


pub fn load_math() -> Object {
  let math = new_module("math");
  {
    let mut m = math.map.borrow_mut();
    m.insert("pi",  Object::Float(PI));
    m.insert("e",   Object::Float(E));
    m.insert("nan", Object::Float(::std::f64::NAN));
    m.insert("inf", Object::Float(::std::f64::INFINITY));

    m.insert_fn_plain("floor",floor,1,1);
    m.insert_fn_plain("ceil",ceil,1,1);
    m.insert_fn_plain("sqrt",sqrt,1,1);
    m.insert_fn_plain("exp",exp,1,1);
    m.insert_fn_plain("ln",ln,1,1);
    m.insert_fn_plain("lg",lg,1,1);

    m.insert_fn_plain("sin",sin,1,1);
    m.insert_fn_plain("cos",cos,1,1);
    m.insert_fn_plain("tan",tan,1,1);
    m.insert_fn_plain("sinh",sinh,1,1);
    m.insert_fn_plain("cosh",cosh,1,1);
    m.insert_fn_plain("tanh",tanh,1,1);

    m.insert_fn_plain("asin",asin,1,1);
    m.insert_fn_plain("acos",acos,1,1);
    m.insert_fn_plain("atan",atan,1,1);
    m.insert_fn_plain("asinh",asinh,1,1);
    m.insert_fn_plain("acosh",acosh,1,1);
    m.insert_fn_plain("atanh",atanh,1,1);

    m.insert_fn_plain("gamma",fgamma,1,1);
    m.insert_fn_plain("hypot",hypot,2,2);
    m.insert_fn_plain("atan2",atan2,2,2);
  }
  return Object::Table(Rc::new(math));
}

pub fn load_cmath() -> Object {
  let cmath = new_module("cmath");
  {
    let mut m = cmath.map.borrow_mut();
    m.insert_fn_plain("exp",exp,1,1);
    m.insert_fn_plain("sin",sin,1,1);
    m.insert_fn_plain("cos",cos,1,1);
    m.insert_fn_plain("tan",tan,1,1);
    m.insert_fn_plain("sinh",sinh,1,1);
    m.insert_fn_plain("cosh",cosh,1,1);
    m.insert_fn_plain("tanh",tanh,1,1);

    m.insert_fn_plain("asin",casin,1,1);
    m.insert_fn_plain("acos",cacos,1,1);
    m.insert_fn_plain("atan",catan,1,1);
    m.insert_fn_plain("asinh",casinh,1,1);
    m.insert_fn_plain("acosh",cacosh,1,1);
    m.insert_fn_plain("atanh",catanh,1,1);

    m.insert_fn_plain("re",re,1,1);
    m.insert_fn_plain("im",im,1,1);
    m.insert_fn_plain("conj",conj,1,1);
    m.insert_fn_plain("ln",cln,1,1);
    m.insert_fn_plain("sqrt",csqrt,1,1);
  }
  return Object::Table(Rc::new(cmath));
}
