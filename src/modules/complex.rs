
use std::ops;

#[derive(Clone,Copy,PartialEq)]
pub struct Complex64{
  pub re: f64,
  pub im: f64
}

impl ops::Add for Complex64{
  type Output = Complex64;
  fn add(self, b: Complex64) -> Complex64{
    Complex64{re: self.re+b.re, im: self.im+b.im}
  }
}

impl ops::Add<f64> for Complex64{
  type Output = Complex64;
  fn add(self, b: f64) -> Complex64{
    Complex64{re: self.re+b, im: self.im}
  }
}

impl ops::Add<Complex64> for f64{
  type Output = Complex64;
  fn add(self, b: Complex64) -> Complex64{
    Complex64{re: self+b.re, im: b.im}
  }
}

impl ops::Sub for Complex64{
  type Output = Complex64;
  fn sub(self, b: Complex64) -> Complex64{
    Complex64{re: self.re-b.re, im: self.im-b.im}
  }
}

impl ops::Sub<f64> for Complex64{
  type Output = Complex64;
  fn sub(self, b: f64) -> Complex64{
    Complex64{re: self.re-b, im: self.im}
  }
}

impl ops::Sub<Complex64> for f64{
  type Output = Complex64;
  fn sub(self, b: Complex64) -> Complex64{
    Complex64{re: self-b.re, im: -b.im}
  }
}

impl ops::Mul<Complex64> for f64{
  type Output = Complex64;
  fn mul(self, b: Complex64) -> Complex64{
    Complex64{re: self*b.re, im: self*b.im}
  }
}

impl ops::Mul for Complex64{
  type Output = Complex64;
  fn mul(self, b: Complex64) -> Complex64{
    Complex64{re: self.re*b.re-self.im*b.im, im: self.re*b.im+self.im*b.re}
  }
}

impl ops::Div<Complex64> for f64{
  type Output = Complex64;
  fn div(self, b: Complex64) -> Complex64{
    let r2 = b.re*b.re+b.im*b.im;
    Complex64{re: self*b.re/r2, im: -self*b.im/r2}
  }
}

impl ops::Div for Complex64{
  type Output = Complex64;
  fn div(self, b: Complex64) -> Complex64{
    let r2 = b.re*b.re+b.im*b.im;
    Complex64{re: (self.re*b.re+self.im*b.im)/r2, im: (self.im*b.re-self.re*b.im)/r2}
  }
}

impl ops::Div<f64> for Complex64{
  type Output = Complex64;
  fn div(self, b: f64) -> Complex64{
    Complex64{re: self.re/b, im: self.im/b}
  }
}

impl ops::Neg for Complex64{
  type Output = Complex64;
  fn neg(self) -> Complex64{
    Complex64{re: -self.re, im: -self.im}
  }
}

impl Complex64{
  pub fn conj(self) -> Complex64{
    Complex64{re: self.re, im: -self.im}
  }
  pub fn abs(self) -> f64{
    self.re.hypot(self.im)
  }
  pub fn abs_square(self) -> f64{
    self.re*self.re+self.im*self.im
  }
  pub fn arg(self) -> f64{
    self.im.atan2(self.re)
  }
  pub fn to_polar(self) -> (f64,f64){
    (self.re.hypot(self.im), self.im.atan2(self.re))
  }
  pub fn from_polar(r: f64, phi: f64) -> Complex64{
    Complex64{re: r*phi.cos(), im: r*phi.sin()}
  }
  pub fn exp(self) -> Complex64{
    let r = self.re.exp();
    Complex64{re: r*self.im.cos(), im: r*self.im.sin()}
  }
  pub fn ln(self) -> Complex64{
    let (r,phi) = self.to_polar();
    Complex64{re: r.ln(), im: phi}
  }
  pub fn sqrt(self) -> Complex64{
    let (r,phi) = self.to_polar();
    Complex64::from_polar(r.sqrt(),0.5*phi)
  }
  pub fn powf(self, a: f64) -> Complex64{
    let (r,phi) = self.to_polar();
    Complex64::from_polar(r.powf(a), phi*a)
  }
  pub fn pow(self, w: Complex64) -> Complex64{
    let (r,phi) = self.to_polar();
    let lnr = r.ln();
    Complex64::from_polar((w.re*lnr-phi*w.im).exp(),phi+w.im*lnr)
  }
  pub fn expa(self, base: f64) -> Complex64{
    Complex64::from_polar(base.powf(self.re), base.ln()+self.im)
  }

  // sin(a+bi) = sin(a)cosh(b) + i*cos(a)sinh(b)
  pub fn sin(self) -> Complex64{
    Complex64{
      re: self.re.sin() * self.im.cosh(),
      im: self.re.cos() * self.im.sinh()
    }
  }

  // cos(a+bi) = cos(a)cosh(b) - i*sin(a)sinh(b)
  pub fn cos(self) -> Complex64{
    Complex64{
      re:  self.re.cos() * self.im.cosh(),
      im: -self.re.sin() * self.im.sinh()
    }
  }

  // tan(a+bi) = (sin(2a) + i*sinh(2b))/(cos(2a) + cosh(2b))
  pub fn tan(self) -> Complex64{
    let x = 2.0*self.re;
    let y = 2.0*self.im;
    let r = 1.0/(x.cos()+y.cosh());
    Complex64{re: r*x.sin(), im: r*y.sinh()}
  }

  pub fn cot(self) -> Complex64{
    1.0/self.tan()
  }

  // sinh(a+bi) = sinh(a)cos(b) + i*cosh(a)sin(b)
  pub fn sinh(self) -> Complex64{
    Complex64{
      re: self.re.sinh() * self.im.cos(),
      im: self.re.cosh() * self.im.sin()
    }
  }
  // cosh(a+bi) = cosh(a)cos(b) + i*sinh(a)sin(b)
  pub fn cosh(self) -> Complex64{
    Complex64{
      re: self.re.cosh() * self.im.cos(),
      im: self.re.sinh() * self.im.sin()
    }
  }
  // tanh(a+bi) = (sinh(2a) + i*sin(2b))/(cosh(2a) + cos(2b))
  pub fn tanh(self) -> Complex64{
    let x = 2.0*self.re;
    let y = 2.0*self.im;
    let r = 1.0/(x.cosh()+y.cos());
    Complex64{re: r*x.sinh(), im: r*y.sin()}
  }
  
  pub fn coth(self) -> Complex64{
    1.0/self.tanh()
  }
  
  pub fn is_nan(self) -> bool {
    self.re.is_nan() || self.im.is_nan()
  }
  pub fn is_infinite(self) -> bool {
    !self.is_nan() && (self.re.is_infinite() || self.im.is_infinite())
  }
  pub fn is_finite(self) -> bool {
    self.re.is_finite() || self.im.is_finite()
  }
  pub fn is_normal(self) -> bool {
    self.re.is_normal() && self.im.is_normal()
  }
}

pub fn la(){
  let z = Complex64{re: 1.0, im: 0.0};
  let w = z.pow(z);
}
