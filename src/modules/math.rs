
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

fn floor(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"floor");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).floor());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.floor());
      Ok(())
    },
    _ => type_error("Type error in floor(x): x is not a number.")
  }
}

fn ceil(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"ceil");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).ceil());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.ceil());
      Ok(())
    },
    _ => type_error("Type error in ceil(x): x is not a number.")
  }
}

fn sqrt(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sqrt");
  }
  match argv[0] {
    Object::Float(x) => {
      *ret = Object::Float(x.sqrt());
      Ok(())
    },
    Object::Int(x) => {
      *ret = Object::Float((x as f64).sqrt());
      Ok(())
    },
    _ => type_error("Type error in sqrt(x): x is not a number.")
  }
}

fn exp(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"exp");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).exp());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.exp());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.exp());
      Ok(())
    },
    _ => type_error("Type error in exp(x): x is not a number.")
  }
}

fn ln(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"ln");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).ln());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.ln());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.ln());
      Ok(())
    },
    _ => type_error("Type error in ln(x): x is not a number.")
  }
}

fn lg(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"lg");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float(LOG10_E*(x as f64).ln());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(LOG10_E*x.ln());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(LOG10_E*z.ln());
      Ok(())
    },
    _ => type_error("Type error in lg(x): x is not a number.")
  }
}


fn sin(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sin");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).sin());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.sin());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.sin());
      Ok(())
    },
    _ => type_error("Type error in sin(x): x is not a number.")
  }
}

fn cos(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"cos");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).cos());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.cos());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.cos());
      Ok(())
    },
    _ => type_error("Type error in cos(x): x is not a number.")
  }
}

fn tan(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"tan");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).tan());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.tan());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.tan());
      Ok(())
    },
    _ => type_error("Type error in tan(x): x is not a number.")
  }
}

fn sinh(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"sinh");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).sinh());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.sinh());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.sinh());
      Ok(())
    },
    _ => type_error("Type error in sinh(x): x is not a number.")
  }
}

fn cosh(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"cosh");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).cosh());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.cosh());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.cosh());
      Ok(())
    },
    _ => type_error("Type error in cosh(x): x is not a number.")
  }
}

fn tanh(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"tanh");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float((x as f64).tanh());
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(x.tanh());
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(z.tanh());
      Ok(())
    },
    _ => type_error("Type error in tanh(x): x is not a number.")
  }
}

fn fgamma(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),1,1,"gamma");
  }
  match argv[0] {
    Object::Int(x) => {
      *ret = Object::Float(gamma(x as f64));
      Ok(())
    },
    Object::Float(x) => {
      *ret = Object::Float(gamma(x));
      Ok(())
    },
    Object::Complex(z) => {
      *ret = Object::Complex(cgamma(z));
      Ok(())
    },
    _ => type_error("Type error in gamma(x): x is not a number.")
  }
}

fn hypot(ret: &mut Object, pself: &Object, argv: &[Object]) -> FnResult{
  if argv.len() != 1 {
    return argc_error(argv.len(),2,2,"hypot");
  }
  match argv[0] {
    Object::Float(x) => {
      match argv[1] {
        Object::Float(y) => {
          *ret = Object::Float(x.hypot(y));
          Ok(())
        },
        Object::Int(y) => {
          *ret = Object::Float(x.hypot(y as f64));
          Ok(())
        },
        _ => type_error("Type error in hypot(x,y): y is not a float.")
      }
    },
    Object::Int(x) => {
      match argv[1] {
        Object::Float(y) => {
          *ret = Object::Float((x as f64).hypot(y));
          Ok(())
        },
        Object::Int(y) => {
          *ret = Object::Float((x as f64).hypot(y as f64));
          Ok(())
        },
        _ => type_error("Type error in hypot(x,y): y is not a float.")
      }
    },
    _ => type_error("Type error in hypot(x,y): x is not a float.")
  }
}

pub fn load_math() -> Object {
  let math = new_module("math");
  {
    let mut m = math.map.borrow_mut();
    m.insert("pi",   Object::Float(PI));
    m.insert("e",    Object::Float(E));
    m.insert("floor",Function::plain(floor,1,1));
    m.insert("ceil", Function::plain(ceil,1,1));
    m.insert("sqrt", Function::plain(sqrt,1,1));
    m.insert("exp",  Function::plain(exp,1,1));
    m.insert("ln",   Function::plain(ln,1,1));
    m.insert("lg",   Function::plain(lg,1,1));
    m.insert("sin",  Function::plain(sin,1,1));
    m.insert("cos",  Function::plain(cos,1,1));
    m.insert("tan",  Function::plain(tan,1,1));
    m.insert("sinh", Function::plain(sinh,1,1));
    m.insert("cosh", Function::plain(cosh,1,1));
    m.insert("tanh", Function::plain(tanh,1,1));
    m.insert("gamma",Function::plain(fgamma,1,1));
    m.insert("hypot",Function::plain(hypot,2,2));
  }
  return Object::Table(Rc::new(math));
}
