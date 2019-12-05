
#![allow(non_snake_case)]

// Special mathematical functions
// == Literature ==
// [1] Richard P. Brent: "Fast Algorithms for High-Precision
// Computation of Elementary Functions", 2006
// [2] Semjon Adlaj: "An eloquent formula for the perimeter
// of an ellipse", Notices of the AMS 59(8) (2012)
// [3] Semjon Adlaj: "An arithmetic-geometric mean of a
// third kind!", 2015

use std::f64::NAN;
use std::f64::consts::{PI};
use std::rc::Rc;

use crate::object::{Object, FnResult, float, new_module};
use crate::vm::Env;
use crate::math::{gamma, lgamma, sgngamma, cgamma};
use crate::complex::c64;

const SQRT_PI: f64 = 1.7724538509055159;

fn agm(mut x: f64, mut y: f64) -> f64 {
    for _ in 0..20 {
        let xh = (x+y)/2.0;
        let yh = (x*y).sqrt();
        x=xh; y=yh;
        if (x-y).abs()<1E-15 {break;}
    }
    return x;
}

// Modified arithmetic-geometric mean, see
// Semjon Adlaj: "An eloquent formula for the perimeter
// of an ellipse", Notices of the AMS 59(8) (2012), p. 1094-1099.
fn magm(mut x: f64, mut y: f64) -> f64 {
    let mut z=0.0;
    for _ in 0..20 {
        let xh = 0.5*(x+y);
        let r = ((x-z)*(y-z)).sqrt();
        let yh = z+r;
        let zh = z-r;
        x=xh; y=yh; z=zh;
        if (x-y).abs()<2E-15 {break;}
    }
    return x;
}

// m=k*k
fn cK(m: f64) -> f64 {
    return 0.5*PI/agm(1.0,(1.0-m).sqrt());
}

// m=k*k
fn cE(m: f64) -> f64 {
    let M = agm(1.0,(1.0-m).sqrt());
    let N = magm(1.0,1.0-m);
    return 0.5*PI*N/M;
}

fn RF(mut x: f64, mut y: f64, mut z: f64) -> f64 {
    for _ in 0..26 {
        let a = (x*y).sqrt()+(x*z).sqrt()+(y*z).sqrt();
        x=0.25*(x+a); y=0.25*(y+a); z=0.25*(z+a);
    }
    return 1.0/(x).sqrt();
}

fn RC(x: f64, y: f64) -> f64 {
    return RF(x,y,y);
}

fn RJ(mut x: f64, mut y: f64, mut z: f64, mut p: f64) -> f64 {
    let delta = (p-x)*(p-y)*(p-z);
    let mut s = 0.0;
    let n: i32 = 12;
    for k in 0..n {
        let rx = x.sqrt();
        let ry = y.sqrt();
        let rz = z.sqrt();
        let rp = p.sqrt();

        let a = rx*ry+rx*rz+ry*rz;
        let d = (rp+rx)*(rp+ry)*(rp+rz);
        let e = (4f64).powi(-3*k)/(d*d)*delta;

        x = 0.25*(x+a);
        y = 0.25*(y+a);
        z = 0.25*(z+a);
        p = 0.25*(p+a);
        s += (4f64).powi(-k)/d*RC(1.0,1.0+e);
    }
    return (x).powf(-3.0/2.0)*(4f64).powi(-n)+6.0*s;
}

fn RD(x: f64, y: f64, z: f64) -> f64 {
    return RJ(x,y,z,z);
}

fn eiF(phi: f64, m: f64) -> f64 {
    let s = (phi).sin();
    let c = (phi).cos();
    return s*RF(c*c,1.0-m*s*s,1.0);
}

fn eiE(phi: f64, m: f64) -> f64 {
    let s = (phi).sin();
    let c = (phi).cos();
    let mss = m*s*s;
    return s*RF(c*c,1.0-mss,1.0)-1.0/3.0*mss*s*RJ(c*c,1.0-mss,1.0,1.0);
}

fn eiPi(phi: f64, n: f64, m: f64) -> f64 {
    let s = (phi).sin();
    let c = (phi).cos();
    let mss = m*s*s;
    let nss = n*s*s;
    return s*RF(c*c,1.0-mss,1.0)+1.0/3.0*nss*s*RJ(c*c,1.0-mss,1.0,1.0-nss);
}

fn legendre_rec(n: i32, m: i32, x: f64) -> f64{
    if n==m {
        return SQRT_PI/gamma(0.5-float(n))*(2.0*(1.0-x*x).sqrt()).powi(n);
    }else if n-1==m {
        return x*float(2*n-1)*legendre_rec(m,m,x);
    }else{
        let mut a = legendre_rec(m,m,x);
        let mut b = legendre_rec(m+1,m,x);
        let mf = float(m);
        for k in m+2..n+1 {
            let k = float(k);
            let h = ((2.0*k-1.0)*x*b-(k-1.0+mf)*a)/(k-mf);
            a=b; b=h;
        }
        return b;
    }
}

pub fn legendre(n: i32, m: i32, x: f64) -> f64 {
    let n = if n<0 {-n-1} else {n};
    if m.abs()>n {
        return 0.0;
    }else if m<0 {
        return NAN;
    }else{
        return legendre_rec(n,m,x);
    }
}

fn hermite(n: i32, x: f64) -> f64 {
    if n<2 {
        return if n==1 {2.0*x} else if n==0 {1.0} else {NAN};
    }else{
        let mut a = 1.0;
        let mut b = 2.0*x;
        for k in 1..n {
            let h = 2.0*x*b-2.0*float(k)*a;
            a = b; b = h;
        }
        return b;
    }
}

fn chebyshev1(n: i32, x: f64) -> f64 {
    if n<2 {
        return if n==0 {1.0} else if n==1 {x} else {NAN};
    }else{
        let mut a = 1.0;
        let mut b = x;
        for _ in 1..n {
            let h = 2.0*x*b-a;
            a = b; b = h;
        }
        return b;
    }
}

fn chebyshev2(n: i32, x: f64) -> f64 {
  if n<2 {
      return if n==0 {1.0} else if n==1 {2.0*x} else {NAN};
  }else{
      let mut a = 1.0;
      let mut b = 2.0*x;
      for _ in 1..n {
          let h = 2.0*x*b-a;
          a = b; b = h;
      }
      return b;
  }
}

fn upper_gamma_cf(s: f64, z: f64, n: u32) -> f64 {
  let mut x = 0.0;
  let mut k = n;
  while k != 0 {
      let kf = float(k);
      x = kf*(s-kf)/(2.0*kf+1.0+z-s+x);
      k-=1;
  }
  return (-z).exp()/(1.0+z-s+x);
}

fn lower_gamma_series(s: f64, z: f64, n: u32) -> f64 {
  let mut y = 0.0;
  let mut p = 1.0/s;
  let mut k: u32 = 1;
  while k<=n {
      y+=p;
      p = p*z/(s+float(k));
      k+=1;
  }
  return y*(-z).exp();
}

fn upper_gamma(s: f64, z: f64) -> f64{
    if s+4.0<z {
        return z.powf(s)*upper_gamma_cf(s,z,40);
    }else{
        return gamma(s)-z.powf(s)*lower_gamma_series(s,z,60);
    }
}

fn lower_gamma(s: f64, z: f64) -> f64 {
    if s+4.0<z {
        return gamma(s)-z.powf(s)*upper_gamma_cf(s,z,40);
    }else{
        return z.powf(s)*lower_gamma_series(s,z,60);
    }
}

// by Euler-Maclaurin summation formula
fn hurwitz_zeta(s: f64, a: f64) -> f64 {
    let mut y = 0.0;
    let N = 18;
    let Npa = float(N)+a;
    for k in 0..N {
        y += (float(k)+a).powf(-s);
    }
    let s2 = s*(s+1.0)*(s+2.0);
    let s4 = s2*(s+3.0)*(s+4.0);
    return y
    + Npa.powf(1.0-s)/(s-1.0)+0.5*Npa.powf(-s)
    + s*Npa.powf(-s-1.0)/12.0
    - s2*Npa.powf(-s-3.0)/720.0
    + s4*Npa.powf(-s-5.0)/30240.0
    - s4*(s+5.0)*(s+6.0)*Npa.powf(-s-7.0)/1209600.0;
}

// by reflection formula (Riemann functional equation)
fn zeta(s: f64) -> f64 {
    if s>60.0 {
        return 1.0;
    }else if s>-1.0 {
        return hurwitz_zeta(s,1.0);
    }else{
        let a = 2.0*(2.0*PI).powf(s-1.0)*(0.5*PI*s).sin();
        return a*gamma(1.0-s)*hurwitz_zeta(1.0-s,1.0);
    }
}

fn czeta_em(s: c64) -> c64 {
    let N = 18;
    let mut y = c64{re: 1.0, im: 0.0};
    for k in 2..N {
        y = y+(-s).expf(float(k));
    }
    let Nf = float(N);
    let s2 = s*(s+1.0)*(s+2.0);
    let s4 = s2*(s+3.0)*(s+4.0);
    let s6 = s4*(s+5.0)*(s+6.0);
    return y
    +(1.0-s).expf(Nf)/(s-1.0)+0.5*(-s).expf(Nf)
    + s*(-s-1.0).expf(Nf)/12.0
    - s2*(-s-3.0).expf(Nf)/720.0
    + s4*(-s-5.0).expf(Nf)/30240.0
    - s6*(-s-7.0).expf(Nf)/1209600.0;
}

fn czeta(s: c64) -> c64 {
    if s.re > -1.0 {
        return czeta_em(s);
    }else{
        let a = 2.0*(s-1.0).expf(2.0*PI)*(0.5*PI*s).sin();
        return a*cgamma(1.0-s)*czeta_em(1.0-s);
    }
}

fn bernoulli_number(n: f64) -> f64 {
    return if n==0.0 {1.0} else {-n*zeta(1.0-n)};
}

fn beta(x: f64, y: f64) -> f64 {
    let s = sgngamma(x)*sgngamma(y)*sgngamma(x+y);
    return s*(lgamma(x)+lgamma(y)-lgamma(x+y)).exp();
}

#[inline(never)]
fn type_error_int_float(env: &mut Env, fapp: &str, id: &str, x: &Object)
-> FnResult
{
    env.type_error1(&format!(
        "Type error in {}: {} shall be of type Int or Float",
    fapp,id),id,x)
}

fn sf_K(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"K")
    }
    let m = match argv[0] {
        Object::Int(m) => float(m),
        Object::Float(m) => m,
        ref m => return type_error_int_float(env,"K(m)","m",m)
    };
    Ok(Object::Float(cK(m)))
}

fn sf_E(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {
            let m = match argv[0] {
                Object::Int(m) => float(m),
                Object::Float(m) => m,
                ref m => return type_error_int_float(env,"E(m)","m",m)
            };
            Ok(Object::Float(cE(m)))
        },
        2 => {
            let phi = match argv[0] {
                Object::Int(x) => float(x),
                Object::Float(x) => x,
                ref x => return type_error_int_float(env,"E(phi,m)","phi",x)
            };
            let m = match argv[1] {
                Object::Int(m) => float(m),
                Object::Float(m) => m,
                ref m => return type_error_int_float(env,"E(phi,m)","m",m)
            };
            Ok(Object::Float(eiE(phi,m)))
        },
        n => env.argc_error(n,1,2,"E")
    }
}

fn sf_F(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {
            let phi = match argv[0] {
                Object::Int(x) => float(x),
                Object::Float(x) => x,
                ref x => return type_error_int_float(env,"F(phi,m)","phi",x)
            };
            let m = match argv[1] {
                Object::Int(m) => float(m),
                Object::Float(m) => m,
                ref m => return type_error_int_float(env,"F(phi,m)","m",m)
            };
            Ok(Object::Float(eiF(phi,m)))
        },
        n => env.argc_error(n,2,2,"F")
    }
}

fn sf_Pi(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"Pi")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"RF(phi,n,m)","phi",x)
    };
    let y = match argv[1] {
        Object::Int(y) => float(y),
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"RF(phi,n,m)","n",y)
    };
    let z = match argv[2] {
        Object::Int(z) => float(z),
        Object::Float(z) => z,
        ref z => return type_error_int_float(env,"RF(phi,n,m)","m",z)
    };
    Ok(Object::Float(eiPi(x,y,z)))
}

fn sf_RF(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"RF")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"RF(x,y,z)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => float(y),
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"RF(x,y,z)","y",y)
    };
    let z = match argv[2] {
        Object::Int(z) => float(z),
        Object::Float(z) => z,
        ref z => return type_error_int_float(env,"RF(x,y,z)","z",z)
    };
    Ok(Object::Float(RF(x,y,z)))
}

fn sf_RC(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"RC")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"RC(x,y)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => float(y),
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"RC(x,y)","y",y)
    };
    Ok(Object::Float(RC(x,y)))
}

fn sf_RJ(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        4 => {}, n => return env.argc_error(n,4,4,"RJ")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"RJ(x,y,z,p)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => float(y),
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"RJ(x,y,z,p)","y",y)
    };
    let z = match argv[2] {
        Object::Int(z) => float(z),
        Object::Float(z) => z,
        ref z => return type_error_int_float(env,"RJ(x,y,z,p)","z",z)
    };
    let p = match argv[3] {
        Object::Int(p) => float(p),
        Object::Float(p) => p,
        ref p => return type_error_int_float(env,"RJ(x,y,z,p)","p",p)
    };
    Ok(Object::Float(RJ(x,y,z,p)))
}

fn sf_RD(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"RD")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"RD(x,y,z)","x",x)
    };
    let y = match argv[1] {
        Object::Int(y) => float(y),
        Object::Float(y) => y,
        ref y => return type_error_int_float(env,"RD(x,y,z)","y",y)
    };
    let z = match argv[2] {
        Object::Int(z) => float(z),
        Object::Float(z) => z,
        ref z => return type_error_int_float(env,"RD(x,y,z)","z",z)
    };
    Ok(Object::Float(RD(x,y,z)))
}

fn sf_PP(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        3 => {}, n => return env.argc_error(n,3,3,"PP")
    }
    let n = match argv[0] {
        Object::Int(x) => x,
        ref x => return type_error_int_float(env,"PP(n,m,x)","n",x)
    };
    let m = match argv[1] {
        Object::Int(x) => x,
        ref x => return type_error_int_float(env,"PP(n,m,x)","m",x)
    };
    let x = match argv[2] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"PP(n,m,x)","x",x)
    };
    Ok(Object::Float(legendre(n,m,x)))
}

fn sf_PH(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"PH")
    }
    let n = match argv[0] {
        Object::Int(x) => x,
        ref x => return type_error_int_float(env,"PH(n,x)","n",x)
    };
    let x = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"PH(n,x)","x",x)
    };
    Ok(Object::Float(hermite(n,x)))
}

fn sf_PT(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"PT")
    }
    let n = match argv[0] {
        Object::Int(x) => x,
        ref x => return type_error_int_float(env,"PT(n,x)","n",x)
    };
    let x = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"PT(n,x)","x",x)
    };
    Ok(Object::Float(chebyshev1(n,x)))
}

fn sf_PU(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"PU")
    }
    let n = match argv[0] {
        Object::Int(x) => x,
        ref x => return type_error_int_float(env,"PU(n,x)","n",x)
    };
    let x = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"PU(n,x)","x",x)
    };
    Ok(Object::Float(chebyshev2(n,x)))
}

fn sf_gamma(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"gamma")
    }
    let s = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"gamma(s,x)","s",x)
    };
    let x = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"gamma(s,x)","x",x)
    };
    Ok(Object::Float(lower_gamma(s,x)))
}

fn sf_Gamma(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"Gamma")
    }
    let s = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"Gamma(s,x)","s",x)
    };
    let x = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"Gamma(s,x)","x",x)
    };
    Ok(Object::Float(upper_gamma(s,x)))
}

fn sf_zeta(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"zeta")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        Object::Complex(z) => {
            return Ok(Object::Complex(czeta(z)));
        },
        ref x => return type_error_int_float(env,"zeta(x)","x",x)
    };
    Ok(Object::Float(zeta(x)))
}

fn sf_B(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        1 => {}, n => return env.argc_error(n,1,1,"B")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"B(x)","x",x)
    };
    Ok(Object::Float(bernoulli_number(x)))
}

fn sf_Beta(env: &mut Env, _pself: &Object, argv: &[Object]) -> FnResult {
    match argv.len() {
        2 => {}, n => return env.argc_error(n,2,2,"Beta")
    }
    let x = match argv[0] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"Beta(x,y)","x",x)
    };
    let y = match argv[1] {
        Object::Int(x) => float(x),
        Object::Float(x) => x,
        ref x => return type_error_int_float(env,"Beta(x,y)","y",x)
    };
    Ok(Object::Float(beta(x,y)))
}

pub fn load_sf_ei() -> Object {
    let ei = new_module("ei");
    {
        let mut m = ei.map.borrow_mut();
        m.insert_fn_plain("K",sf_K,1,1);
        m.insert_fn_plain("E",sf_E,1,2);
        m.insert_fn_plain("F",sf_F,2,2);
        m.insert_fn_plain("Pi",sf_Pi,3,3);
        m.insert_fn_plain("RF",sf_RF,3,3);
        m.insert_fn_plain("RC",sf_RC,2,2);
        m.insert_fn_plain("RJ",sf_RJ,4,4);
        m.insert_fn_plain("RD",sf_RD,3,3);
    }
    return Object::Interface(Rc::new(ei));
}

pub fn load_sf() -> Object {
    let sf = new_module("sf");
    {
        let mut m = sf.map.borrow_mut();
        m.insert_fn_plain("PP",sf_PP,3,3);
        m.insert_fn_plain("PH",sf_PH,2,2);
        m.insert_fn_plain("PT",sf_PT,2,2);
        m.insert_fn_plain("PU",sf_PU,2,2);
        m.insert_fn_plain("gamma",sf_gamma,2,2);
        m.insert_fn_plain("Gamma",sf_Gamma,2,2);
        m.insert_fn_plain("zeta",sf_zeta,1,1);
        m.insert_fn_plain("B",sf_B,1,1);
        m.insert_fn_plain("Beta",sf_Beta,2,2);
    }
    return Object::Interface(Rc::new(sf));
}

