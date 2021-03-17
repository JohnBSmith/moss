
use std::ops::{Add,Sub,Mul,Div,Neg,AddAssign};
use std::fmt;

#[derive(Clone,Copy,PartialEq)]
pub struct Complex64 {
    pub re: f64,
    pub im: f64
}

#[allow(non_camel_case_types)]
pub type C64 = Complex64;

fn float_to_string(x: f64) -> String {
    if x == 0.0 {
        "0".to_string()
    } else if x.abs() > 1E14 || x.abs() < 0.0001 {
        format!("{:e}",x)
    } else {
        format!("{}",x)
    }
}

impl fmt::Display for C64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.im < 0.0 {
            write!(f, "{}{}i", float_to_string(self.re), float_to_string(self.im))
        } else {
            write!(f, "{}+{}i", float_to_string(self.re), float_to_string(self.im))
        }
    }
}

impl Add for C64 {
    type Output = C64;
    fn add(self, b: C64) -> C64 {
        C64 {re: self.re+b.re, im: self.im+b.im}
    }
}

impl Add<f64> for C64 {
    type Output = C64;
    fn add(self, b: f64) -> C64 {
        C64 {re: self.re+b, im: self.im}
    }
}

impl Add<C64> for f64 {
    type Output = C64;
    fn add(self, b: C64) -> C64 {
        C64 {re: self+b.re, im: b.im}
    }
}

impl Sub for C64 {
    type Output = C64;
    fn sub(self, b: C64) -> C64 {
        C64 {re: self.re-b.re, im: self.im-b.im}
    }
}

impl Sub<f64> for C64 {
    type Output = C64;
    fn sub(self, b: f64) -> C64 {
        C64 {re: self.re-b, im: self.im}
    }
}

impl Sub<C64> for f64 {
    type Output = C64;
    fn sub(self, b: C64) -> C64 {
        C64 {re: self-b.re, im: -b.im}
    }
}

impl Mul<C64> for f64 {
    type Output = C64;
    fn mul(self, b: C64) -> C64 {
        C64 {re: self*b.re, im: self*b.im}
    }
}

impl Mul for C64 {
    type Output = C64;
    fn mul(self, b: C64) -> C64 {
        C64 {
            re: self.re*b.re-self.im*b.im,
            im: self.re*b.im+self.im*b.re
        }
    }
}

impl Div<C64> for f64 {
    type Output = C64;
    fn div(self, b: C64) -> C64 {
        let r2 = b.re*b.re+b.im*b.im;
        C64 {re: self*b.re/r2, im: -self*b.im/r2}
    }
}

impl Div for C64 {
    type Output = C64;
    fn div(self, b: C64) -> C64 {
        let r2 = b.re*b.re+b.im*b.im;
        C64 {
            re: (self.re*b.re+self.im*b.im)/r2,
            im: (self.im*b.re-self.re*b.im)/r2
        }
    }
}

impl Div<f64> for C64 {
    type Output = C64;
    fn div(self, b: f64) -> C64 {
        C64 {re: self.re/b, im: self.im/b}
    }
}

impl Neg for C64 {
    type Output = C64;
    fn neg(self) -> C64 {
        C64 {re: -self.re, im: -self.im}
    }
}

impl AddAssign for C64 {
    fn add_assign(&mut self, b: C64) {
        self.re += b.re;
        self.im += b.im;
    }
}

impl C64 {
    pub fn conj(self) -> C64 {
        C64 {re: self.re, im: -self.im}
    }
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
    pub fn abs_square(self) -> f64 {
        self.re*self.re+self.im*self.im
    }
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }
    pub fn to_polar(self) -> (f64,f64) {
        (self.re.hypot(self.im), self.im.atan2(self.re))
    }
    pub fn from_polar(r: f64, phi: f64) -> C64 {
        C64 {re: r*phi.cos(), im: r*phi.sin()}
    }
    pub fn exp(self) -> C64 {
        let r = self.re.exp();
        C64{re: r*self.im.cos(), im: r*self.im.sin()}
    }
    pub fn ln(self) -> C64 {
        let (r,phi) = self.to_polar();
        C64 {re: r.ln(), im: phi}
    }
    pub fn sqrt(self) -> C64 {
        let (r,phi) = self.to_polar();
        C64::from_polar(r.sqrt(),0.5*phi)
    }
    pub fn powf(self, a: f64) -> C64 {
        let (r,phi) = self.to_polar();
        C64::from_polar(r.powf(a), phi*a)
    }
    pub fn powc(self, w: C64) -> C64 {
        let (r,phi) = self.to_polar();
        let lnr = r.ln();
        C64::from_polar(
            (w.re*lnr-phi*w.im).exp(),
            w.re*phi+w.im*lnr
        )
    }
    pub fn expf(self, base: f64) -> C64 {
        C64::from_polar(base.powf(self.re), self.im*base.ln())
    }

    // sin(a+bi) = sin(a)cosh(b) + i*cos(a)sinh(b)
    pub fn sin(self) -> C64 {
        C64 {
            re: self.re.sin() * self.im.cosh(),
            im: self.re.cos() * self.im.sinh()
        }
    }

    // cos(a+bi) = cos(a)cosh(b) - i*sin(a)sinh(b)
    pub fn cos(self) -> C64 {
        C64 {
            re:  self.re.cos() * self.im.cosh(),
            im: -self.re.sin() * self.im.sinh()
        }
    }

    // tan(a+bi) = (sin(2a) + i*sinh(2b))/(cos(2a) + cosh(2b))
    pub fn tan(self) -> C64 {
        let x = 2.0*self.re;
        let y = 2.0*self.im;
        let r = 1.0/(x.cos()+y.cosh());
        C64 {re: r*x.sin(), im: r*y.sinh()}
    }

    pub fn cot(self) -> C64 {
        1.0/self.tan()
    }

    // sinh(a+bi) = sinh(a)cos(b) + i*cosh(a)sin(b)
    pub fn sinh(self) -> C64 {
        C64 {
            re: self.re.sinh() * self.im.cos(),
            im: self.re.cosh() * self.im.sin()
        }
    }

    // cosh(a+bi) = cosh(a)cos(b) + i*sinh(a)sin(b)
    pub fn cosh(self) -> C64 {
        C64 {
            re: self.re.cosh() * self.im.cos(),
            im: self.re.sinh() * self.im.sin()
        }
    }

    // tanh(a+bi) = (sinh(2a) + i*sin(2b))/(cosh(2a) + cos(2b))
    pub fn tanh(self) -> C64 {
        let x = 2.0*self.re;
        let y = 2.0*self.im;
        let r = 1.0/(x.cosh()+y.cos());
        C64 {re: r*x.sinh(), im: r*y.sin()}
    }

    pub fn coth(self) -> C64 {
        1.0/self.tanh()
    }

    // asin(z) = -i*ln(sqrt(1-z^2) + i*z)
    pub fn asin(self) -> C64 {
        let i = C64 {re: 0.0, im: 1.0};
        -i*((1.0-self*self).sqrt()+i*self).ln()
    }

    // acos(z) = -i*ln(i*sqrt(1-z^2) + z)  
    pub fn acos(self) -> C64 {
        let i = C64 {re: 0.0, im: 1.0};
        -i*(i*(1.0-self*self).sqrt()+self).ln()
    }

    // atan(z) = (ln(1+iz) - ln(1-iz))/(2i)
    pub fn atan(self) -> C64 {
        let i = C64 {re: 0.0, im: 1.0};
        ((1.0+i*self).ln()-(1.0-i*self).ln())/(2.0*i)
    }

    // asinh(z) = ln(z + sqrt(1+z^2))
    pub fn asinh(self) -> C64 {
        (self+(1.0+self*self).sqrt()).ln()
    }

    // acosh(z) = 2 ln(sqrt((z+1)/2) + sqrt((z-1)/2))
    pub fn acosh(self) -> C64 {
        2.0*((0.5*(self+1.0)).sqrt() + (0.5*(self-1.0)).sqrt()).ln()
    }

    // atanh(z) = (ln(1+z) - ln(1-z))/2
    pub fn atanh(self) -> C64 {
        0.5*((1.0+self).ln() - (1.0-self).ln())
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


